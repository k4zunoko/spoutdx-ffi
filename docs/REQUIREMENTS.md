# REQUIREMENTS

## 対象環境

- **OS**: Windows 10/11（x64）
- **ビルドツール**: CMake 3.25+ + MSVC（Visual Studio 2022 / Build Tools）
- **依存ライブラリ**: Windows SDK（D3D11/DXGI 等）
- **実行環境**: DirectX 11 対応 GPU

**根拠**: Spout は Windows 専用であり、DirectX 11 を使用するため、MSVC と Windows SDK が必須です。

## 機能要件

### 実装済み機能

#### 1. 基本 API

**1.1 Spout SDK バージョン取得**

- 機能: Spout SDK のバージョン番号を取得
- API: `spoutdx_ffi_get_sdk_version()`
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L14-L19)

**1.2 FFI ライブラリバージョン取得**

- 機能: spoutdx-ffi 自体のバージョン文字列を取得
- API: `spoutdx_ffi_version()`
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L11-L13)

**1.3 DirectX 11 初期化疎通テスト**

- 機能: DirectX 11 デバイスの作成と破棄が正常に動作するかをテスト
- API: `spoutdx_ffi_test_dx11_init()`
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L21-L31)

#### 2. Receiver API（受信機能） - 2026-01-08 実装完了

**2.1 ライフサイクル管理**

- 機能: Receiver インスタンスの作成と破棄
- API:
  - `spoutdx_receiver_create()` - Receiver 作成
  - `spoutdx_receiver_destroy(handle)` - Receiver 破棄
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L50-L69)

**2.2 DirectX 11 初期化**

- 機能: 外部で作成した `ID3D11Device*` を使用して Receiver を初期化
- API:
  - `spoutdx_receiver_open_dx11(handle, device)` - 外部デバイスで初期化
  - `spoutdx_receiver_close_dx11(handle)` - DirectX をクローズ
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L74-L101)
- 根拠: 呼び出し側が独自の DirectX パイプラインを持つ場合、既存デバイスを再利用することで効率的な統合が可能

**2.3 受信設定**

- 機能: 接続するセンダーの名前を指定（NULL で active sender）
- API: `spoutdx_receiver_set_sender_name(handle, sender_name)`
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L106-L114)

**2.4 テクスチャ受信**

- 機能: センダーからテクスチャを受信
- API:
  - `spoutdx_receiver_receive_texture(handle, dst_texture)` - 指定したテクスチャへ受信
  - `spoutdx_receiver_receive(handle)` - 内部テクスチャへ受信
  - `spoutdx_receiver_get_received_texture(handle)` - 内部受信テクスチャを取得
  - `spoutdx_receiver_get_dx11_context(handle)` - D3D11 コンテキストを取得
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L119-L179)
- 根拠: `receive_texture` は既存パイプライン統合用、`receive` は簡易ツール用として提供

**2.5 Receiver リリース**

- 機能: Receiver を解放し、別のセンダーへ再接続可能にする
- API: `spoutdx_receiver_release(handle)`
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L181-L189)

**2.6 状態取得**

- 機能: センダー情報と接続状態の取得
- API:
  - `spoutdx_receiver_get_sender_info(handle, out_info)` - センダー情報取得
  - `spoutdx_receiver_is_updated(handle)` - センダー変更検出
  - `spoutdx_receiver_is_connected(handle)` - 接続状態確認
  - `spoutdx_receiver_is_frame_new(handle)` - 新規フレーム確認
- 実装: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp#L194-L231)
- 根拠: サイズ変更やフォーマット変更を検出し、テクスチャの再作成などに対応するため

### 未実装機能（ロードマップ）

以下は将来の実装候補です（[ROADMAP.md](ROADMAP.md) 参照）:

- **Sender API**: テクスチャの送信機能
- **センダー一覧取得**: 利用可能なセンダーのリスト取得
- **フレーム同期機能**: `SetFrameSync`, `WaitFrameSync`
- **内部デバイス生成モード**: FFI 側で DirectX デバイスを作成

## 非機能要件

### 1. ABI 安定性

- **要件**: Rust から安全に呼べる C ABI を維持する
- **根拠**: C++ ABI は標準化されておらず、コンパイラ・バージョン間で互換性がない
- **実装**: すべての公開関数を `extern "C"` で宣言
- **検証**: [include/spoutdx_ffi/spoutdx_ffi.h](../include/spoutdx_ffi/spoutdx_ffi.h)

### 2. 例外安全性

- **要件**: C ABI 境界を例外が越えない
- **根拠**: C++ 例外が C ABI を越えると未定義動作となる
- **実装**: すべての公開関数で `try/catch (...)` を使用
- **検証**: [src/spoutdx_ffi.cpp](../src/spoutdx_ffi.cpp) のすべての公開関数

### 3. エラーハンドリング

- **要件**: すべてのエラーを数値コードで明示的に返す
- **根拠**: エラー状況を呼び出し側で適切に処理できるようにする
- **実装**: `SpoutDxResult` enum を使用
- **詳細**: [ERROR_HANDLING.md](ERROR_HANDLING.md)

### 4. ビルド再現性

- **要件**: `CMakePresets.json` のプリセットでビルド手順が固定される
- **根拠**: 環境依存のビルド失敗を防ぎ、CI/CD を容易にする
- **実装**: [CMakePresets.json](../CMakePresets.json)

### 5. パフォーマンス

- **要件**: テクスチャコピーのオーバーヘッドを最小化
- **根拠**: リアルタイム映像伝送では低レイテンシが重要
- **実装**: SpoutDX の内部実装を活用（GPU 間共有メモリ）

## 提供形態（重要）

### スコープ内

- **C ABI シム DLL**: `spoutdx_ffi.dll`
- **ヘッダファイル**: `include/spoutdx_ffi/spoutdx_ffi.h`
- **利用ガイド**: [USAGE_DLL.md](../USAGE_DLL.md)

### スコープ外

このプロジェクトは **shim DLL（C ABI）提供に限定**します。以下は含めません:

- Rust の bindings/wrapper クレート（必要なら利用側で別途管理）
- 高レベル API ラッパー（安全性チェック、ライフタイム管理など）

**根拠**: FFI 層は最小限に保ち、言語固有のラッパーは利用側の要件に応じて実装する方が柔軟性が高い。

## 依存関係

### Windows ライブラリ（リンク）

- `d3d11.lib` - Direct3D 11 API
- `dxgi.lib` - DirectX Graphics Infrastructure
- `psapi.lib` - Process Status API（SpoutUtils 用）
- `version.lib` - バージョン情報リソース（SpoutUtils 用）
- `winmm.lib` - Windows Multimedia（SpoutFrameCount 用）

**定義場所**: [CMakeLists.txt](../CMakeLists.txt#L76-L83)

### Spout ソース

- **配置**: [third_party/Spout2/SPOUTSDK/](../third_party/Spout2/SPOUTSDK/)
- **ビルド対象**: [CMakeLists.txt](../CMakeLists.txt#L22-L35) の `SPOUT_SOURCES`
- **根拠**: ソース統合により C++ ABI 互換性問題を回避

## 受信 API の優先順位（設計判断）

### 主要 API: `receive_texture`（既存パイプライン統合）

- 呼び出し側が D3D11 デバイス/出力先テクスチャを用意
- 受信結果を指定テクスチャへ書き込み
- 根拠: ゲームエンジンや映像ツールでの統合が容易

### 補助 API: `receive`（簡易ツール用）

- 内部テクスチャへ受信し、ポインタを返す
- テスト/ツール用途として補助的に提供
- 根拠: 簡易ツールではテクスチャ管理を簡略化できる

## Assumptions（前提）

- Spout の必要最小ソースはリポジトリに含まれており、追加ダウンロード不要
- 主要ターゲットは x64（32-bit は未サポート）
- DirectX 11 互換 GPU が利用可能
- Windows 10 以降（古い Windows では未検証）

## Questions（検討事項）

- CI 環境での自動ビルド・テストをどう実現するか？（DX11 実機依存）
- Release ビルドの配布形態（パッケージング、バージョン管理）をどうするか？

