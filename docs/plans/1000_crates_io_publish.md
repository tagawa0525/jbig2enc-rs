# Phase 10: crates.io公開準備

Status: IN_PROGRESS

## Context

`jbig2enc-rs` をcrates.ioに公開するための整備。現状は `cargo publish --dry-run` で警告が出る状態（メタデータ不足、LICENSEファイル欠如、25.3MBのパッケージサイズ）。

## 変更対象ファイル

| ファイル | 変更内容 |
|---------|---------|
| `LICENSE` | Apache License 2.0 全文を新規作成 |
| `Cargo.toml` | メタデータ追加、clap feature分離、exclude設定 |
| `src/lib.rs` | クレートレベルドキュメント追加 |

## 1. LICENSEファイル作成

- `/LICENSE` にApache License 2.0の全文を配置
- READMEの記載およびオリジナルjbig2encのライセンスと一致

## 2. Cargo.tomlメタデータ追加・clap feature分離

- `description`, `license`, `repository`, `readme`, `keywords`, `categories` を追加
- `exclude` で `reference/`（サブモジュール25MB+）や開発用ファイルを除外
- `clap` を `optional = true` にし `cli` feature の背後に配置
- `[[bin]]` に `required-features = ["cli"]` を設定
- `default = ["cli"]` で `cargo install` 時はバイナリが自動ビルドされる

## 3. lib.rsクレートレベルドキュメント追加

`src/lib.rs` の先頭に英語で `//!` ドキュメントを追加:
- クレートの概要（JBIG2エンコーダのRust移植）
- 主要モジュールの説明
- 基本的な使い方

## 4. 検証

- `cargo publish --dry-run` で警告なし・パッケージサイズ妥当を確認
- `cargo test` 全テスト通過
- `cargo clippy --all-targets -- -D warnings` 通過
- `cargo doc --no-deps` でドキュメント生成確認
- `--no-default-features` でライブラリのみビルド確認
