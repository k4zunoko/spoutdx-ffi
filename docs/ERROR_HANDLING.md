# ERROR_HANDLING

## 概要

spoutdx-ffi では、すべてのエラーを **数値エラーコード（`SpoutDxResult` enum）** で返します。C++ 例外は C ABI 境界を越えないよう、すべての公開関数で `try/catch (...)` により捕捉されます。

## エラーコード体系

### SpoutDxResult enum

```c
typedef enum SpoutDxResult {
    SPOUTDX_OK                   = 0,   // 成功
    SPOUTDX_ERROR_NULL_HANDLE    = -1,  // ハンドルが NULL
    SPOUTDX_ERROR_NULL_DEVICE    = -2,  // デバイスポインタが NULL
    SPOUTDX_ERROR_NOT_CONNECTED  = -3,  // センダーに未接続
    SPOUTDX_ERROR_INIT_FAILED    = -4,  // 初期化失敗
    SPOUTDX_ERROR_RECEIVE_FAILED = -5,  // 受信失敗
    SPOUTDX_ERROR_INTERNAL       = -99  // 内部エラー（例外捕捉）
} SpoutDxResult;
```

**定義場所**: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h#L23-L31)

### エラーコードの詳細と対処方法

#### `SPOUTDX_OK (0)`

- **意味**: 操作が成功
- **対処**: なし（正常終了）

#### `SPOUTDX_ERROR_NULL_HANDLE (-1)`

- **意味**: 無効なハンドルが渡された
- **原因**:
  - `spoutdx_receiver_create()` が失敗して NULL を返したハンドルを使用
  - 既に破棄されたハンドルを使用
- **対処**:
  - ハンドル作成時に NULL チェックを行う
  - ハンドル破棄後は使用しない

**例**:
```rust
let handle = unsafe { spoutdx_receiver_create() };
if handle.is_null() {
    eprintln!("Failed to create receiver");
    return;
}
```

#### `SPOUTDX_ERROR_NULL_DEVICE (-2)`

- **意味**: NULL デバイスポインタが渡された
- **原因**: `spoutdx_receiver_open_dx11()` に NULL デバイスを渡した
- **対処**: 有効な `ID3D11Device*` を渡す

**例**:
```rust
let device: *mut c_void = /* D3D11 デバイス作成 */;
if device.is_null() {
    eprintln!("Failed to create D3D11 device");
    return;
}
let result = unsafe { spoutdx_receiver_open_dx11(handle, device) };
```

#### `SPOUTDX_ERROR_NOT_CONNECTED (-3)`

- **意味**: センダーに接続していない
- **原因**:
  - アクティブなセンダーが存在しない
  - 指定した名前のセンダーが見つからない
  - センダーが切断された
- **対処**:
  - `spoutdx_receiver_is_connected()` で接続状態を確認
  - センダーが起動しているか確認
  - 別のセンダー名を試す

**例**:
```rust
let result = unsafe { spoutdx_receiver_receive(handle) };
if result == SPOUTDX_ERROR_NOT_CONNECTED {
    println!("No sender connected. Waiting...");
    // リトライロジック
}
```

#### `SPOUTDX_ERROR_INIT_FAILED (-4)`

- **意味**: DirectX 11 初期化に失敗
- **原因**:
  - 無効なデバイスポインタ
  - DirectX 11 がサポートされていない環境
  - GPU ドライバの問題
- **対処**:
  - デバイスが正しく作成されているか確認
  - GPU ドライバを更新
  - DirectX 11 互換性を確認

#### `SPOUTDX_ERROR_RECEIVE_FAILED (-5)`

- **意味**: テクスチャ受信に失敗
- **原因**:
  - センダーとのフォーマット不一致
  - テクスチャサイズの不一致
  - GPU メモリ不足
  - 共有テクスチャアクセス失敗
- **対処**:
  - `spoutdx_receiver_is_updated()` で変更を検出し、テクスチャを再作成
  - `spoutdx_receiver_get_sender_info()` で正しいサイズ・フォーマットを取得

**例**:
```rust
if unsafe { spoutdx_receiver_is_updated(handle) } != 0 {
    // センダー情報を再取得してテクスチャを再作成
    let mut info = SpoutDxSenderInfo { /* ... */ };
    unsafe { spoutdx_receiver_get_sender_info(handle, &mut info) };
    // テクスチャ再作成ロジック
}
```

#### `SPOUTDX_ERROR_INTERNAL (-99)`

- **意味**: 内部エラー（予期しない例外）
- **原因**:
  - C++ 例外が発生した
  - メモリ不足
  - システムリソースの枯渇
- **対処**:
  - ログを確認
  - システムリソースを確認
  - バグレポートを提出

## 例外安全性の実装

すべての公開 C ABI 関数は、以下のパターンで例外を捕捉します:

```cpp
int spoutdx_receiver_some_function(SpoutDxReceiverHandle handle) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        // 実装
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}
```

**根拠**: C++ 例外が C ABI 境界を越えると未定義動作となるため、必ず捕捉してエラーコードに変換。

**実装**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp) のすべての公開関数

## エラーハンドリングのベストプラクティス

### 1. 常にエラーコードをチェックする

```rust
let result = unsafe { spoutdx_receiver_open_dx11(handle, device) };
if result != SPOUTDX_OK {
    eprintln!("Failed to open DX11: {}", result);
    return;
}
```

### 2. センダー状態を定期的に確認する

```rust
loop {
    // 接続状態確認
    if unsafe { spoutdx_receiver_is_connected(handle) } == 0 {
        println!("Sender disconnected");
        break;
    }
    
    // 変更検出
    if unsafe { spoutdx_receiver_is_updated(handle) } != 0 {
        println!("Sender updated (size/format changed)");
        // テクスチャ再作成
    }
    
    // 受信
    let result = unsafe { spoutdx_receiver_receive(handle) };
    if result != SPOUTDX_OK {
        eprintln!("Receive failed: {}", result);
    }
}
```

### 3. リソースは必ず解放する

```rust
let handle = unsafe { spoutdx_receiver_create() };
if handle.is_null() {
    return;
}

// 使用後は必ず破棄
unsafe { spoutdx_receiver_destroy(handle) };
```

**根拠**: RAII パターンにより、C++ 側でリソースは自動解放されるが、ハンドル自体は呼び出し側で管理する必要がある。

## 将来の拡張（未実装）

### 詳細エラー情報の取得

将来的に、スレッドローカルストレージを使用した詳細エラーメッセージ取得 API を追加する可能性があります:

```c
// 想定される API（未実装）
SPOUTDX_FFI_API const char* spoutdx_get_last_error_message();
```

**根拠**: エラーコードだけでは不十分な場合に、デバッグ用の詳細情報を提供。

## 参考

- API 定義: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h)
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp)
- 利用ガイド: [USAGE_DLL.md](../USAGE_DLL.md)
- 実装計画: [impl/RECEIVER_IMPLEMENTATION_PLAN.md](impl/RECEIVER_IMPLEMENTATION_PLAN.md)
