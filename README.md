# spoutdx-ffi

Rust から Spout（DirectX 11）を扱うための **純 C ABI シム**です。
Spout2 の必要最小ソースをビルドに統合するため、DLL/C++ ABI 互換性問題を回避できます。

## ドキュメント

詳細なプロジェクト情報は docs/ に集約しています。

- 索引: [AGENTS.md](AGENTS.md)
- docs 入口: [docs/README.md](docs/README.md)

## ビルド（Windows）

```powershell
cmake --preset msvc-debug
cmake --build --preset msvc-debug
out\build\msvc-debug\Debug\spoutdx_ffi_example.exe
```
