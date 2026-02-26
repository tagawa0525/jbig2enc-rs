# Phase 9: Verbose統計出力 & Text/Graphicsセグメンテーション

Status: IN_PROGRESS

## Context

C++版jbig2encからの移植はPhase 1〜8で完了済み。残りの2つの未実装機能を補完する:
1. `-v` (verbose) の圧縮統計出力
2. `-S` (text/graphics segmentation) のセグメンテーション機能

## 変更対象ファイル

| ファイル | 変更内容 |
|---------|---------|
| `src/encoder/context.rs` | verbose フィールド追加、`pages_complete()` で統計出力 |
| `src/pipeline.rs` | `segment_image()` 関数追加 |
| `src/cli.rs` | `-S` の NotImplemented エラー除去 |
| `src/main.rs` | verbose 設定、セグメンテーション呼び出し追加 |

## 1. Verbose 統計出力

### C++版の動作 (`jbig2enc.cc:662-665`)

`pages_complete()` 内で以下を stderr に出力:
- ページ数
- シンボル数
- log2(シンボル数)

### 実装

- `Jbig2Context` に `verbose: bool` フィールド追加（デフォルト `false`）
- `set_verbose(&mut self, verbose: bool)` setter 追加
- `pages_complete()` 内で統計出力

## 2. Text/Graphics セグメンテーション (`-S`)

### C++版の処理 (`jbig2.cc:141-213`)

1. `morph_sequence(pixb, "r11")` → mask
2. `morph_sequence(pixb, "r1143 + o4.4 + x4")` → seed
3. `seedfill_binary_restricted(seed, mask, 8-way)` → filled
4. `morph_sequence(filled, "d3.3")` → dilated
5. `expand_replicate(dilated, 4)` → 元サイズに復元
6. `pixSubtract(pixb, pixb, pixd)` → テキスト = binary AND NOT graphics_mask
7. ピクセル数チェック（< 100 でスキップ）
8. graphics画像の深度変換と合成

### 実装: `pipeline::segment_image()`

```rust
pub fn segment_image(pixb: &Pix, piximg: &Pix, verbose: bool)
    -> Result<(Option<Pix>, Option<Pix>), CliError>
```

- 返り値: `(text: Option<Pix>, graphics: Option<Pix>)`

## TDD コミット構成

1. RED: verbose テスト（`#[ignore]`）
2. GREEN: verbose 実装
3. RED: segment_image テスト（`#[ignore]`）
4. GREEN: segment_image 実装 + CLI 統合
