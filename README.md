# spoutdx-ffi

Rust から Spout を扱うための **純C ABI シム** です。  
Spoutのソースコードを直接ビルドして統合するため、DLL ABI問題がありません。

## 必要なもの（Windows）

- Visual Studio 2022 もしくは **Build Tools for Visual Studio 2022**
  - 「Desktop development with C++」
  - Windows 10/11 SDK
  - MSVC v143
- CMake (3.25+)
- VS Code（推奨）
  - 拡張: C/C++ (ms-vscode.cpptools), CMake Tools (ms-vscode.cmake-tools)

## Spoutソースコードについて

**重要**: このプロジェクトは Spout のソースコードを取り込んでビルドします。

Spout2 のソースコード（SPOUTSDK）は `third_party/Spout2/SPOUTSDK/` に含まれており、Git管理されています。
追加のセットアップは不要で、そのままビルドできます。

**含まれているファイル**:
- `SPOUTSDK/SpoutGL/`: SpoutDXが依存する共通ファイル
- `SPOUTSDK/SpoutDirectX/SpoutDX/`: SpoutDX本体

## ビルド

VS Code の CMake Tools を使うのが最短です。

- Configure Preset: `msvc-debug`
- Build Preset: `msvc-debug`

ターミナルからなら:

```powershell
cmake --preset msvc-debug
cmake --build --preset msvc-debug
.\out\build\msvc-debug\Debug\spoutdx_ffi_example.exe
```

## 構成

- `spoutdx_ffi` : 純C ABI シムの共有ライブラリ（Spoutソースから直接ビルド）
- `spoutdx_ffi_example` : 動作確認用EXE

## 開発環境の状態

✅ **プロジェクト構成**
- CMake + MSBuild (Visual Studio 2022 BuildTools)
- C++ 20
- 純C ABI インターフェース（Rust FFI ready）

✅ **Spout統合**
- Spout2ソースコードを `third_party/Spout2/SPOUTSDK/` に含む
- ビルド後は DLL ABI 問題なし
- 全機能がネイティブ実装として利用可能

## 今後の実装

Spoutソース統合後、以下のC API を実装予定：

1. **Receiver機能**
   - `spoutdx_ffi_receiver_create()` / `_destroy()`
   - `spoutdx_ffi_receiver_receive_texture()`
   - `spoutdx_ffi_receiver_get_sender_info()`

2. **DirectX統合**
   - `spoutdx_ffi_open_dx11()`
   - `spoutdx_ffi_get_dx11_device()`

3. **エラーハンドリング**
   - Rust FFI 用の安全なエラーコード体系
