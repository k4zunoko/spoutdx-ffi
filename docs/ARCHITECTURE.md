# ARCHITECTURE

## 全体像

このリポジトリは、Spout2 の必要最小ソースをビルドに取り込み、`spoutdx_ffi`（DLL）として **C ABI** を公開します。

- `spoutdx_ffi`（DLL）
  - 役割: Rust から呼ぶための C ABI シム
  - 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp)
  - 公開ヘッダ: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h)

- Spout2 ソース（ビルドに統合）
  - 役割: SpoutDX（DX11）本体
  - 配置: [third_party/Spout2/SPOUTSDK/](../third_party/Spout2/SPOUTSDK/)

- サンプル
  - 役割: ローカル疎通（SDK バージョン/DX11 初期化）
  - [examples/ping.cpp](../examples/ping.cpp)

## ビルド構成

- CMake プリセット
  - 定義: [CMakePresets.json](../CMakePresets.json)
  - `msvc-debug` / `msvc-release`

- 重要な CMake ロジック
  - Spout ソース存在チェック（`SpoutGL/SpoutCommon.h`）
  - `SPOUT_SOURCES` を `spoutdx_ffi` に直接コンパイル投入
  - Windows 系ライブラリをリンク
  - 定義: [CMakeLists.txt](../CMakeLists.txt)

## データフロー（現状）

- Rust（将来）→ C ABI（`spoutdx_ffi`）→ C++ 実装（SpoutDX クラス呼び出し）
- 現状の API は疎通用のため、テクスチャ受信フローはまだ存在しません

## データフロー（将来: Receiver）

優先する統合形態は `receive_into` です。

- 呼び出し側
  - D3D11 デバイス
  - 出力先テクスチャ（受信結果の書き込み先）
- spoutdx-ffi
  - センダーから受信した内容を、指定された出力先テクスチャへコピー/書き込み

補助として、テスト/ツール用途向けに「内部で受信して返す」形式を追加する可能性があります。

## 将来拡張の想定

Receiver API を追加する際は、以下を守ります。

- C ABI ではハンドル（不透明ポインタ）+ エラーコードを基本形にする
- 呼び出し側が所有する DX11 デバイス/コンテキストとの関係（どちらが所有/生成するか）を明確化する

詳細計画は [ROADMAP.md](ROADMAP.md) に記載します。
