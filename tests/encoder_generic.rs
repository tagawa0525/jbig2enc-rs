use jbig2enc::encoder::encode_generic;
use jbig2enc::wire::{
    SEGMENT_END_OF_FILE, SEGMENT_END_OF_PAGE, SEGMENT_IMM_GENERIC_REGION, SEGMENT_PAGE_INFORMATION,
};
use leptonica::{Pix, PixMut, PixelDepth};

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

/// 全白の1bpp画像を作成する。
fn white_1bpp(width: u32, height: u32) -> Pix {
    PixMut::new(width, height, PixelDepth::Bit1).unwrap().into()
}

/// 全黒の1bpp画像を作成する。
fn black_1bpp(width: u32, height: u32) -> Pix {
    let mut pm = PixMut::new(width, height, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    pm.into()
}

/// チェッカーボードパターンの1bpp画像を作成する（1行おき）。
fn striped_1bpp(width: u32, height: u32) -> Pix {
    let mut pm = PixMut::new(width, height, PixelDepth::Bit1).unwrap();
    for y in 0..height {
        if y % 2 == 0 {
            let row = pm.row_data_mut(y);
            for word in row.iter_mut() {
                *word = 0xFFFF_FFFF;
            }
        }
    }
    pm.into()
}

/// セグメントヘッダからタイプフィールドを抽出する。
/// セグメントヘッダ: [0..4]=number, [4]=flags(bit0-5=type)
fn segment_type_at(data: &[u8], offset: usize) -> u8 {
    data[offset + 4] & 0x3F
}

/// セグメントヘッダからdata_lengthを抽出する。
/// セグメントの末尾4バイト（page=1byte想定）。
/// offset はセグメントヘッダ先頭。ヘッダ: 4(number)+1(flags)+1(referred)=6, +1(page)=7, +4(data_length)=11
fn segment_data_length_at(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        data[offset + 7],
        data[offset + 8],
        data[offset + 9],
        data[offset + 10],
    ])
}

// ---------------------------------------------------------------------------
// エラーケース
// ---------------------------------------------------------------------------

/// 8bpp画像を渡すとエラーになること。
#[test]
fn rejects_non_1bpp_image() {
    let pix = PixMut::new(32, 32, PixelDepth::Bit8).unwrap().into();
    let result = encode_generic(&pix, true, 0, 0, false);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// full_headers=true の構造テスト
// ---------------------------------------------------------------------------

/// 全白32x32画像（full_headers=true）の出力構造を検証する。
///
/// 期待レイアウト:
/// [0..13]   FileHeader
/// [13..24]  SegmentHeader #0 (PageInfo)
/// [24..43]  PageInfo data (19 bytes)
/// [43..54]  SegmentHeader #1 (GenericRegion)
/// [54..80]  GenericRegion header (26 bytes)
/// [80..X]   算術符号化データ
/// [X..X+11] SegmentHeader #2 (EndOfPage)
/// [X+11..X+22] SegmentHeader #3 (EndOfFile)
#[test]
fn full_headers_structure_white_32x32() {
    let pix = white_1bpp(32, 32);
    let output = encode_generic(&pix, true, 300, 300, false).unwrap();

    // FileHeader: magic(8) + flags(1) + n_pages(4) = 13
    assert_eq!(
        &output[0..8],
        &[0x97, 0x4a, 0x42, 0x32, 0x0d, 0x0a, 0x1a, 0x0a]
    );
    assert_eq!(output[8], 0x01); // organisation_type=1, unknown_n_pages=0
    assert_eq!(&output[9..13], &[0x00, 0x00, 0x00, 0x01]); // n_pages=1

    // Segment #0: PageInfo
    assert_eq!(segment_type_at(&output, 13), SEGMENT_PAGE_INFORMATION);
    assert_eq!(segment_data_length_at(&output, 13), 19);

    // PageInfo data starts at 24
    // width=32 → 0x00000020, height=32 → 0x00000020
    assert_eq!(&output[24..28], &[0x00, 0x00, 0x00, 0x20]); // width
    assert_eq!(&output[28..32], &[0x00, 0x00, 0x00, 0x20]); // height
    // xres=300=0x012C, yres=300=0x012C
    assert_eq!(&output[32..36], &[0x00, 0x00, 0x01, 0x2C]); // xres
    assert_eq!(&output[36..40], &[0x00, 0x00, 0x01, 0x2C]); // yres
    // flags: is_lossless=1
    assert_eq!(output[40], 0x01);

    // Segment #1: GenericRegion
    assert_eq!(segment_type_at(&output, 43), SEGMENT_IMM_GENERIC_REGION);
    let genreg_data_len = segment_data_length_at(&output, 43);
    // data_length = sizeof(GenericRegion)(26) + encoded_data
    assert!(genreg_data_len >= 26);

    // GenericRegion header starts at 54
    assert_eq!(&output[54..58], &[0x00, 0x00, 0x00, 0x20]); // width=32
    assert_eq!(&output[58..62], &[0x00, 0x00, 0x00, 0x20]); // height=32
    // flags byte at 71: tpgdon=0 (duplicate_line_removal=false)
    assert_eq!(output[71], 0x00);

    // 算術符号化データの後にEndOfPageとEndOfFileがある
    let arith_data_len = (genreg_data_len - 26) as usize;
    let end_of_page_offset = 54 + 26 + arith_data_len;
    assert_eq!(
        segment_type_at(&output, end_of_page_offset),
        SEGMENT_END_OF_PAGE
    );
    assert_eq!(segment_data_length_at(&output, end_of_page_offset), 0);

    let end_of_file_offset = end_of_page_offset + 11;
    assert_eq!(
        segment_type_at(&output, end_of_file_offset),
        SEGMENT_END_OF_FILE
    );
    assert_eq!(segment_data_length_at(&output, end_of_file_offset), 0);

    // 全体サイズの整合性
    assert_eq!(output.len(), end_of_file_offset + 11);
}

/// セグメント番号が0から順に振られていること。
#[test]
fn segment_numbers_sequential() {
    let pix = white_1bpp(32, 32);
    let output = encode_generic(&pix, true, 0, 0, false).unwrap();

    // Segment #0 (PageInfo): number=0
    assert_eq!(&output[13..17], &[0x00, 0x00, 0x00, 0x00]);
    // Segment #1 (GenericRegion): number=1
    assert_eq!(&output[43..47], &[0x00, 0x00, 0x00, 0x01]);

    // EndOfPage: number=2, EndOfFile: number=3
    let genreg_data_len = segment_data_length_at(&output, 43) as usize;
    let eop_offset = 54 + genreg_data_len;
    assert_eq!(
        &output[eop_offset..eop_offset + 4],
        &[0x00, 0x00, 0x00, 0x02]
    );
    let eof_offset = eop_offset + 11;
    assert_eq!(
        &output[eof_offset..eof_offset + 4],
        &[0x00, 0x00, 0x00, 0x03]
    );
}

// ---------------------------------------------------------------------------
// full_headers=false の構造テスト
// ---------------------------------------------------------------------------

/// full_headers=false ではFileHeaderとEndOfPage/EndOfFileがない。
#[test]
fn no_headers_structure() {
    let pix = white_1bpp(32, 32);
    let output = encode_generic(&pix, false, 300, 300, false).unwrap();

    // FileHeaderなし → いきなりSegment #0 (PageInfo)
    assert_eq!(segment_type_at(&output, 0), SEGMENT_PAGE_INFORMATION);
    assert_eq!(segment_data_length_at(&output, 0), 19);

    // Segment #1 (GenericRegion)
    assert_eq!(segment_type_at(&output, 30), SEGMENT_IMM_GENERIC_REGION);

    // EndOfPage/EndOfFileなし
    let genreg_data_len = segment_data_length_at(&output, 30) as usize;
    let expected_total = 30 + 11 + genreg_data_len;
    assert_eq!(output.len(), expected_total);
}

// ---------------------------------------------------------------------------
// TPGD（duplicate_line_removal）テスト
// ---------------------------------------------------------------------------

/// TPGD有効時、GenericRegionのフラグバイトにtpgdon=1が立つ。
#[test]
fn tpgdon_flag_set() {
    let pix = white_1bpp(32, 32);
    let output = encode_generic(&pix, true, 0, 0, true).unwrap();

    // GenericRegion flags byte at offset 71: bit3=tpgdon
    // mmr=0, gbtemplate=0, tpgdon=1 → 0b0000_1000 = 0x08
    assert_eq!(output[71], 0x08);
}

/// TPGD有無で出力が異なること。
///
/// 算術符号化の最終フラッシュやコンテキスト切替のオーバーヘッドにより、
/// 全白画像ではTPGD有効の方が大きくなりうる。サイズの大小ではなく
/// 出力が異なることだけを検証する。
#[test]
fn tpgd_produces_different_output() {
    let pix = white_1bpp(64, 64);

    let without_tpgd = encode_generic(&pix, false, 0, 0, false).unwrap();
    let with_tpgd = encode_generic(&pix, false, 0, 0, true).unwrap();

    assert_ne!(without_tpgd, with_tpgd);
}

// ---------------------------------------------------------------------------
// 解像度テスト
// ---------------------------------------------------------------------------

/// xres/yres=0のときPixの解像度が使われる。
#[test]
fn uses_pix_resolution_when_zero() {
    let mut pm = PixMut::new(32, 32, PixelDepth::Bit1).unwrap();
    pm.set_resolution(150, 200);
    let pix: Pix = pm.into();

    let output = encode_generic(&pix, true, 0, 0, false).unwrap();

    // PageInfo xres at offset 32..36, yres at 36..40
    assert_eq!(&output[32..36], &[0x00, 0x00, 0x00, 0x96]); // 150=0x96
    assert_eq!(&output[36..40], &[0x00, 0x00, 0x00, 0xC8]); // 200=0xC8
}

/// xres/yresを明示指定した場合はそちらが使われる。
#[test]
fn explicit_resolution_overrides_pix() {
    let mut pm = PixMut::new(32, 32, PixelDepth::Bit1).unwrap();
    pm.set_resolution(150, 200);
    let pix: Pix = pm.into();

    let output = encode_generic(&pix, true, 72, 72, false).unwrap();

    // PageInfo xres/yres = 72
    assert_eq!(&output[32..36], &[0x00, 0x00, 0x00, 0x48]); // 72=0x48
    assert_eq!(&output[36..40], &[0x00, 0x00, 0x00, 0x48]);
}

// ---------------------------------------------------------------------------
// エッジケース
// ---------------------------------------------------------------------------

/// 幅が32の倍数でない画像でもエンコードできること。
#[test]
fn non_32_aligned_width() {
    let pix = white_1bpp(33, 10);
    let output = encode_generic(&pix, false, 0, 0, false).unwrap();

    // GenericRegion header width=33
    // SegmentHeader(11) + PageInfo(19) = 30のオフセット
    // Segment #1 starts at 30, GenericRegion header starts at 41
    assert_eq!(&output[41..45], &[0x00, 0x00, 0x00, 0x21]); // width=33
    assert_eq!(&output[45..49], &[0x00, 0x00, 0x00, 0x0A]); // height=10
}

/// 全黒画像もエンコードできること。
#[test]
fn all_black_image() {
    let pix = black_1bpp(32, 32);
    let result = encode_generic(&pix, false, 0, 0, false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}

/// 縞模様画像（TPGD有効時に行が交互に異なる）。
#[test]
fn striped_image_with_tpgd() {
    let pix = striped_1bpp(64, 64);
    let result = encode_generic(&pix, false, 0, 0, true);
    assert!(result.is_ok());
}

/// 1x1の最小画像。
#[test]
fn minimal_1x1_image() {
    let pix = white_1bpp(1, 1);
    let result = encode_generic(&pix, true, 0, 0, false);
    assert!(result.is_ok());
}
