# TESTING_STRATEGY

## 概要

spoutdx-ffi は現在、自動テスト（ユニットテスト/CI）は未整備で、動作確認用の実行ファイルで疎通を確認しています。

## 現状のテスト形態

### 手動テスト: examples/main.rs（Rust FFI）

- **場所**: [examples/src/main.rs](../examples/src/main.rs)
- **役割**: C ABI 関数の疎通確認と Receiver API の動作検証
- **実行方法**:
  ```powershell
  .\dev.ps1              # Debug ビルド + 実行
  .\dev.ps1 -Release     # Release ビルド + 実行
  ```

**検証項目**:

#### 1. 基本 API テスト

- **バージョン取得**
  - `spoutdx_ffi_version()` が正しい文字列を返すか
  - `spoutdx_ffi_get_sdk_version()` が正の値を返すか

- **DirectX 11 初期化**
  - `spoutdx_ffi_test_dx11_init()` が成功するか（戻り値 1）

#### 2. Receiver API テスト

- **ライフサイクル**
  - `spoutdx_receiver_create()` が非 NULL ハンドルを返すか
  - `spoutdx_receiver_destroy()` がクラッシュせずに完了するか

- **DirectX 11 初期化**
  - 外部で作成した `ID3D11Device*` で初期化できるか（`spoutdx_receiver_open_dx11`）
  - 初期化後にデバイスがアクセス可能か

- **受信機能**
  - `spoutdx_receiver_receive()` がセンダー未接続時に適切なエラーを返すか（`SPOUTDX_ERROR_NOT_CONNECTED`）
  - センダー接続時にテクスチャを正常に受信できるか
  - `spoutdx_receiver_receive_texture()` が指定テクスチャへ正しく書き込むか

- **状態取得**
  - `spoutdx_receiver_is_connected()` がセンダーの接続状態を正しく返すか
  - `spoutdx_receiver_is_updated()` がサイズ/フォーマット変更を検出するか
  - `spoutdx_receiver_is_frame_new()` が新規フレームを検出するか
  - `spoutdx_receiver_get_sender_info()` が正しい情報（名前・サイズ・フォーマット）を返すか

- **画像診断**（examples/main.rs 独自機能）
  - 受信したテクスチャの平均色計算
  - 全黒/全白/アルファ異常の検出
  - サンプルピクセルの表示
  - PNG 出力による目視確認

**期待結果**:
```
spoutdx-ffi version: spoutdx-ffi/0.1.0
Spout SDK version: 2007
DirectX 11 initialization: OK
Receiver created successfully
D3D11 device created successfully
Receiver opened with external device successfully
[Receiver Test 1: No Sender]
  No sender connected (expected)
[Sender Availability Check]
  Waiting for sender...
```

**検証の制約**:
- センダーがない環境では接続テストは未接続エラーを返すのみ
- 完全な受信テストにはアクティブな Spout センダーが必要
- 画像品質の検証は PNG 出力による目視確認に依存

## 手動テスト手順（Windows）

### 前提条件

- Windows 10/11（x64）
- Visual Studio 2022 または Build Tools（MSVC）
- Windows SDK
- Rust（examples 実行用）

### 手順

1. **C++ シム DLL をビルド**:
   ```powershell
   cmake --preset msvc-debug
   cmake --build --preset msvc-debug
   ```

2. **Rust example を実行**:
   ```powershell
   cd examples
   cargo run
   ```

3. **期待結果を確認**:
   - バージョン文字列が表示される
   - Spout SDK version が正の値（例: 2007）
   - DirectX 11 initialization が OK
   - Receiver が正常に作成される
   - センダーがない場合は "No sender connected" が表示される

### 完全テスト（センダーあり）

1. **Spout センダーを起動** （例: SpoutCam, Resolume, TouchDesigner）

2. **examples を実行**:
   ```powershell
   cd examples
   cargo run
   ```

3. **期待結果**:
   - センダー名が表示される
   - テクスチャサイズ・フォーマットが表示される
   - 画像診断結果（平均色、アルファ値）が表示される
   - PNG ファイルが出力される（`spout_received_*.png`）

4. **PNG を目視確認**:
   - 画像が正しく受信されているか
   - 色が正しいか（RGB/BGR 入れ替わりがないか）
   - アルファチャンネルが正しいか

## 将来の自動化候補（未実装）

### 1. ユニットテスト（C++ / C ABI 層）

**目的**: C ABI 関数の戻り値契約を検証

**候補フレームワーク**:
- Catch2
- GoogleTest

**テスト項目例**:
```cpp
TEST_CASE("spoutdx_receiver_create returns non-null handle") {
    auto handle = spoutdx_receiver_create();
    REQUIRE(handle != nullptr);
    spoutdx_receiver_destroy(handle);
}

TEST_CASE("spoutdx_receiver_destroy with null handle returns error") {
    auto result = spoutdx_receiver_destroy(nullptr);
    REQUIRE(result == SPOUTDX_ERROR_NULL_HANDLE);
}
```

**課題**:
- DirectX 11 実機依存（CI 環境でのテストが困難）
- モック/スタブの作成が必要

### 2. 統合テスト（Rust FFI 層）

**目的**: Rust から FFI 経由で呼び出す際の安全性検証

**候補フレームワーク**:
- Rust の `#[test]` + `cargo test`

**テスト項目例**:
```rust
#[test]
fn test_receiver_create_destroy() {
    unsafe {
        let handle = spoutdx_receiver_create();
        assert!(!handle.is_null());
        let result = spoutdx_receiver_destroy(handle);
        assert_eq!(result, SPOUTDX_OK);
    }
}

#[test]
fn test_error_handling() {
    unsafe {
        let result = spoutdx_receiver_destroy(std::ptr::null_mut());
        assert_eq!(result, SPOUTDX_ERROR_NULL_HANDLE);
    }
}
```

### 3. CI/CD パイプライン

**目的**: ビルドの再現性チェックと自動テスト実行

**候補**:
- GitHub Actions
- Azure Pipelines

**課題**:
- DirectX 11 実機依存（Windows + GPU が必要）
- クラウド CI 環境での GPU アクセス制限

**暫定案**:
- ビルド確認のみ自動化（Debug/Release）
- DirectX 依存テストは手動実行

**設定例（GitHub Actions）**:
```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - name: Configure CMake
        run: cmake --preset msvc-release
      - name: Build
        run: cmake --build --preset msvc-release
```

### 4. パフォーマンステスト

**目的**: テクスチャコピーのレイテンシ測定

**テスト項目**:
- 受信フレームレート（FPS）
- テクスチャコピー時間
- メモリ使用量

**実装案**:
- examples/main.rs に計測機能を追加
- 1000 フレーム受信時の平均・最大・最小レイテンシを記録

## テストカバレッジ目標（将来）

### Phase 1: 基本テスト自動化

- [ ] C ABI 関数のエラーハンドリング検証
- [ ] ライフサイクル管理（create/destroy）の安全性検証
- [ ] NULL ハンドル/NULL ポインタのエラー処理検証

### Phase 2: 統合テスト

- [ ] DirectX 11 初期化の成功/失敗テスト
- [ ] センダー未接続時のエラーハンドリング
- [ ] Receiver API の状態遷移テスト

### Phase 3: CI/CD 整備

- [ ] ビルド自動化（Debug/Release）
- [ ] 静的解析（clang-tidy, cppcheck）
- [ ] コードフォーマットチェック

## Assumptions（前提）

- 現時点では DirectX 11 の実機依存があるため、CI 実行環境の確保が必要
- 完全な受信テストはアクティブな Spout センダーが必要
- 画像品質の自動検証は困難（目視確認に依存）

## 参考

- 手動テストコード: [examples/src/main.rs](../examples/src/main.rs)
- ビルドスクリプト: [dev.ps1](../dev.ps1)
- API 定義: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h)

