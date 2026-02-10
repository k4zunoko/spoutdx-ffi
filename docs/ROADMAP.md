# ROADMAP

## 実装済み（✅ 完了）

### ✅ Receiver API の追加（2026-01-08）

- **ライフサイクル管理**
  - `spoutdx_receiver_create` / `spoutdx_receiver_destroy`
  
- **DirectX 11 初期化**
  - 外部デバイスでの初期化（`spoutdx_receiver_open_dx11`）
  - DirectX クローズ（`spoutdx_receiver_close_dx11`）
  
- **テクスチャ受信**
  - 指定テクスチャへの受信（`spoutdx_receiver_receive_texture`）
  - 内部テクスチャへの受信（`spoutdx_receiver_receive`）
  - 受信テクスチャ取得（`spoutdx_receiver_get_received_texture`）
  - D3D11 コンテキスト取得（`spoutdx_receiver_get_dx11_context`）
  
- **受信設定**
  - センダー名指定（`spoutdx_receiver_set_sender_name`）
  - Receiver リリース（`spoutdx_receiver_release`）
  
- **状態取得**
  - センダー情報取得（`spoutdx_receiver_get_sender_info`）
  - センダー変更検出（`spoutdx_receiver_is_updated`）
  - 接続状態確認（`spoutdx_receiver_is_connected`）
  - 新規フレーム確認（`spoutdx_receiver_is_frame_new`）

**成果**:
- C ABI による安全な FFI 境界
- 既存パイプライン統合（外部デバイス利用）
- 簡易ツール向け内部テクスチャ受信
- 包括的なエラーハンドリング（`SpoutDxResult` enum）
- Rust FFI 利用例の確立

**詳細**: [impl/RECEIVER_IMPLEMENTATION_PLAN.md](impl/RECEIVER_IMPLEMENTATION_PLAN.md)

### ✅ エラーハンドリング体系の確立（2026-01-08）

- **エラーコードの固定化**
  - `SpoutDxResult` enum の定義
  - 各エラーコードの意味と対処方法の文書化
  
- **例外安全性の確保**
  - すべての公開関数で `try/catch (...)` を使用
  - 例外を C ABI 境界で捕捉してエラーコードに変換

**詳細**: [ERROR_HANDLING.md](ERROR_HANDLING.md)

### ✅ Rust 側の利用例の確立（2026-01-08）

- **windows crate を使用した DirectX 11 統合**
  - D3D11 デバイス作成例
  - テクスチャ作成・管理
  - ステージングテクスチャへのコピー
  
- **FFI 経由での Receiver API 利用**
  - ライフサイクル管理の実例
  - エラーハンドリングのパターン
  - 画像診断・PNG 出力

**詳細**: [examples/src/main.rs](../examples/src/main.rs)

## 近い将来（実装予定）

### センダー一覧取得 API

- **目的**: 利用可能な Spout センダーのリストを取得
- **想定 API**:
  ```c
  typedef struct SpoutDxSenderList {
      char** names;
      unsigned int count;
  } SpoutDxSenderList;
  
  SPOUTDX_FFI_API int spoutdx_get_sender_list(SpoutDxSenderList* out_list);
  SPOUTDX_FFI_API void spoutdx_free_sender_list(SpoutDxSenderList* list);
  ```
- **根拠**: センダー選択 UI を実装するために必要
- **課題**: 動的メモリ管理を C ABI で安全に行う必要がある

### フレーム同期機能

- **目的**: センダーとレシーバーのフレーム同期
- **想定 API**:
  ```c
  SPOUTDX_FFI_API int spoutdx_receiver_set_frame_sync(
      SpoutDxReceiverHandle handle,
      int enabled
  );
  SPOUTDX_FFI_API int spoutdx_receiver_wait_frame_sync(
      SpoutDxReceiverHandle handle,
      unsigned int timeout_ms
  );
  ```
- **根拠**: 高精度な映像同期が必要なアプリケーション向け
- **実装**: Spout の `SetFrameSync` / `WaitFrameSync` をラップ

### 内部デバイス生成モード

- **目的**: FFI 側で DirectX デバイスを作成・管理
- **想定 API**:
  ```c
  SPOUTDX_FFI_API int spoutdx_receiver_open_dx11_default(
      SpoutDxReceiverHandle handle
  );
  ```
- **根拠**: 簡易ツールでは外部デバイス管理が不要
- **課題**: デバイスのライフサイクル管理を明確化

## 中期（将来の拡張）

### Sender API

- **目的**: テクスチャの送信機能
- **想定 API**:
  ```c
  typedef void* SpoutDxSenderHandle;
  
  SPOUTDX_FFI_API SpoutDxSenderHandle spoutdx_sender_create(void);
  SPOUTDX_FFI_API int spoutdx_sender_destroy(SpoutDxSenderHandle handle);
  SPOUTDX_FFI_API int spoutdx_sender_open_dx11(
      SpoutDxSenderHandle handle,
      void* device
  );
  SPOUTDX_FFI_API int spoutdx_sender_send_texture(
      SpoutDxSenderHandle handle,
      void* texture,
      const char* sender_name
  );
  ```
- **根拠**: 受信だけでなく送信も必要なユースケースがある
- **設計方針**: Receiver と同様のパターンを踏襲

### Release ビルドの配布形態

- **目的**: パッケージング、バージョン管理の整備
- **検討事項**:
  - DLL + ヘッダのアーカイブ配布
  - バージョン番号の付与（セマンティックバージョニング）
  - GitHub Releases での配布
  - vcpkg / Conan などのパッケージマネージャー対応
- **根拠**: 他プロジェクトからの利用を容易にする

### CI/CD の整備

- **目的**: ビルドの再現性チェック
- **候補**:
  - GitHub Actions（Debug/Release ビルド）
  - 静的解析（clang-tidy, cppcheck）
  - コードフォーマットチェック
- **課題**: DirectX 11 実機依存のため、完全な自動テストは困難
- **暫定案**: ビルド確認のみ自動化、DirectX 依存テストは手動実行

**詳細**: [TESTING_STRATEGY.md](TESTING_STRATEGY.md)

## 長期（将来の可能性）

### DirectX 12 サポート

- **目的**: 最新の DirectX API への対応
- **課題**: Spout 本体が DX12 をサポートする必要がある
- **現状**: スコープ外（DX11 のみ）

### OpenGL 経路のサポート

- **目的**: OpenGL アプリケーションとの統合
- **課題**: Spout の OpenGL 実装をソース統合する必要がある
- **現状**: スコープ外（DX11 のみ）

### クロスプラットフォーム対応

- **目的**: Linux / macOS での利用
- **課題**: Spout は Windows 専用（代替として NDI 等を検討）
- **現状**: スコープ外（Windows のみ）

## 設計方針の維持

将来の実装において、以下の原則を維持します:

1. **C ABI のみに露出**
   - すべての公開関数は `extern "C"` で宣言
   - C++ 型（`std::string`, `std::vector` 等）は境界に出さない

2. **例外安全性の確保**
   - すべての公開関数で `try/catch (...)` を使用
   - エラーは数値コード（`SpoutDxResult`）で返す

3. **ハンドルベース設計**
   - 内部 C++ クラスは不透明ハンドル（`void*`）として公開
   - ライフサイクル管理を明確化（create/destroy）

4. **段階的な実装**
   - 最小限の API から開始し、必要に応じて拡張
   - 各機能を独立してテスト可能にする

5. **ドキュメントの同期**
   - 実装追加時は該当ドキュメントを必ず更新
   - コード例とリファレンスを含める

**詳細**: [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md)

## Notes

- このファイルは「実装追加のたびに更新する計画表」です
- 実装されていない仕様を断定しないため、詳細な API 仕様は実装着手時に別ドキュメントへ分離します
- 実装完了時は該当項目を「実装済み」セクションへ移動し、✅ マークを付けます
