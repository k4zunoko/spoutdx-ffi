# ROADMAP

## 実装済み（2026-01-08）

- ✅ Receiver API の追加（C ABI）
  - `*_create` / `*_destroy` のライフサイクル
  - テクスチャ受信（DX11, `receive_into` 形式）
  - センダー情報取得（名前・サイズ・フォーマット）
  - 状態取得（is_updated, is_connected, is_frame_new）
  - 外部デバイスでの初期化（open_dx11）

- ✅ エラーハンドリング体系の確立
  - エラーコードの固定化（SpoutDxResult enum）
  - try/catch による例外安全性確保

- ✅ Rust 側の利用例の確立
  - windows crate を使用した D3D11 デバイス作成例
  - FFI 経由での Receiver API 利用例

  詳細: [docs/impl/RECEIVER_IMPLEMENTATION_PLAN.md](impl/RECEIVER_IMPLEMENTATION_PLAN.md)

## 近い将来（実装予定）

- センダー一覧取得 API
- フレーム同期機能（SetFrameSync, WaitFrameSync）
- 内部デバイス生成モードのサポート

## 中期

- Release ビルドの配布形態（成果物/パッケージング）整理
- CI（ビルドの再現性チェック）
- Sender API の追加

## 方向性メモ

- 既存パイプライン統合を優先するため、呼び出し側が用意した出力先テクスチャへ書き込む `receive_into` を主 API とする
- 「内部で受信して返す」形式はテスト/ツール用途として補助的に提供する想定

## Notes

このファイルは「実装追加のたびに更新する計画表」です。
実装されていない仕様を断定しないため、詳細な API 仕様は実装着手時に別ドキュメントへ分離します。
