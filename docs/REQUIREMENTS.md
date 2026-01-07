# REQUIREMENTS

## 対象環境

- OS: Windows
- ビルド: CMake + MSVC（Visual Studio 2022 / Build Tools）
- 依存: Windows SDK（D3D11/DXGI 等）

## 機能要件（現状）

- Spout SDK バージョンを取得できること
- DirectX 11 の初期化疎通が取れること

根拠（実装）:
- [spoutdx_ffi_get_sdk_version](../src/spoutdx_ffi.cpp)
- [spoutdx_ffi_test_dx11_init](../src/spoutdx_ffi.cpp)

## 非機能要件

- **ABI 安定性**: Rust から安全に呼べる C ABI を維持する
- **例外安全性**: C ABI 境界を例外が越えない
- **ビルド再現性**: `CMakePresets.json` のプリセットでビルド手順が固定される

## 提供形態（重要）

- このプロジェクトは **shim DLL（C ABI）提供に限定**します。
- Rust の bindings/wrapper クレートは含めません（必要なら利用側で別途管理）。

## 依存関係

- Windows ライブラリ（リンク）
  - `d3d11`, `dxgi`, `psapi`, `version`, `winmm`
  - 定義場所: [CMakeLists.txt](../CMakeLists.txt)

- Spout ソース
  - 配置: [third_party/Spout2/SPOUTSDK/](../third_party/Spout2/SPOUTSDK/)
  - ビルド対象ファイルは [CMakeLists.txt](../CMakeLists.txt) の `SPOUT_SOURCES` 参照

## 未実装（ロードマップ）

Receiver としての本命 API（受信・メタデータ取得・ライフサイクル）は未実装です。
計画は [ROADMAP.md](ROADMAP.md) に集約します。

## 受信 API の優先順位

- 既存パイプライン統合を優先し、呼び出し側が D3D11 デバイス/出力先テクスチャを用意する `receive_into` 形式を主とします。
- 内部で受信して返す形式は、テスト/ツール用途として補助的に提供する想定です。

## Assumptions

- Spout の必要最小ソースはリポジトリに含まれており、追加ダウンロード不要。
- 主要ターゲットは x64。
