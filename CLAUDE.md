# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# jbig2enc-rs

C++版 [jbig2enc](https://github.com/agl/jbig2enc) のRust移植。JBIG2はbi-level（1bpp）画像をG4より高い圧縮率で符号化するフォーマット。Rust edition 2024。

画像処理基盤として [leptonica-rs](https://github.com/tagawa0525/leptonica-rs) を利用する。

## ビルド・テスト・リント

```bash
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

## リファレンス

C++版ソースとleptonica-rsをgit submoduleとして保持:

```bash
git submodule update --init --recursive
```

- `reference/jbig2enc/` — C++版jbig2enc（移植元）
- `reference/leptonica/` — C版Leptonica（jbig2enc依存ライブラリ・API参照用）
- `reference/leptonica-rs/` — Rust版Leptonica（画像処理基盤）

### C++版の主要ソース

| ファイル | 内容 |
|---------|------|
| `reference/jbig2enc/src/jbig2enc.h` | 公開API定義 |
| `reference/jbig2enc/src/jbig2.cc` | CLIツール・テキストセグメンテーション |
| `reference/jbig2enc/src/jbig2enc.cc` | エンコーダ本体（シンボル管理・ページ生成） |
| `reference/jbig2enc/src/jbig2arith.cc` | 算術符号化（QMコーダ） |
| `reference/jbig2enc/src/jbig2sym.cc` | シンボル処理・リファインメント符号化 |
| `reference/jbig2enc/src/jbig2comparator.cc` | シンボル比較・分類 |
| `reference/jbig2enc/src/jbig2structs.h` | JBIG2セグメント・ヘッダのデータ構造 |

### JBIG2仕様書

`reference/jbig2enc/doc/fcd14492.pdf`

## アーキテクチャ

### C++版jbig2encの処理フロー

```
入力画像(1bpp) → テキストセグメンテーション → シンボル抽出・分類(JbClasser)
    → シンボルテーブル符号化(jbig2_pages_complete)
    → ページ別符号化(jbig2_produce_page) → JBIG2出力 or PDF埋め込み断片
```

### 主要API（移植対象）

- `jbig2_init()` — 圧縮コンテキスト生成（threshold, weight, xres/yres, full_headers, refine_level）
- `jbig2_add_page()` — ページ追加（シンボル抽出・分類を実行）
- `jbig2_pages_complete()` — シンボルテーブル符号化
- `jbig2_produce_page()` — ページ符号化
- `jbig2_encode_generic()` — 単一ページのジェネリックリージョン符号化（ロスレス）
- `jbig2enc_auto_threshold()` — シンボルクラス統合

### leptonica-rs API互換性

`reference/leptonica-rs/docs/porting/jbig2enc-api-compatibility.md` にC++版が使うleptonica関数とleptonica-rs APIの差異を記載。移植前に確認すること。

HIGH差異（移植ブロッカー）:
1. `morph_sequence`の`r`（ランク縮小）・`x`（バイナリ拡張）演算子が未実装
2. `pixRasterop`のオフセット・領域指定付きラスタ演算が未実装

## PRワークフロー

### コミット構成

1. RED: テスト（`#[ignore = "not yet implemented"]` 付き）
2. GREEN: 実装（`#[ignore]` 除去）
3. REFACTOR: 必要に応じて
4. 全テスト・clippy・fmt通過を確認

### PR作成〜マージ

1. PR作成
2. `/gh-actions-check` でCopilotレビューワークフローが `completed/success` になるまで待つ
3. `/gh-pr-review` でコメント確認・対応
4. レビュー修正は独立した `fix:` コミットで積む（RED/GREENに混入させない）
5. push後の再レビューサイクルも完了を確認
6. `docs/plans/` の進捗ステータスを更新（`docs:` コミット）
7. 全チェック通過後 `/gh-pr-merge --merge`

### 規約

- ブランチ命名: `feat/<機能>`, `test/<スコープ>`, `refactor/<スコープ>`, `docs/<スコープ>`
- コミット: Conventional Commits
- マージコミット: `## Why` / `## What` / `## Impact` セクション
- 計画書 (`docs/plans/`) を実装着手前にコミットすること

## 計画書

`docs/plans/NNN_<機能名>.md`（NNN = Phase番号×100 + 連番）。Status: PLANNED → IN_PROGRESS → IMPLEMENTED。C++版の対応ファイル・関数を明記。
