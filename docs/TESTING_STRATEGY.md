# TESTING_STRATEGY

## 現状のテスト形態

- 自動テスト（ユニットテスト/CI）は未整備
- 動作確認用の実行ファイルで疎通を確認
  - [examples/src/main.rs](../examples/src/main.rs) (Rust)

## 手動テスト手順（Windows）

1. C++ シム DLL をビルド:
   ```
   cmake --preset msvc-debug
   cmake --build --preset msvc-debug
   ```
2. Rust example を実行:
   ```
   cd examples
   cargo run
   ```

期待結果:
- バージョン文字列が表示される
- Spout SDK version が正の値
- DirectX 11 initialization が OK

## 将来の自動化候補（未実装）

- C ABI 関数の戻り値契約を検証する軽量テスト（例: Catch2 / GoogleTest）
- CI で Debug/Release のビルド確認

## Assumptions

- 現時点では DX11 の実機依存があるため、CI 実行環境の確保が必要。
