# spoutdx-ffi

Rust から Spout（DirectX 11）を扱うための **純 C ABI シム DLL**です。
Spout2（SPOUTSDK）の必要最小ソースをビルドに統合し、公開 API を C ABI のみに固定することで **DLL/C++ ABI 互換性問題（例: `std::string` など）を回避**します。

このリポジトリの成果物は「C ABI DLL（`spoutdx_ffi.dll`）」です。Rust 側の wrapper/bindings クレートは同梱しません（利用側で管理してください）。

## できること（現状）

- Spout SDK version の取得
- DX11 初期化疎通（デバイス作成テスト）
- Receiver（受信）API（C ABI）
	- 外部で作成した `ID3D11Device*` を渡して初期化
	- `ID3D11Texture2D*`（呼び出し側が用意）へ受信結果を書き込み（`receive_into` 形式）
	- 接続状態/更新/新規フレームの取得
	- センダー情報（名前/サイズ/フォーマット）の取得

## 前提環境（Windows）

- Windows 10/11（x64）
- Visual Studio 2022 または Build Tools（MSVC）
- Windows SDK（D3D11/DXGI 等）
- CMake 3.25+
- Rust（examples 実行用、任意）

## ビルド（Windows）

### クイックスタート（推奨）

```powershell
# DLL ビルド + example 実行 を一度に行う
.\dev.ps1

# Release ビルド版で実行
.\dev.ps1 -Release

# DLL のみ再ビルド（example は実行しない）
.\dev.ps1 -NoExample

# Rust example のみ実行（DLL 再ビルドをスキップ）
.\dev.ps1 -NoRebuild
```

- **Windows**: `dev.ps1` (PowerShell)

| オプション | 説明 |
|-----------|------|
| `-Release` | Release ビルド（デフォルト: Debug） |
| `-NoExample` | example 実行をスキップ（DLL のみビルド） |
| `-NoRebuild` | DLL ビルドをスキップ（example のみ実行） |

補足:

- `dev.proxy.ps1` が存在する場合、`dev.ps1` 実行時に読み込まれます（プロキシ環境向けの任意設定）。

### 手動ビルド

```powershell
# 1. C ABI シム DLL をビルド（Debug）
cmake --preset msvc-debug
cmake --build --preset msvc-debug

# Release の場合
cmake --preset msvc-release
cmake --build --preset msvc-release

# 2. Rust example を実行
cd examples
cargo run

# Release の場合
cargo run --release
```

`examples/build.rs` は、ビルド済み DLL を `target\debug` / `target\release` にコピーして実行時に見つかるようにします。

### 成果物の場所

- DLL:
	- Debug: `out\build\msvc-debug\Debug\spoutdx_ffi.dll`
	- Release: `out\build\msvc-release\Release\spoutdx_ffi.dll`
- Rust example バイナリ:
	- Debug: `examples\target\debug\ping.exe`
	- Release: `examples\target\release\ping.exe`

## 公開 C ABI

公開ヘッダ: `include/spoutdx_ffi/spoutdx_ffi.h`

### 既存 API

- `const char* spoutdx_ffi_version(void);`
	- バージョン文字列を返します（静的文字列）。
- `int spoutdx_ffi_get_sdk_version(void);`
	- Spout SDK version を返します（例: `2007` は "2.007" 相当）。
- `int spoutdx_ffi_test_dx11_init(void);`
	- DX11 初期化疎通（成功=1、失敗=0）。

### Receiver API（受信）

型:

- `typedef void* SpoutDxReceiverHandle;`
- `typedef enum SpoutDxResult { ... } SpoutDxResult;`
- `typedef struct SpoutDxSenderInfo { char name[256]; unsigned int width; unsigned int height; unsigned int format; } SpoutDxSenderInfo;`

ライフサイクル:

- `SpoutDxReceiverHandle spoutdx_receiver_create(void);`
- `int spoutdx_receiver_destroy(SpoutDxReceiverHandle handle);`

DX11 初期化（呼び出し側デバイスを使用）:

- `int spoutdx_receiver_open_dx11(SpoutDxReceiverHandle handle, void* device /* ID3D11Device* */);`
- `int spoutdx_receiver_close_dx11(SpoutDxReceiverHandle handle);`

受信設定:

- `int spoutdx_receiver_set_sender_name(SpoutDxReceiverHandle handle, const char* sender_name);`
	- `sender_name == NULL` の場合は active sender。

受信（テクスチャへ書き込み）:

- `int spoutdx_receiver_receive_texture(SpoutDxReceiverHandle handle, void* dst_texture /* ID3D11Texture2D* */);`
- `int spoutdx_receiver_release(SpoutDxReceiverHandle handle);`

状態取得:

- `int spoutdx_receiver_get_sender_info(SpoutDxReceiverHandle handle, SpoutDxSenderInfo* out_info);`
- `int spoutdx_receiver_is_updated(SpoutDxReceiverHandle handle);`
- `int spoutdx_receiver_is_connected(SpoutDxReceiverHandle handle);`
- `int spoutdx_receiver_is_frame_new(SpoutDxReceiverHandle handle);`

### Receiver の戻り値（SpoutDxResult）

Receiver API の戻り値は `0` が成功、負の値が失敗です。

- `SPOUTDX_OK (0)`
- `SPOUTDX_ERROR_NULL_HANDLE (-1)`
- `SPOUTDX_ERROR_NULL_DEVICE (-2)`
- `SPOUTDX_ERROR_NOT_CONNECTED (-3)`（センダーが見つからない/接続できない）
- `SPOUTDX_ERROR_INIT_FAILED (-4)`
- `SPOUTDX_ERROR_RECEIVE_FAILED (-5)`
- `SPOUTDX_ERROR_INTERNAL (-99)`（例外など内部エラー）

## 利用メモ（重要）

- Receiver API は **呼び出し側が D3D11 デバイスと受信先テクスチャを用意**する前提です。
- `spoutdx_receiver_receive_texture` が `SPOUTDX_ERROR_NOT_CONNECTED` を返すのは正常です（Spout sender が起動していない場合）。
- `dst_texture` は `ID3D11Texture2D*` を渡してください（NULL は不可）。
- `SpoutDxSenderInfo.format` は `DXGI_FORMAT` の数値です。

## トラブルシュート

- CMake が失敗する: Visual Studio 2022 / Build Tools と Windows SDK が入っているか確認してください（`Visual Studio 17 2022` ジェネレータを使用します）。
- `Spout source not found...` と出る: `third_party/Spout2/SPOUTSDK/` の配置が必要です。
- 実行時に DLL が見つからない: `spoutdx_ffi.dll` が `ping.exe` と同じフォルダにあるか確認してください（example は `build.rs` がコピーします）。
- Release 実行で DLL パスが合わない: 既定では Cargo の `PROFILE` に合わせて Debug/Release を切り替えます。必要なら環境変数 `SPOUTDX_FFI_CMAKE_PRESET=msvc-debug|msvc-release` で上書きできます。
