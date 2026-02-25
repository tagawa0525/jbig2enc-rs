# Phase 3: ジェネリックリージョン符号化

**Status**: IN_PROGRESS

## 概要

C++版 `jbig2_encode_generic()`（`jbig2enc.cc:898-1002`）のRust移植。
1bpp画像をJBIG2ジェネリックリージョンとして符号化し、完全なJBIG2バイトストリームを返す。

## C++版対応

| C++関数/構造体 | Rust対応 |
|---|---|
| `jbig2_encode_generic()` | `encode_generic()` |
| `jbig2enc_bitimage()` | `ArithEncoder::encode_bitimage()` |
| `jbig2enc_final()` | `ArithEncoder::encode_final()` |
| `jbig2enc_datasize()` / `jbig2enc_tobuffer()` | `ArithEncoder::to_vec()` |
| `pixSetPadBits(bw, 0)` | `PixMut::set_pad_bits(0)` |
| `struct jbig2_file_header` | `wire::FileHeader` |
| `struct jbig2_page_info` | `wire::PageInfo` |
| `struct jbig2_generic_region` | `wire::GenericRegion` |
| `struct Segment` | `wire::SegmentHeader` |

## API設計

```rust
pub fn encode_generic(
    pix: &Pix,
    full_headers: bool,
    xres: u32,
    yres: u32,
    duplicate_line_removal: bool,
) -> Result<Vec<u8>, Jbig2Error>
```

- `pix`: 1bpp入力画像（`depth() == PixelDepth::Bit1` を検証）
- `full_headers`: true=完全なJBIG2ファイル、false=PDF埋め込み用断片
- `xres`/`yres`: 解像度（0の場合はPix自身の解像度を使用）
- `duplicate_line_removal`: TPGDフラグ（同一行スキップ最適化）
- 戻り値: JBIG2バイトストリーム

## 出力セグメント構成

### full_headers=true の場合

```
[FileHeader]                      13 bytes
[SegmentHeader #0: PageInfo]      11 bytes
[PageInfo data]                   19 bytes
[SegmentHeader #1: GenericRegion] 11 bytes
[GenericRegion header]            26 bytes
[算術符号化データ]                 variable
[SegmentHeader #2: EndOfPage]     11 bytes
[SegmentHeader #3: EndOfFile]     11 bytes
```

### full_headers=false の場合

```
[SegmentHeader #0: PageInfo]      11 bytes
[PageInfo data]                   19 bytes
[SegmentHeader #1: GenericRegion] 11 bytes
[GenericRegion header]            26 bytes
[算術符号化データ]                 variable
```

## GenericRegion固定パラメータ

C++版と同一の値を使用:
- `gbtemplate = 0`（テンプレート0: 4ピクセルAT）
- `mmr = false`
- `a1x=3, a1y=-1, a2x=-3, a2y=-1, a3x=2, a3y=-2, a4x=-2, a4y=-2`
- `tpgdon = duplicate_line_removal`

## 処理フロー

1. 入力検証（1bpp確認）
2. パッドビットゼロ化（`set_pad_bits(0)`）
3. 算術符号化（`ArithEncoder::encode_bitimage()` → `encode_final()` → `to_vec()`）
4. セグメント組み立て（FileHeader → PageInfo → GenericRegion → data → EndOfPage → EndOfFile）
5. バッファ結合して返却

## パッドビット処理

`Pix` は immutable（`Arc<PixData>`）なので、`pix.to_mut()` でコピーを作り
`set_pad_bits(0)` を呼ぶ。C++版の `pixSetPadBits(bw, 0)` に対応。

## テスト方針

1. **構造テスト**: 既知サイズの画像で出力バイト列のヘッダ部分を検証
   - FileHeader、PageInfo、SegmentHeaderのバイト列が期待値と一致
   - セグメントのdata_lengthが正しい
2. **TPGD on/off**: 同一画像でTPGD有無の出力差を確認
3. **エッジケース**: 幅が32の倍数でない画像、1x1画像
4. **エラーケース**: 非1bpp画像でエラー返却

## PR構成

1. `docs:` 計画書コミット
2. `test:` REDテスト（`#[ignore]`付き）
3. `feat(encoder):` GREEN実装
