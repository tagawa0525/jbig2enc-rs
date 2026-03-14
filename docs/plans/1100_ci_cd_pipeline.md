# CI/CD パイプライン構築 + 雑務

Status: IMPLEMENTED

## Context

本プロジェクトには `.github/` ディレクトリが存在せず、CI/CD が未構築。CLAUDE.md の PR ワークフローでは Copilot レビューや CI チェック通過が必須ゲートとして定義されているが、実行基盤がない状態。これを解消する。

あわせて以下の雑務も処理する:

- `docs/plans/majestic-mapping-cocke.md` の削除（`1000_crates_io_publish.md` と内容重複）
- `README.md` / `README.ja.md` に日英相互リンクを追加

## ファイル構成

```text
.github/workflows/
  ci.yml   # CI: lint + test
  cd.yml   # CD: crates.io publish + GitHub Release
```

注: Copilot コードレビューは GitHub が自動管理する
`dynamic/copilot-pull-request-reviewer` ワークフローとして実行されるため、
カスタムワークフローは不要。

## 1. CI ワークフロー (`ci.yml`)

**トリガー:** `push` to main, `pull_request` to main

### lint ジョブ (ubuntu-latest, stable)

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --no-default-features --all-targets -- -D warnings`

### test ジョブ (matrix)

| toolchain   | OS                     |
| ----------- | ---------------------- |
| 1.88 (MSRV) | ubuntu, macos, windows |
| stable      | ubuntu                 |
| nightly     | ubuntu                 |

- `cargo test`（default features = CLI 含む）
- `cargo test --no-default-features`（ライブラリのみ）

`fail-fast: false` で全組み合わせを完走させる。

**前提修正:** `tests/cli.rs` の先頭に `#![cfg(feature = "cli")]` を追加。これがないと `cargo test --no-default-features` でCLIバイナリが見つからずコンパイルエラーになる。

**キャッシュ:** `Swatinem/rust-cache@v2`（toolchain を key に指定）

## 2. CD ワークフロー (`cd.yml`)

**トリガー:** `push` tags `v*`

### verify ジョブ

- fmt / clippy / test を再実行
- タグが main ブランチ上にあることを検証（`git merge-base --is-ancestor` で origin/main 確認）
- タグ名 (`v0.1.0`) と `Cargo.toml` の `version` が一致することを検証

### publish ジョブ (needs: verify)

- `cargo publish`（`CARGO_REGISTRY_TOKEN` シークレット使用）

### release ジョブ (needs: verify)

- `softprops/action-gh-release@v2` で GitHub Release 作成（リリースノート自動生成）

**必要なシークレット:** `CARGO_REGISTRY_TOKEN`（crates.io API トークン）

**セキュリティ:** 全ての `uses:` はコミット SHA にピン留め（`@v4` 等の可変タグは使用しない）

## 3. 雑務

### `docs/plans/majestic-mapping-cocke.md` の削除

`1000_crates_io_publish.md` と同一内容の重複ファイル。git rm で削除。

### README 相互リンク

- `README.md` のタイトル直後に: `[日本語版はこちら](README.ja.md)`
- `README.ja.md` のタイトル直後に: `[English version](README.md)`

## 対象ファイル

| ファイル                               | 操作                                    |
| -------------------------------------- | --------------------------------------- |
| `tests/cli.rs`                         | 編集（`#![cfg(feature = "cli")]` 追加） |
| `.github/workflows/ci.yml`             | 新規作成                                |
| `.github/workflows/cd.yml`             | 新規作成                                |
| `README.md`                            | 編集（日本語版リンク追加）              |
| `README.ja.md`                         | 編集（英語版リンク追加）                |
| `docs/plans/majestic-mapping-cocke.md` | 削除                                    |

## コミット構成

1. `fix(test): guard cli tests with cfg(feature = "cli")`
2. `ci: add CI workflow`
3. `ci: add CD workflow`
4. `docs: add language links to READMEs`
5. `fix: bump MSRV to 1.88 (leptonica requires let-chains)`
6. `ci: remove copilot-review.yml (GitHub manages it automatically)`
7. `fix(cd): fix tag-on-main check and pin actions to SHA`
8. `fix(ci): pin actions to commit SHA for supply-chain security`

## 検証

1. `cargo test` — 既存テスト全通過
2. `cargo test --no-default-features` — CLI テストがスキップされ成功
3. `cargo clippy --all-targets -- -D warnings` — 警告なし
4. PR を作成し、CI ワークフローが起動・成功することを確認
