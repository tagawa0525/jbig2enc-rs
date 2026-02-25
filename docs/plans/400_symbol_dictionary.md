# Phase 4: シンボル辞書符号化

**Status**: IMPLEMENTED

## 概要

C++版 `jbig2enc_symboltable()`（`jbig2sym.cc:91-180`）のRust移植。
シンボルテンプレートを高さ/幅でソートし、デルタ符号化+算術符号化でシンボル辞書データを生成する。

## C++版対応

| C++関数/構造体 | Rust対応 |
|---|---|
| `jbig2enc_symboltable()` | `encode_symbol_table()` |
| `HeightSorter` / `WidthSorter` | `sort_by_key` クロージャ |
| `jbig2enc_int(ctx, JBIG2_IADH, ...)` | `ArithEncoder::encode_int(IntProc::Dh, ...)` |
| `jbig2enc_int(ctx, JBIG2_IADW, ...)` | `ArithEncoder::encode_int(IntProc::Dw, ...)` |
| `jbig2enc_int(ctx, JBIG2_IAEX, ...)` | `ArithEncoder::encode_int(IntProc::Ex, ...)` |
| `jbig2enc_oob(ctx, JBIG2_IADW)` | `ArithEncoder::encode_oob(IntProc::Dw)` |
| `jbig2enc_bitimage()` | `ArithEncoder::encode_bitimage()` |
| `jbig2enc_final()` | `ArithEncoder::encode_final()` |
| `pixRemoveBorder()` | `Pix::remove_border()` |
| `pixSetPadBits()` | `PixMut::set_pad_bits()` |
| `kBorderSize = 6` | `border_size` パラメータ（leptonica-rs: `TEMPLATE_BORDER = 4`） |

## API設計

```rust
/// シンボルテーブルの算術符号化結果。
pub struct SymbolTableResult {
    /// 算術符号化されたデータ
    pub data: Vec<u8>,
    /// 元のシンボルインデックス → 符号化番号のマッピング
    pub symmap: HashMap<usize, usize>,
}

/// シンボル辞書を算術符号化する。
pub fn encode_symbol_table(
    symbols: &[Pix],
    symbol_indices: &[usize],
    unborder: bool,
    border_size: u32,
) -> Result<SymbolTableResult, Jbig2Error>
```

- `symbols`: シンボルテンプレート配列（`JbClasser.pixat` をそのまま渡す想定）
- `symbol_indices`: 符号化するシンボルのインデックス（`symbols` へのインデックス）
- `unborder`: true→各シンボルから `border_size` ピクセルのボーダーを除去して符号化
- `border_size`: ボーダーサイズ（`unborder=true` の場合のみ使用）
- 戻り値: 算術符号化データとシンボルマッピング

## 処理フロー

1. **ソート**: `symbol_indices` を高さ順にソート
2. **高さクラス分割**: 同一高さのシンボルをグループ化
3. **幅ソート**: 各高さクラス内で幅順にソート
4. **デルタ符号化**:
   - 高さクラスごとに `encode_int(Dh, delta_height)` でデルタ高さを符号化
   - 各シンボルについて `encode_int(Dw, delta_width)` でデルタ幅を符号化
   - 各シンボルのビットマップを `encode_bitimage()` で符号化
   - シンボルマッピングを記録（元インデックス→符号化番号）
   - 高さクラスの末尾で `encode_oob(Dw)` を符号化
5. **エクスポートテーブル**: `encode_int(Ex, 0)` + `encode_int(Ex, n)` で全シンボルをエクスポート
6. **最終化**: `encode_final()` でフラッシュ

## ボーダー処理

C++版: `kBorderSize = 6`、leptonica-rs: `TEMPLATE_BORDER = 4`。

`unborder=true` の場合:
- シンボル寸法: `width - 2 * border_size`, `height - 2 * border_size`
- `Pix::remove_border(border_size)` でボーダー除去してから符号化

`unborder=false` の場合:
- シンボルをそのまま符号化（`Pix::clone()` 相当）

## テスト方針

1. **ソート検証**: 既知サイズのシンボル群で symmap の順序が正しいか
2. **デルタ値の正確性**: シンボルの並び順からデルタ値を手計算で検証（直接検証は困難なので symmap で間接検証）
3. **出力の非空**: 空でないバイト列が返ること
4. **ボーダー除去**: unborder=true/false で出力が異なること
5. **エッジケース**: 空リスト、単一シンボル、全同一サイズ
6. **エラーケース**: 範囲外インデックス

## PR構成

1. `docs:` 計画書コミット
2. `test:` REDテスト（`#[ignore]` 付き）
3. `feat(symbol):` GREEN実装
