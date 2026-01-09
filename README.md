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

## 使い方（DLL 利用ガイド）

- DLL を他プロジェクトから利用する際の手順・注意点: [USAGE_DLL.md](USAGE_DLL.md)

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

