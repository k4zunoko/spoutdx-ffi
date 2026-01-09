# DESIGN_PHILOSOPHY

## 目的

spoutdx-ffi は、Windows の Spout (DirectX 11) を Rust から扱うための **純 C ABI シム** を提供します。

**背景と問題意識**:
- Spout の公式 DLL は C++ ABI を露出しており、`std::string` 等の STL 型がバイナリ境界を跨ぐため、MSVC バージョン・ランタイム設定の不一致により ABI 互換性問題が発生します。
- Rust から安全に FFI 呼び出しを行うには、C ABI のみを境界として確立する必要があります。

## 設計原則

### 1. C ABI のみに露出

- Rust 側から呼び出す公開 API は C 互換（`extern "C"`）に限定します。
- C++ の例外・STL 型（`std::string`, `std::vector` 等）を ABI 境界に出しません。
- 不透明ハンドル（`void*`）を使用し、内部 C++ クラスへのポインタを隠蔽します。

**根拠**: C ABI は標準化されており、コンパイラ・ランタイムバージョンに依存しません。

**実装例**:
- ヘッダ: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h)
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp)

### 2. Spout をソース統合して ABI 問題を回避

- 事前ビルド DLL の C++ ABI（`std::string` など）互換性問題を避けるため、Spout をソースからビルドしライブラリに静的に統合します。
- その結果、利用側（Rust）と Spout の間には **C ABI のみ** が存在します。

**根拠**: ソース統合により、Spout の内部実装を完全に制御し、同一コンパイラ設定でビルドできます。

**ビルド設定**: [CMakeLists.txt](../CMakeLists.txt) の `SPOUT_SOURCES` 参照

### 3. 最小 API から段階的に拡張

- 現状は疎通確認（SDK バージョン取得、DX11 初期化）から開始し、Receiver API を段階的に追加しました。
- 将来的には Sender API、センダー一覧取得、フレーム同期機能などを追加予定です。

**根拠**: 段階的な実装により、各 API の動作を確実に検証し、安定性を確保します。

### 4. 例外安全性の確保

- C ABI 境界を例外が越えないよう、すべての公開関数で `try/catch (...)` を使用します。
- エラーは数値コード（`SpoutDxResult` enum）で返します。

**根拠**: C++ 例外が C ABI 境界を越えると未定義動作となるため、明示的なエラーコードによる安全な伝播が必要です。

**実装例**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp) のすべての公開関数

## スコープ

### 現在のスコープ（実装済み）

- **基本 API**
  - Spout SDK バージョン取得
  - DirectX 11 初期化疎通テスト

- **Receiver API（受信機能）** - 2026-01-08 実装完了
  - ライフサイクル管理（`spoutdx_receiver_create` / `spoutdx_receiver_destroy`）
  - 外部デバイスでの初期化（`spoutdx_receiver_open_dx11`）
  - テクスチャ受信（`spoutdx_receiver_receive_texture` / `spoutdx_receiver_receive`）
  - センダー情報取得（名前・サイズ・フォーマット）
  - 接続状態/更新/新規フレーム判定

### スコープ外（現時点）

- Rust の bindings/wrapper クレートの提供（必要なら利用側で別管理）
- DX12/DX9 など DX11 以外の統合
- OpenGL 経路のサポート
- Sender API（将来実装予定）
- センダー一覧取得（将来実装予定）
- フレーム同期機能（将来実装予定）

## API 設計の方針

### Receiver API

**既存パイプライン統合を優先**:
- 呼び出し側が D3D11 デバイスと出力先テクスチャを用意し、そこへ受信結果を書き込む `receive_texture` 形式を主とします。
- これにより、既存のレンダリングパイプラインへの統合が容易になります。

**根拠**: 既存のゲームエンジンや映像ツールでは、独自の DX11 デバイスとテクスチャ管理を持つため、外部から提供されたテクスチャへの書き込みが最も柔軟です。

**「内部で受信して返す」は補助**:
- テストやツール用途の補助 API として、内部で受信した結果を返す `receive` 形式も提供します。
- 内部テクスチャは `spoutdx_receiver_get_received_texture` で取得できます。

**根拠**: 簡易ツールやテストコードでは、テクスチャ管理を簡略化できる方が便利です。

### ハンドルベース設計

- Receiver は不透明ハンドル（`SpoutDxReceiverHandle = void*`）として公開します。
- 内部では `SpoutDxReceiver` クラスを使用し、ライフサイクルを管理します。

**根拠**: 不透明ハンドルにより、C++ クラスの実装詳細を隠蔽し、C ABI 境界を維持します。

### エラーハンドリング

- すべての関数は `SpoutDxResult` enum（または状態を示す `int`）を返します。
- 成功は `SPOUTDX_OK (0)`、失敗は負値で、エラー種別を識別できます。

**根拠**: 数値エラーコードは C ABI で安全に伝播でき、呼び出し側でエラー処理ロジックを実装できます。

## 変更許容性のガイドライン

### 保護すべきもの（変更禁止）

- **C ABI 境界**: 公開関数は必ず `extern "C"` とし、C++ 型を露出しない
- **例外安全性**: 公開関数は必ず `try/catch (...)` で例外を捕捉
- **エラーコード体系**: `SpoutDxResult` の既存値は変更せず、追加のみ

### 変更可能なもの

- **内部実装**: C++ クラスの構造や実装詳細
- **新規 API の追加**: C ABI を守る限り、新しい関数を追加可能
- **エラーコードの追加**: 既存値を変更せず、新しいエラーコードを追加可能

## 参照

- 現在の公開 API: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h)
- ビルド統合（Spout ソース追加）: [CMakeLists.txt](../CMakeLists.txt)
- 詳細な実装計画: [impl/RECEIVER_IMPLEMENTATION_PLAN.md](impl/RECEIVER_IMPLEMENTATION_PLAN.md)

