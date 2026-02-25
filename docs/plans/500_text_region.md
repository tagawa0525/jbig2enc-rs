# Phase 5: テキストリージョン符号化

**Status**: IMPLEMENTED

## 概要

C++版 `jbig2enc_textregion()`（`jbig2sym.cc:218-461`）のRust移植。
シンボルインスタンスをストリップ単位でソートし、位置デルタ＋シンボルID＋リファインメントを
算術符号化する。

## C++版対応

| C++関数/構造体 | Rust対応 |
|---|---|
| `jbig2enc_textregion()` | `encode_text_region()` |
| `YSorter` | `sort_by_key(y)` |
| `XSorter` | `sort_by_key(x)` within strip |
| `jbig2enc_int(ctx, JBIG2_IADT, ...)` | `ArithEncoder::encode_int(IntProc::Dt, ...)` |
| `jbig2enc_int(ctx, JBIG2_IAFS, ...)` | `ArithEncoder::encode_int(IntProc::Fs, ...)` |
| `jbig2enc_int(ctx, JBIG2_IADS, ...)` | `ArithEncoder::encode_int(IntProc::Ds, ...)` |
| `jbig2enc_int(ctx, JBIG2_IAIT, ...)` | `ArithEncoder::encode_int(IntProc::It, ...)` |
| `jbig2enc_iaid(ctx, symbits, ...)` | `ArithEncoder::encode_iaid(symbits, ...)` |
| `jbig2enc_int(ctx, JBIG2_IARI, ...)` | `ArithEncoder::encode_int(IntProc::Ri, ...)` |
| `jbig2enc_oob(ctx, JBIG2_IADS)` | `ArithEncoder::encode_oob(IntProc::Ds)` |
| `symmap` / `symmap2` | `HashMap<usize, usize>` |

## API設計

```rust
/// テキストリージョンに配置するシンボルインスタンス。
pub struct SymbolInstance {
    /// X座標（左端）
    pub x: i32,
    /// Y座標（下端、lower-left convention）
    pub y: i32,
    /// シンボルクラスID（シンボル辞書のインデックス）
    pub class_id: usize,
}

/// テキストリージョンの算術符号化結果。
pub struct TextRegionResult {
    /// 算術符号化されたデータ
    pub data: Vec<u8>,
}

/// テキストリージョンを算術符号化する。
pub fn encode_text_region(
    instances: &[SymbolInstance],
    symbols: &[Pix],
    symmap: &HashMap<usize, usize>,
    symmap2: Option<&HashMap<usize, usize>>,
    global_sym_count: usize,
    symbits: u32,
    strip_width: u32,
    unborder: bool,
    border_size: u32,
) -> Result<TextRegionResult, Jbig2Error>
```

- `instances`: 配置するシンボルインスタンスの配列
- `symbols`: シンボルテンプレート配列（幅取得用）
- `symmap`: グローバル辞書のマッピング（class_id → encoded_id）
- `symmap2`: ページ固有辞書のマッピング（Optional）
- `global_sym_count`: グローバル辞書のシンボル数（symmap2のオフセット計算用）
- `symbits`: シンボルID符号化に必要なビット数（`log2(total_symbols)` の切り上げ）
- `strip_width`: ストリップ高さ（1, 2, 4, 8 のいずれか）
- `unborder` / `border_size`: ボーダー除去設定（curs更新のシンボル幅計算用）

## 処理フロー

1. **Y座標でソート**: 全インスタンスをY座標昇順にソート
2. **ストリップ分割**: `floor(y / strip_width) * strip_width` で同一ストリップを特定
3. **X座標でソート**: 各ストリップ内でX座標昇順にソート
4. **初期IADT**: `encode_int(Dt, 0)` を符号化
5. **ストリップごとの処理**:
   - デルタT: `encode_int(Dt, (height - stript) / strip_width)`
   - 各シンボル:
     - 最初のシンボル: `encode_int(Fs, deltafs)`
     - 2番目以降: `encode_int(Ds, deltas)`
     - strip_width > 1 の場合: `encode_int(It, deltat)`
     - シンボルID: `encode_iaid(symbits, symid)`
     - curs更新: `curs += symbol_width - 1`
   - ストリップ終端: `encode_oob(Ds)`
6. **最終化**: `encode_final()`

## リファインメント

C++版でも「broken」とコメントされている。Phase 5では非リファインメントパスのみ実装。
リファインメントが必要になった場合は後続のPRで追加する。

## テスト方針

1. **ストリップ分割**: 既知座標のシンボルでストリップ境界を検証
2. **X座標ソート**: 同一ストリップ内の左→右順序を検証
3. **出力の非空**: 空でないバイト列が返ること
4. **strip_width検証**: 不正な値でエラー
5. **symmap2対応**: 2辞書にまたがるシンボルIDの検索
6. **エッジケース**: 空リスト、単一シンボル、strip_width=1

## PR構成

1. `docs:` 計画書コミット
2. `test:` REDテスト（`#[ignore]` 付き）
3. `feat(symbol):` GREEN実装
