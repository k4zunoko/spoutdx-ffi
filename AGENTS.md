# spoutdx-ffi

spoutdx-ffi は、Windows の Spout (DirectX 11) を DLL/C++ ABI 互換性問題を回避しつつ Rust から扱うための **純 C ABI シム**です。Spout2 の必要最小ソースをビルドに統合し、C ABI のみを公開することで DLL/C++ ABI 問題を回避します。

## Docs（索引）

- docs 全体の入口: [docs/README.md](docs/README.md)
- 設計原則・方針: [docs/DESIGN_PHILOSOPHY.md](docs/DESIGN_PHILOSOPHY.md)
- 要求仕様・スコープ: [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md)
- 全体構成・ビルド: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- エラーハンドリング: [docs/ERROR_HANDLING.md](docs/ERROR_HANDLING.md)
- テスト方針: [docs/TESTING_STRATEGY.md](docs/TESTING_STRATEGY.md)
- 今後の計画: [docs/ROADMAP.md](docs/ROADMAP.md)

## ドキュメント管理方針

**重要**: これらのドキュメントは実装状況と常に一致するよう、以下のタイミングで更新してください:

1. **設計判断の変更時**: DESIGN_PHILOSOPHY.md, 該当レイヤのドキュメント
2. **実装追加時**: 該当レイヤのドキュメント
3. **テスト追加時**: TESTING_STRATEGY.md
4. **エラーハンドリング変更時**: ERROR_HANDLING.md

GitHub Copilotは常にこれらのドキュメントを参照してサポートを提供します。

---

## LLM向けメタ情報

### このドキュメント群について

**AGENTS.mdとdocs/配下のドキュメント群は、GitHub CopilotなどのLLM言語モデルがプロジェクト状況を正確に理解するために設計されています。**

### ドキュメント設計の責務

#### AGENTS.md
- **プロジェクトの最低限の概要**（目的、技術スタック、アーキテクチャ）
- **docs/配下のドキュメントへのナビゲーション**（索引・使用ガイド）
- **このメタ情報**（LLMがドキュメント管理を理解するため）

#### docs/配下のドキュメント
- **詳細な設計方針と実装指針**（AGENTS.mdには書かない）
- **設計判断の根拠**（なぜこの設計にしたのか）
- **具体的な実装例とコードスニペット**
- **変更許容性のガイドライン**（何を変更してよいか、何を保護すべきか）
- **レイヤごとの責務と境界**

### ドキュメント品質の原則

1. **具体性**: 抽象的な記述ではなく、コード例や具体的な数値を含める
2. **根拠**: 設計判断には必ず理由を明記（パフォーマンス、保守性、安全性など）
3. **最新性**: 実装と常に一致させる（古い情報は削除または更新）
4. **ナビゲーション**: AGENTS.mdから各ドキュメントへの明確な案内
5. **文脈**: LLMが次回セッションで読んでも理解できる十分な文脈情報

### 注意事項

- **AGENTS.mdに詳細を書かない**: 詳細はdocs/配下に分離
- **重複を避ける**: 同じ情報は1箇所のみに記載し、相互参照を使用
- **実装との一致**: ドキュメントと実装が乖離した場合、必ず同期する
