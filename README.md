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

## Spoutソースコードの配置

**重要**: このプロジェクトは Spout のソースコードを取り込んでビルドします。

### 1. Spout2 リポジトリを取得

```bash
# GitHub から clone するか、リリースアーカイブをダウンロード
git clone https://github.com/leadedge/Spout2.git
```

### 2. SPOUTSDKフォルダをコピー

以下の場所に `SPOUTSDK` フォルダを配置してください：

```
spoutdx-ffi/
└── third_party/
    └── Spout2/
        └── SPOUTSDK/       ← ここに配置
            ├── SpoutGL/    (SpoutDXが依存する共通ファイル)
            └── SpoutDirectX/
                └── SpoutDX/
```

**必要なファイル**:
- `SPOUTSDK/SpoutGL/SpoutCommon.h`, `SpoutCopy.*`, `SpoutDirectX.*`, `SpoutFrameCount.*`, `SpoutSenderNames.*`, `SpoutSharedMemory.*`, `SpoutUtils.*`
- `SPOUTSDK/SpoutDirectX/SpoutDX/SpoutDX.h`, `SpoutDX.cpp`

※ `.gitignore` に `third_party/Spout2/SPOUTSDK/` を追加済みなので、Git管理されません。

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

⏳ **Spout統合（準備完了）**
- ソースコード配置待ち: `third_party/Spout2/SPOUTSDK/`
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
