# CI/CDパイプライン構成

## ワークフロー概要

### 1. CI (継続的インテグレーション)
**ファイル**: `.github/workflows/ci.yml`

#### テストジョブ (test)
- **実行環境**: Ubuntu, Windows, macOS
- **Rustバージョン**: stable, beta, nightly (Ubuntu のみ)
- **実行内容**:
  - コンパイルチェック (`cargo check`)
  - 全テスト実行 (`cargo test --all-features`)
- **キャッシュ**: Cargoレジストリ、ビルド成果物

#### Lintジョブ (clippy)
- **実行環境**: Ubuntu
- **実行内容**:
  - Clippy静的解析 (`cargo clippy -D warnings`)

#### フォーマットジョブ (fmt)
- **実行環境**: Ubuntu
- **実行内容**:
  - コードフォーマットチェック (`cargo fmt --check`)

#### カバレッジジョブ (coverage)
- **実行環境**: Ubuntu
- **実行内容**:
  - `cargo-llvm-cov` でカバレッジ計測
  - Codecovへレポート送信

#### セキュリティ監査 (security-audit)
- **実行環境**: Ubuntu
- **実行内容**:
  - `cargo-audit` で既知の脆弱性チェック

#### リリースビルド (build-release)
- **実行環境**: Ubuntu, Windows, macOS
- **実行内容**:
  - リリースバイナリのビルド
  - アーティファクトのアップロード

### 2. Release (リリース自動化)
**ファイル**: `.github/workflows/release.yml`

- **トリガー**: `v*` タグのプッシュ (例: `v1.0.0`)
- **実行内容**:
  1. GitHubリリースの作成
  2. マルチプラットフォームバイナリのビルド:
     - Linux (x86_64)
     - Windows (x86_64)
     - macOS (x86_64, aarch64/Apple Silicon)
  3. 圧縮アーカイブの作成
  4. リリースへのアセット添付

### 3. Dependabot (依存関係の自動更新)
**ファイル**: `.github/dependabot.yml`

- **更新対象**:
  - Cargo パッケージ (毎週月曜日)
  - GitHub Actions (毎週月曜日)
- **プルリクエスト**: 最大5件まで同時オープン
- **ラベル**: 自動的に `dependencies` ラベルを付与

## 使い方

### 通常の開発フロー
1. ブランチでコードを変更
2. プルリクエストを作成
3. CI が自動実行され、全チェックをパス
4. レビュー後、`main` へマージ

### リリースの作成
```bash
# タグを作成
git tag v1.0.0
git push origin v1.0.0
```
→ 自動的にリリースビルドが実行され、GitHubリリースが作成されます

### Codecovの設定 (オプション)
1. https://codecov.io でアカウント作成
2. リポジトリを追加
3. トークンを GitHub Secrets に `CODECOV_TOKEN` として追加

## CI/CDバッジ

以下のバッジをREADME.mdに追加できます:

```markdown
![CI](https://github.com/katoy/rust-r-cube/actions/workflows/ci.yml/badge.svg)
![Security Audit](https://github.com/katoy/rust-r-cube/actions/workflows/ci.yml/badge.svg?job=security-audit)
[![codecov](https://codecov.io/gh/katoy/rust-r-cube/branch/main/graph/badge.svg)](https://codecov.io/gh/katoy/rust-r-cube)
```

## トラブルシューティング

### テストが失敗する場合
- ローカルで `cargo test --all-features` を実行して再現
- エラーログを確認

### Clippyエラー
- ローカルで `cargo clippy --all-targets --all-features -- -D warnings`
- 警告を修正してコミット

### ビルドエラー
- キャッシュの問題の可能性: GitHub ActionsのキャッシュをクリアしてRe-run
