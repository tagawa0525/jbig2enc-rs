# Phase 6: シンボル視覚的等価性判定

**Status**: IMPLEMENTED

## 概要

C++版 `jbig2enc_are_equivalent()`（`jbig2comparator.cc:44-262`）のRust移植。
2つの1bppシンボルテンプレートが視覚的に等価かどうかを、XOR差分の空間分布分析で判定する。

## C++版対応

| C++関数/構造体                                    | Rust対応                                |
| ------------------------------------------------- | --------------------------------------- |
| `jbig2enc_are_equivalent(PIX*, PIX*)`             | `are_equivalent(&Pix, &Pix)`            |
| `pixSizesEqual()`                                 | `Pix::sizes_equal()`                    |
| `pixGetWpl()`                                     | `Pix::wpl()`                            |
| `pixXor(NULL, pix1, pix2)`                        | `Pix::xor(&other)`                      |
| `pixCountPixels(pix, &count, NULL)`               | `Pix::count_pixels()`                   |
| `pixThresholdPixelSum(pix, thresh, &above, NULL)` | `Pix::threshold_pixel_sum(thresh)`      |
| `pixGetPixel(pix, x, y, &val)`                    | `Pix::get_pixel(x, y)`                  |
| `pixGetDimensions(pix, &w, &h, &d)`               | `Pix::width()` / `height()` / `depth()` |

## API設計

```rust
use leptonica::Pix;
use crate::error::Jbig2Error;

/// 2つのシンボルテンプレートが視覚的に等価かどうかを判定する。
pub fn are_equivalent(first: &Pix, second: &Pix) -> Result<bool, Jbig2Error>
```

- `first`, `second`: 1bppシンボルテンプレート
- サイズ不一致/深度不一致 → `Ok(false)`（C++と同じくエラーではなく非等価として返す）
- leptonica API呼び出し失敗 → `Err(Jbig2Error::...)`

## アルゴリズム

### Stage 1: 早期棄却

1. **サイズチェック**: `sizes_equal()` で width/height/depth が一致するか
2. **WPLチェック**: `wpl()` が一致するか（sizes_equal が true なら通常一致するが、C++に忠実に）
3. **深度チェック**: `depth() == PixelDepth::Bit1` か

### Stage 2: XOR差分の全体評価

1. `first.xor(second)` で差分画像を生成
2. `first.count_pixels()` で第1テンプレートのONピクセル数（`pcount`）を取得
3. `xor_pix.threshold_pixel_sum(pcount / 4)` で差分が25%を超えるか早期判定
   - 超える → `Ok(false)`

### Stage 3: 9x9グリッド空間分布分析

画像を9x9のセルに分割（余りピクセルは先頭セルに+1ずつ分配）。
各セルについてXOR差分画像のONピクセルを4象限（左/右/上/下）にカウント:

- `parsed_pix_counts[9][9]` — セルごとの合計差分ピクセル数
- `horizontal_parsed_pix_counts[18][9]` — 左右分割カウント
- `vertical_parsed_pix_counts[9][18]` — 上下分割カウント

### Stage 4: パターン検出（4種の棄却チェック）

閾値計算:

- `vertical_part = height / 9`, `horizontal_part = width / 9`
- `a = max(h_part/2, w_part/2)`, `b = min(h_part/2, w_part/2)`
- `point_thresh = a * b * PI`（楕円面積）
- `vline_thresh = (v_part * (h_part/2)) * 0.9`
- `hline_thresh = (h_part * (v_part/2)) * 0.9`

1. **水平線検出**: `horizontal_parsed_pix_counts` の2x2ブロック合計が `hline_thresh` 以上か
2. **垂直線検出**: `vertical_parsed_pix_counts` の2x2ブロック合計が `vline_thresh` 以上か
3. **交差線検出**: `parsed_pix_counts` の3x3ブロック内の対角線合計が `hline_thresh` 以上か
4. **集中差分検出**: `parsed_pix_counts` の2x2ブロック合計が `point_thresh` 以上か

全チェックをパス → `Ok(true)`（等価）

## leptonica-rs API

| 関数                  | シグネチャ                                                   | 備考                   |
| --------------------- | ------------------------------------------------------------ | ---------------------- |
| `sizes_equal`         | `fn sizes_equal(&self, other: &Pix) -> bool`                 | width/height/depth比較 |
| `wpl`                 | `fn wpl(&self) -> u32`                                       |                        |
| `xor`                 | `fn xor(&self, other: &Pix) -> Result<Pix>`                  | 新規Pixを返す          |
| `count_pixels`        | `fn count_pixels(&self) -> u64`                              | C++は`l_int32`         |
| `threshold_pixel_sum` | `fn threshold_pixel_sum(&self, thresh: u64) -> Result<bool>` | true=超過              |
| `get_pixel`           | `fn get_pixel(&self, x: u32, y: u32) -> Option<u32>`         |                        |
| `depth`               | `fn depth(&self) -> PixelDepth`                              | enum                   |

## テスト方針

1. **同一シンボル**: 同じ画像 → `Ok(true)`
2. **異なるサイズ**: width/height不一致 → `Ok(false)`
3. **全白 vs 全黒**: 差分が大きすぎる → `Ok(false)`
4. **微小差分**: 数ピクセルの違い → `Ok(true)`（閾値内）
5. **水平線パターン**: 水平方向に連続した差分 → `Ok(false)`
6. **集中差分パターン**: 局所的に大きな差分 → `Ok(false)`
7. **空画像（全白同士）**: ONピクセル0 → `Ok(true)`（threshold=0で即パス）

## PR構成

1. `docs:` 計画書コミット
2. `test:` REDテスト（`#[ignore]` 付き）
3. `feat(comparator):` GREEN実装
