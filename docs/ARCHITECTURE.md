# ARCHITECTURE

## 全体像

このリポジトリは、Spout2 の必要最小ソースをビルドに取り込み、`spoutdx_ffi`（DLL）として **C ABI** を公開します。

### 主要コンポーネント

```
┌─────────────────────────────────────────┐
│  利用側（Rust / C / C++）                │
│  - FFI 経由で C ABI を呼び出し           │
└─────────────────────────────────────────┘
                   ↓ C ABI
┌─────────────────────────────────────────┐
│  spoutdx_ffi.dll（C ABI シム）           │
│  - extern "C" 公開関数                   │
│  - 例外安全性確保（try/catch）           │
│  - エラーコード変換                      │
└─────────────────────────────────────────┘
                   ↓ C++ 内部呼び出し
┌─────────────────────────────────────────┐
│  Spout2 ソース（静的統合）               │
│  - SpoutDX クラス（DirectX 11 実装）     │
│  - SpoutUtils, SpoutCopy 等              │
└─────────────────────────────────────────┘
```

**根拠**: C ABI 境界により、Rust/C/C++ のどの言語からも安全に呼び出せる。Spout をソース統合することで、C++ ABI 互換性問題を回避。

### ファイル構成

- **spoutdx_ffi（DLL）**
  - 役割: Rust から呼ぶための C ABI シム
  - 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp)
  - 公開ヘッダ: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h)

- **Spout2 ソース（ビルドに統合）**
  - 役割: SpoutDX（DX11）本体
  - 配置: [third_party/Spout2/SPOUTSDK/](../third_party/Spout2/SPOUTSDK/)
  - 主要クラス: `spoutDX`（DirectX 11 実装）

- **サンプル**
  - 役割: ローカル疎通（SDK バージョン/DX11 初期化/Receiver API）
  - 実装: [examples/src/main.rs](../examples/src/main.rs) (Rust)

## レイヤ構成

### 1. C ABI 層（spoutdx_ffi.dll）

**責務**:
- C ABI 境界の維持
- 例外の捕捉とエラーコード変換
- 不透明ハンドルの管理
- 型変換（C++ ↔ C）

**重要な設計判断**:
- すべての公開関数は `extern "C"` で宣言
- C++ 例外が境界を越えないよう `try/catch (...)` で捕捉
- 内部 C++ クラスは `void*` ハンドルとして公開
- STL 型（`std::string`, `std::vector` 等）は境界に出さない

**実装例**:
```cpp
SpoutDxReceiverHandle spoutdx_receiver_create() {
    try {
        return new SpoutDxReceiver();
    } catch (...) {
        return nullptr;
    }
}
```

**詳細**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp)

### 2. 内部 C++ ラッパー層

**責務**:
- SpoutDX クラスのライフサイクル管理
- 複数 Receiver/Sender インスタンスの管理（将来）

**実装例**:
```cpp
class SpoutDxReceiver {
public:
    spoutDX dx;  // SpoutDX インスタンス

    SpoutDxReceiver() = default;
    ~SpoutDxReceiver() {
        dx.ReleaseReceiver();
        dx.CloseDirectX11();
    }
};
```

**根拠**: RAII パターンにより、リソースリークを防止。デストラクタで確実にリソースを解放。

**詳細**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L40-L48)

### 3. Spout2 本体層

**責務**:
- DirectX 11 デバイス管理
- 共有メモリ/テクスチャ管理
- センダー/レシーバーの実装

**主要クラス**: `spoutDX`
- 配置: [third_party/Spout2/SPOUTSDK/SpoutDirectX/SpoutDX/SpoutDX.h](../third_party/Spout2/SPOUTSDK/SpoutDirectX/SpoutDX/SpoutDX.h)

## ビルド構成

### CMake プリセット

- **定義**: [CMakePresets.json](../CMakePresets.json)
- **プリセット**:
  - `msvc-debug`: Debug ビルド（Visual Studio 2022, x64）
  - `msvc-release`: Release ビルド（Visual Studio 2022, x64）

**ビルド手順**:
```powershell
# Debug
cmake --preset msvc-debug
cmake --build --preset msvc-debug

# Release
cmake --preset msvc-release
cmake --build --preset msvc-release
```

### 重要な CMake ロジック

#### 1. Spout ソース存在チェック

```cmake
if (NOT EXISTS "${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutCommon.h")
  message(FATAL_ERROR "Spout source not found at: ${SPOUT_SOURCE_ROOT}")
endif()
```

**根拠**: ビルド時に Spout ソースの存在を確認し、欠落時にエラーを明示。

#### 2. Spout ソースの統合

```cmake
set(SPOUT_SOURCES
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutCopy.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutDirectX.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutFrameCount.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutSenderNames.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutSharedMemory.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutUtils.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutDirectX/SpoutDX/SpoutDX.cpp
)

add_library(spoutdx_ffi SHARED
  src/spoutdx_ffi.cpp
  ${SPOUT_SOURCES}
)
```

**根拠**: Spout ソースを直接コンパイルに投入することで、C++ ABI 互換性問題を回避。

#### 3. Windows ライブラリのリンク

```cmake
target_link_libraries(spoutdx_ffi PRIVATE 
  d3d11 
  dxgi
  psapi      # GetModuleFileNameEx (SpoutUtils)
  version    # version resources (SpoutUtils)
  winmm      # timeGetTime/timeBeginPeriod/timeEndPeriod (SpoutFrameCount)
)
```

**根拠**: Spout が依存する Windows ライブラリを明示的にリンク。

**詳細**: [CMakeLists.txt](../CMakeLists.txt)

## データフロー

### 基本 API（疎通テスト）

```
Rust (examples/main.rs)
  ↓ FFI 呼び出し
spoutdx_ffi_get_sdk_version() [C ABI]
  ↓ 内部呼び出し
spoututils::GetSDKversion(&version) [C++]
  ↓ 戻り値
Rust へ返却
```

**詳細**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L14-L19)

### Receiver API（受信フロー）

#### 1. 初期化フロー

```
Rust 側
  ↓ 1. D3D11Device* 作成
  ↓ 2. spoutdx_receiver_create()
  ↓ 3. spoutdx_receiver_open_dx11(handle, device)
  
spoutdx_ffi.dll (C ABI)
  ↓ new SpoutDxReceiver()
  ↓ rx->dx.OpenDirectX11(pDevice)
  
SpoutDX (C++)
  ↓ デバイスを保存
  ↓ コンテキスト取得
```

**根拠**: 呼び出し側が D3D11 デバイスを管理し、spoutdx-ffi はそれを借用する形で動作。これにより、既存パイプラインとの統合が容易。

#### 2. 受信フロー（receive_texture）

```
Rust 側
  ↓ 1. ID3D11Texture2D* 作成（出力先）
  ↓ 2. spoutdx_receiver_receive_texture(handle, texture)
  
spoutdx_ffi.dll (C ABI)
  ↓ rx->dx.ReceiveTexture(&pTexturePtr)
  
SpoutDX (C++)
  ↓ センダー検索
  ↓ 共有テクスチャから出力先へコピー
  ↓ 成功/失敗を返却
  
Rust 側
  ↓ 3. 出力先テクスチャを使用
```

**根拠**: GPU 間での直接コピーにより、CPU を経由せず高速にテクスチャを転送。

**詳細**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L119-L139)

#### 3. 受信フロー（receive - 内部テクスチャ）

```
Rust 側
  ↓ 1. spoutdx_receiver_receive(handle)
  
spoutdx_ffi.dll (C ABI)
  ↓ rx->dx.ReceiveTexture()
  
SpoutDX (C++)
  ↓ センダー検索
  ↓ 内部テクスチャへ受信
  
Rust 側
  ↓ 2. spoutdx_receiver_get_received_texture(handle)
  ↓ 3. 内部テクスチャのポインタを取得
  ↓ 4. CopyResource 等で自前テクスチャへコピー
```

**根拠**: 簡易ツールやテストコードでは、テクスチャ管理を簡略化できる。

**詳細**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L141-L177)

#### 4. 状態取得フロー

```
Rust 側
  ↓ spoutdx_receiver_is_connected(handle)
  ↓ spoutdx_receiver_is_updated(handle)
  ↓ spoutdx_receiver_is_frame_new(handle)
  ↓ spoutdx_receiver_get_sender_info(handle, &info)
  
spoutdx_ffi.dll (C ABI)
  ↓ rx->dx.IsConnected()
  ↓ rx->dx.IsUpdated()
  ↓ rx->dx.IsFrameNew()
  ↓ rx->dx.GetSenderName/Width/Height/Format()
  
SpoutDX (C++)
  ↓ 内部状態を返却
```

**根拠**: センダーの状態変化（サイズ変更、フォーマット変更、新規フレーム）を検出し、適切に対応できるようにする。

**詳細**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L194-L231)

## 将来拡張の設計

### Sender API の追加

計画は [ROADMAP.md](ROADMAP.md) に記載。基本的な設計方針は Receiver と同様:

- 不透明ハンドル（`SpoutDxSenderHandle`）
- 外部デバイスでの初期化
- テクスチャ送信（`send_texture`）
- エラーコード（`SpoutDxResult`）

### センダー一覧取得

```c
// 想定される API（未実装）
typedef struct SpoutDxSenderList {
    char** names;
    unsigned int count;
} SpoutDxSenderList;

SPOUTDX_FFI_API int spoutdx_get_sender_list(SpoutDxSenderList* out_list);
SPOUTDX_FFI_API void spoutdx_free_sender_list(SpoutDxSenderList* list);
```

**根拠**: 動的に確保されたメモリは、FFI 境界で明示的に解放する関数を提供する必要がある。

## レイヤ間の責務と境界

### C ABI 層 ↔ 内部 C++ ラッパー層

- **境界**: `void*` ハンドル
- **責務分離**:
  - C ABI 層: 例外捕捉、エラーコード変換、型変換
  - C++ ラッパー層: ライフサイクル管理、SpoutDX クラスのラッピング

### 内部 C++ ラッパー層 ↔ Spout2 本体層

- **境界**: C++ クラスインスタンス
- **責務分離**:
  - C++ ラッパー層: 複数インスタンス管理、RAII パターン
  - Spout2 本体層: DirectX 操作、共有メモリ管理

## 参考

- エラーハンドリング体系: [ERROR_HANDLING.md](ERROR_HANDLING.md)
- テスト戦略: [TESTING_STRATEGY.md](TESTING_STRATEGY.md)
- 詳細実装計画: [impl/RECEIVER_IMPLEMENTATION_PLAN.md](impl/RECEIVER_IMPLEMENTATION_PLAN.md)

