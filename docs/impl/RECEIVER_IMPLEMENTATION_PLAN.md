# Receiver API 実装計画

## 概要

本ドキュメントは、spoutdx-ffi の Receiver API 実装計画を詳細に記述します。
SubAgent による実装を想定し、「実装するだけ」のレベルまで具体化しています。

## 設計方針（DESIGN_PHILOSOPHY準拠）

- **C ABI のみに露出**: `extern "C"` 関数のみを公開
- **不透明ハンドル**: Receiver は `void*` として公開（Rust 側で `*mut c_void`）
- **エラーコード**: 戻り値は `int`（成功=0、失敗=負の値）
- **利用側デバイス優先**: 呼び出し側が D3D11 デバイスを作成・所有

## スコープ

### Phase 1（本実装）
- Receiver ライフサイクル（create/destroy）
- 外部デバイスでの初期化（`OpenDirectX11(pDevice)`）
- テクスチャ受信（`receive_into` 形式）
- センダー情報取得（名前、サイズ、フォーマット）
- 更新チェック（IsUpdated, IsConnected）

### Phase 2（将来）
- 内部デバイス生成モード
- センダー一覧取得
- フレームカウント機能

---

## API 仕様

### 型定義

```c
// 不透明ハンドル
typedef void* SpoutDxReceiverHandle;

// エラーコード
typedef enum SpoutDxResult {
    SPOUTDX_OK                  = 0,
    SPOUTDX_ERROR_NULL_HANDLE   = -1,
    SPOUTDX_ERROR_NULL_DEVICE   = -2,
    SPOUTDX_ERROR_NOT_CONNECTED = -3,
    SPOUTDX_ERROR_INIT_FAILED   = -4,
    SPOUTDX_ERROR_RECEIVE_FAILED = -5,
    SPOUTDX_ERROR_INTERNAL      = -99
} SpoutDxResult;

// センダー情報構造体
typedef struct SpoutDxSenderInfo {
    char name[256];           // センダー名（null終端）
    unsigned int width;       // テクスチャ幅
    unsigned int height;      // テクスチャ高さ
    unsigned int format;      // DXGI_FORMAT 値
} SpoutDxSenderInfo;
```

### ライフサイクル API

```c
// Receiver を作成
// 戻り値: ハンドル（失敗時 NULL）
SPOUTDX_FFI_API SpoutDxReceiverHandle spoutdx_receiver_create(void);

// Receiver を破棄
// 戻り値: SPOUTDX_OK
SPOUTDX_FFI_API int spoutdx_receiver_destroy(SpoutDxReceiverHandle handle);
```

### 初期化 API

```c
// 外部 D3D11 デバイスで初期化
// device: ID3D11Device* を void* として渡す
// 戻り値: エラーコード
SPOUTDX_FFI_API int spoutdx_receiver_open_dx11(
    SpoutDxReceiverHandle handle,
    void* device
);

// DirectX をクローズ
SPOUTDX_FFI_API int spoutdx_receiver_close_dx11(SpoutDxReceiverHandle handle);
```

### 受信 API

```c
// 特定のセンダーに接続設定（NULL で active sender）
SPOUTDX_FFI_API int spoutdx_receiver_set_sender_name(
    SpoutDxReceiverHandle handle,
    const char* sender_name
);

// テクスチャへ受信
// dst_texture: ID3D11Texture2D* を void* として渡す
// 戻り値: SPOUTDX_OK (成功), SPOUTDX_ERROR_NOT_CONNECTED (センダーなし)
SPOUTDX_FFI_API int spoutdx_receiver_receive_texture(
    SpoutDxReceiverHandle handle,
    void* dst_texture
);

// Receiver をリリース（別のセンダーへ再接続用）
SPOUTDX_FFI_API int spoutdx_receiver_release(SpoutDxReceiverHandle handle);
```

### 状態取得 API

```c
// センダー情報を取得（接続後に有効）
SPOUTDX_FFI_API int spoutdx_receiver_get_sender_info(
    SpoutDxReceiverHandle handle,
    SpoutDxSenderInfo* out_info
);

// センダー変更があったか（サイズ・フォーマット変更検出）
SPOUTDX_FFI_API int spoutdx_receiver_is_updated(SpoutDxReceiverHandle handle);

// センダーに接続中か
SPOUTDX_FFI_API int spoutdx_receiver_is_connected(SpoutDxReceiverHandle handle);

// 新しいフレームが届いたか
SPOUTDX_FFI_API int spoutdx_receiver_is_frame_new(SpoutDxReceiverHandle handle);
```

---

## 実装詳細

### ファイル構成

```
include/spoutdx_ffi/
  spoutdx_ffi.h          # 既存 + 新規 API 追加

src/
  spoutdx_ffi.cpp        # 既存 + Receiver 実装追加
```

### 内部クラス構造

```cpp
// src/spoutdx_ffi.cpp 内部

// Receiver ラッパークラス（C++ 内部のみ）
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

### 各関数の実装方針

#### `spoutdx_receiver_create`

```cpp
SpoutDxReceiverHandle spoutdx_receiver_create() {
    try {
        return new SpoutDxReceiver();
    } catch (...) {
        return nullptr;
    }
}
```

#### `spoutdx_receiver_destroy`

```cpp
int spoutdx_receiver_destroy(SpoutDxReceiverHandle handle) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        delete static_cast<SpoutDxReceiver*>(handle);
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}
```

#### `spoutdx_receiver_open_dx11`

```cpp
int spoutdx_receiver_open_dx11(SpoutDxReceiverHandle handle, void* device) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    if (!device) return SPOUTDX_ERROR_NULL_DEVICE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        ID3D11Device* pDevice = static_cast<ID3D11Device*>(device);
        if (!rx->dx.OpenDirectX11(pDevice)) {
            return SPOUTDX_ERROR_INIT_FAILED;
        }
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}
```

#### `spoutdx_receiver_set_sender_name`

```cpp
int spoutdx_receiver_set_sender_name(SpoutDxReceiverHandle handle, const char* name) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        rx->dx.SetReceiverName(name);  // NULL で active sender
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}
```

#### `spoutdx_receiver_receive_texture`

```cpp
int spoutdx_receiver_receive_texture(SpoutDxReceiverHandle handle, void* dst_texture) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        ID3D11Texture2D* pTexture = static_cast<ID3D11Texture2D*>(dst_texture);
        ID3D11Texture2D* pTexturePtr = pTexture;
        
        // ReceiveTexture(ID3D11Texture2D**) を使用
        // この関数は内部でセンダーを探し、共有テクスチャから dst にコピー
        if (!rx->dx.ReceiveTexture(&pTexturePtr)) {
            if (!rx->dx.IsConnected()) {
                return SPOUTDX_ERROR_NOT_CONNECTED;
            }
            return SPOUTDX_ERROR_RECEIVE_FAILED;
        }
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}
```

#### `spoutdx_receiver_get_sender_info`

```cpp
int spoutdx_receiver_get_sender_info(SpoutDxReceiverHandle handle, SpoutDxSenderInfo* out_info) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    if (!out_info) return SPOUTDX_ERROR_INTERNAL;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        
        // センダー名
        const char* name = rx->dx.GetSenderName();
        if (name) {
            strncpy_s(out_info->name, sizeof(out_info->name), name, _TRUNCATE);
        } else {
            out_info->name[0] = '\0';
        }
        
        // サイズ・フォーマット
        out_info->width = rx->dx.GetSenderWidth();
        out_info->height = rx->dx.GetSenderHeight();
        out_info->format = static_cast<unsigned int>(rx->dx.GetSenderFormat());
        
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}
```

#### 状態取得関数群

```cpp
int spoutdx_receiver_is_updated(SpoutDxReceiverHandle handle) {
    if (!handle) return 0;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        return rx->dx.IsUpdated() ? 1 : 0;
    } catch (...) {
        return 0;
    }
}

int spoutdx_receiver_is_connected(SpoutDxReceiverHandle handle) {
    if (!handle) return 0;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        return rx->dx.IsConnected() ? 1 : 0;
    } catch (...) {
        return 0;
    }
}

int spoutdx_receiver_is_frame_new(SpoutDxReceiverHandle handle) {
    if (!handle) return 0;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        return rx->dx.IsFrameNew() ? 1 : 0;
    } catch (...) {
        return 0;
    }
}
```

---

## ヘッダファイル変更

### `include/spoutdx_ffi/spoutdx_ffi.h` 追加内容

```c
#pragma once

#ifdef _WIN32
  #ifdef SPOUTDX_FFI_EXPORTS
    #define SPOUTDX_FFI_API __declspec(dllexport)
  #else
    #define SPOUTDX_FFI_API __declspec(dllimport)
  #endif
#else
  #define SPOUTDX_FFI_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================
// 既存 API
// ============================================================

SPOUTDX_FFI_API const char* spoutdx_ffi_version();
SPOUTDX_FFI_API int spoutdx_ffi_get_sdk_version();
SPOUTDX_FFI_API int spoutdx_ffi_test_dx11_init();

// ============================================================
// Receiver API
// ============================================================

// -- 型定義 --

typedef void* SpoutDxReceiverHandle;

typedef enum SpoutDxResult {
    SPOUTDX_OK                   = 0,
    SPOUTDX_ERROR_NULL_HANDLE    = -1,
    SPOUTDX_ERROR_NULL_DEVICE    = -2,
    SPOUTDX_ERROR_NOT_CONNECTED  = -3,
    SPOUTDX_ERROR_INIT_FAILED    = -4,
    SPOUTDX_ERROR_RECEIVE_FAILED = -5,
    SPOUTDX_ERROR_INTERNAL       = -99
} SpoutDxResult;

typedef struct SpoutDxSenderInfo {
    char name[256];
    unsigned int width;
    unsigned int height;
    unsigned int format;  // DXGI_FORMAT
} SpoutDxSenderInfo;

// -- ライフサイクル --

SPOUTDX_FFI_API SpoutDxReceiverHandle spoutdx_receiver_create(void);
SPOUTDX_FFI_API int spoutdx_receiver_destroy(SpoutDxReceiverHandle handle);

// -- DirectX 初期化 --

SPOUTDX_FFI_API int spoutdx_receiver_open_dx11(
    SpoutDxReceiverHandle handle,
    void* device  // ID3D11Device*
);
SPOUTDX_FFI_API int spoutdx_receiver_close_dx11(SpoutDxReceiverHandle handle);

// -- 受信設定 --

SPOUTDX_FFI_API int spoutdx_receiver_set_sender_name(
    SpoutDxReceiverHandle handle,
    const char* sender_name  // NULL で active sender
);

// -- 受信 --

SPOUTDX_FFI_API int spoutdx_receiver_receive_texture(
    SpoutDxReceiverHandle handle,
    void* dst_texture  // ID3D11Texture2D*
);

SPOUTDX_FFI_API int spoutdx_receiver_release(SpoutDxReceiverHandle handle);

// -- 状態取得 --

SPOUTDX_FFI_API int spoutdx_receiver_get_sender_info(
    SpoutDxReceiverHandle handle,
    SpoutDxSenderInfo* out_info
);

SPOUTDX_FFI_API int spoutdx_receiver_is_updated(SpoutDxReceiverHandle handle);
SPOUTDX_FFI_API int spoutdx_receiver_is_connected(SpoutDxReceiverHandle handle);
SPOUTDX_FFI_API int spoutdx_receiver_is_frame_new(SpoutDxReceiverHandle handle);

#ifdef __cplusplus
}
#endif
```

---

## Rust 利用例（参考）

以下は、Rust から利用する際の典型的なコード例です。
※ 本リポジトリには含めず、利用側で別途管理します。

```rust
use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
use windows::Win32::Graphics::Direct3D11::*;

// FFI 型定義
type SpoutDxReceiverHandle = *mut c_void;

#[repr(C)]
pub struct SpoutDxSenderInfo {
    pub name: [c_char; 256],
    pub width: c_uint,
    pub height: c_uint,
    pub format: c_uint,
}

// FFI 関数宣言
unsafe extern "C" {
    fn spoutdx_receiver_create() -> SpoutDxReceiverHandle;
    fn spoutdx_receiver_destroy(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_open_dx11(handle: SpoutDxReceiverHandle, device: *mut c_void) -> c_int;
    fn spoutdx_receiver_receive_texture(handle: SpoutDxReceiverHandle, dst: *mut c_void) -> c_int;
    fn spoutdx_receiver_get_sender_info(handle: SpoutDxReceiverHandle, info: *mut SpoutDxSenderInfo) -> c_int;
    fn spoutdx_receiver_is_updated(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_is_connected(handle: SpoutDxReceiverHandle) -> c_int;
}

// 使用例
fn example(device: &ID3D11Device, texture: &ID3D11Texture2D) {
    unsafe {
        let rx = spoutdx_receiver_create();
        
        // デバイスのポインタを取得して渡す
        // windows crate では .as_raw() で生ポインタ取得可能
        let device_ptr = device.as_raw() as *mut c_void;
        spoutdx_receiver_open_dx11(rx, device_ptr);
        
        // 受信
        let texture_ptr = texture.as_raw() as *mut c_void;
        let result = spoutdx_receiver_receive_texture(rx, texture_ptr);
        
        if result == 0 {
            // 成功
            let mut info = std::mem::zeroed::<SpoutDxSenderInfo>();
            spoutdx_receiver_get_sender_info(rx, &mut info);
        }
        
        spoutdx_receiver_destroy(rx);
    }
}
```

---

## テスト方針

### 単体テスト（C++ 側）

Phase 1 では最小限の手動テストを行います:

1. **ライフサイクルテスト**: create/destroy が正常動作
2. **NULL ハンドルテスト**: 各関数が適切なエラーコードを返す
3. **デバイス初期化テスト**: 外部デバイスで OpenDirectX11 が成功

### 結合テスト（Rust 側）

examples/src/main.rs を拡張:

1. D3D11 デバイスを作成
2. Receiver を作成・初期化
3. テクスチャを作成
4. 受信を試行（センダーがなくても NOT_CONNECTED が返れば OK）
5. クリーンアップ

---

## 実装手順（SubAgent 向け）

### Step 1: ヘッダファイル更新

**ファイル**: `include/spoutdx_ffi/spoutdx_ffi.h`

1. `extern "C"` を `#ifdef __cplusplus` でガード
2. 型定義（`SpoutDxReceiverHandle`, `SpoutDxResult`, `SpoutDxSenderInfo`）を追加
3. 各関数プロトタイプを追加

### Step 2: 実装ファイル更新

**ファイル**: `src/spoutdx_ffi.cpp`

1. 内部クラス `SpoutDxReceiver` を定義
2. 各 API 関数を実装（上記の実装例に従う）
3. 全関数で `try/catch (...)` を使用し、例外が漏れないようにする

### Step 3: サンプル更新

**ファイル**: `examples/src/main.rs`

1. windows crate で D3D11 デバイスを作成
2. 新しい FFI 関数を呼び出す簡単なテスト追加

### Step 4: ビルド確認

```powershell
.\dev.ps1
```

---

## 注意事項

### SpoutDX の ReceiveTexture 使用法

SpoutDX には2種類の `ReceiveTexture` があります:

1. `ReceiveTexture()` - 内部クラステクスチャに受信
2. `ReceiveTexture(ID3D11Texture2D** ppTexture)` - 指定テクスチャに受信

本実装では **2番** を使用します。ただし、このAPIは:

- 呼び出し側がテクスチャを事前に作成する必要がある
- `IsUpdated()` が true の場合、テクスチャの再作成が必要
- テクスチャのフォーマットはセンダーと一致させる必要がある

### テクスチャ作成の責務

呼び出し側（Rust）でテクスチャを作成する必要があります:

1. 最初は適当なサイズで作成（例: 1920x1080, BGRA）
2. `receive_texture` を呼ぶ
3. `is_updated` が true なら、`get_sender_info` でサイズを取得
4. テクスチャを再作成して再度 `receive_texture`

### エラーハンドリング

- 全関数で `try/catch` を使用
- 例外発生時は `SPOUTDX_ERROR_INTERNAL` を返す
- NULL チェックは最初に行う

---

## 更新履歴

- 2026-01-08: 初版作成
