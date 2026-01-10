# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI/CD設定（test, clippy, fmt）
- Cargo.tomlにプロジェクトメタデータ追加
- .gitignoreの充実化
- CHANGELOG.md追加

## [0.1.0] - 2026-01-11

### Added
- Core部のテストカバレッジ100%達成
- 非同期向き復元機能（GUIフリーズ防止）
- 6面スキャン入力機能
- ファイル保存/読み込み機能
- 3D描画エンジン
- 双方向BFSソルバー
- リアルタイム進捗表示
- 解法ステップ操作（前進/後退/リセット/最後まで）
- アニメーション制御（速度調整、イージング関数）
- 向き考慮/無視モード切り替え
- キーボードショートカット対応

### Technical
- FxHashMapによる高速ハッシュ処理
- Entry APIによる効率的なHashMapアクセス
- OnceLockによる完成状態のキャッシュ
- 容量事前確保によるメモリ最適化

### Quality
- Clippy警告ゼロ
- 包括的なテストスイート（約100件）
- Rustdocによる充実したドキュメント
