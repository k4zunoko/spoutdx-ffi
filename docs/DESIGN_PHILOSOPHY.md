# DESIGN_PHILOSOPHY

## 目的

spoutdx-ffi は、Windows の Spout (DirectX 11) を Rust から扱うための **純 C ABI シム** を提供します。

## 設計原則

- **C ABI のみに露出**
  - Rust 側から呼び出す公開 API は C 互換（`extern "C"`）に限定します。
  - C++ の例外・STL 型（`std::string` 等）を ABI 境界に出しません。

- **Spout をソース統合して ABI 問題を回避**
  - 事前ビルド DLL の C++ ABI（`std::string` など）互換性問題を避けるため、Spout をソースからビルドしライブラリに静的に統合します。
  - その結果、利用側（Rust）と Spout の間には **C ABI のみ** が存在します。

- **最小 API から段階的に拡張**
  - 現状は疎通確認（SDK バージョン取得、DX11 初期化）から開始し、Receiver API を段階的に追加します。

## スコープ外（現時点）

- Rust の bindings/wrapper クレートの提供（必要なら利用側で別管理）
- DX12/DX9 など DX11 以外の統合
- OpenGL 経路のサポート

## API 設計の方針（Receiver）

- **既存パイプライン統合を優先**
  - 呼び出し側が D3D11 デバイスと出力先テクスチャを用意し、そこへ受信結果を書き込む `receive_into` 形式を主とします。

- **「内部で受信して返す」は補助**
  - テストやツール用途の補助 API として、内部で受信した結果を返す形式を将来的に追加する可能性があります。

## 参照

- 現在の公開 API: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h)
- ビルド統合（Spout ソース追加）: [CMakeLists.txt](../CMakeLists.txt)
