# spoutdx-ffi

Rust から Spout（DirectX 11）を扱うための **純 C ABI シム**です。
Spout2 の必要最小ソースをビルドに統合するため、DLL/C++ ABI 互換性問題を回避できます。

## ドキュメント

詳細なプロジェクト情報は docs/ に集約しています。

- 索引: [AGENTS.md](AGENTS.md)
- docs 入口: [docs/README.md](docs/README.md)

## ビルド（Windows）

### クイックスタート（推奨）

```powershell
# DLL ビルド + example 実行 を一度に行う
.\dev.ps1

# Release ビルド版で実行
.\dev.ps1 -Release

# DLL のみ再ビルド（example は実行しない）
.\dev.ps1 -NoExample

# Rust example のみ実行（DLL 再ビルドをスキップ）
.\dev.ps1 -NoRebuild
```

### 手動ビルド

```powershell
# 1. C ABI シム DLL をビルド
cmake --preset msvc-debug
cmake --build --preset msvc-debug

# 2. Rust example を実行
cd examples
cargo run
```

## スクリプト仕様

- **Windows**: `dev.ps1` (PowerShell)
- **macOS/Linux**: `dev.sh` (Bash)

| オプション | 説明 |
|-----------|------|
| `-Release` | Release ビルド（デフォルト: Debug） |
| `-NoExample` | example 実行をスキップ（DLL のみビルド） |
| `-NoRebuild` | DLL ビルドをスキップ（example のみ実行） |
