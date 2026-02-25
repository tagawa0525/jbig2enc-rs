use leptonica::{Pix, PixelDepth};

use crate::arith::ArithEncoder;
use crate::error::Jbig2Error;
use crate::wire::{
    FileHeader, GenericRegion, PageInfo, SEGMENT_END_OF_FILE, SEGMENT_END_OF_PAGE,
    SEGMENT_IMM_GENERIC_REGION, SEGMENT_PAGE_INFORMATION, SegmentHeader,
};

/// 1bpp画像をJBIG2ジェネリックリージョンとして符号化する。
///
/// C++版 `jbig2_encode_generic()`（`jbig2enc.cc:898-1002`）に対応。
///
/// # Arguments
/// - `pix` - 1bpp入力画像
/// - `full_headers` - true: 完全なJBIG2ファイル、false: PDF埋め込み用断片
/// - `xres`/`yres` - 解像度（ppi）。0の場合はPix自身の解像度を使用
/// - `duplicate_line_removal` - TPGD（同一行スキップ最適化）を有効にするか
pub fn encode_generic(
    pix: &Pix,
    full_headers: bool,
    xres: u32,
    yres: u32,
    duplicate_line_removal: bool,
) -> Result<Vec<u8>, Jbig2Error> {
    // 入力検証: 1bpp のみ
    if pix.depth() != PixelDepth::Bit1 {
        return Err(Jbig2Error::InvalidInput(format!(
            "expected 1bpp image, got {}bpp",
            pix.depth().bits()
        )));
    }

    let w = pix.width();
    let h = pix.height();

    // 解像度: 0の場合はPix自身の値を使用（C++: xres ? xres : bw->xres）
    let xres = if xres != 0 {
        xres
    } else {
        pix.xres().max(0) as u32
    };
    let yres = if yres != 0 {
        yres
    } else {
        pix.yres().max(0) as u32
    };

    // パッドビットゼロ化（C++: pixSetPadBits(bw, 0)）
    let mut pix_mut = pix.to_mut();
    pix_mut.set_pad_bits(0);

    // 算術符号化
    let mut encoder = ArithEncoder::new();
    encoder.encode_bitimage(pix_mut.data(), w, h, duplicate_line_removal);
    encoder.encode_final();
    let encoded_data = encoder.to_vec();

    // セグメント組み立て
    let mut segnum: u32 = 0;
    let mut output = Vec::new();

    // FileHeader（full_headers時のみ）
    if full_headers {
        let header = FileHeader {
            organisation_type: true,
            unknown_n_pages: false,
            n_pages: 1,
        };
        output.extend_from_slice(&header.to_bytes());
    }

    // Segment #0: PageInfo
    let page_info = PageInfo {
        width: w,
        height: h,
        xres,
        yres,
        is_lossless: true,
        contains_refinements: false,
        default_pixel: false,
        default_operator: 0,
        aux_buffers: false,
        operator_override: false,
        segment_flags: 0,
    };
    let page_info_bytes = page_info.to_bytes();

    let seg0 = SegmentHeader {
        number: segnum,
        seg_type: SEGMENT_PAGE_INFORMATION,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 1,
        data_length: page_info_bytes.len() as u32,
    };
    output.extend_from_slice(&seg0.to_bytes());
    output.extend_from_slice(&page_info_bytes);
    segnum += 1;

    // Segment #1: GenericRegion
    let genreg = GenericRegion {
        width: w,
        height: h,
        x: 0,
        y: 0,
        comb_operator: 0,
        mmr: false,
        gbtemplate: 0,
        tpgdon: duplicate_line_removal,
        a1x: 3,
        a1y: -1,
        a2x: -3,
        a2y: -1,
        a3x: 2,
        a3y: -2,
        a4x: -2,
        a4y: -2,
    };
    let genreg_bytes = genreg.to_bytes();
    let genreg_data_length = genreg_bytes.len() as u32 + encoded_data.len() as u32;

    let seg1 = SegmentHeader {
        number: segnum,
        seg_type: SEGMENT_IMM_GENERIC_REGION,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 1,
        data_length: genreg_data_length,
    };
    output.extend_from_slice(&seg1.to_bytes());
    output.extend_from_slice(&genreg_bytes);
    output.extend_from_slice(&encoded_data);
    segnum += 1;

    // EndOfPage + EndOfFile（full_headers時のみ）
    if full_headers {
        let seg_eop = SegmentHeader {
            number: segnum,
            seg_type: SEGMENT_END_OF_PAGE,
            deferred_non_retain: false,
            retain_bits: 0,
            referred_to: vec![],
            page: 1,
            data_length: 0,
        };
        output.extend_from_slice(&seg_eop.to_bytes());
        segnum += 1;

        let seg_eof = SegmentHeader {
            number: segnum,
            seg_type: SEGMENT_END_OF_FILE,
            deferred_non_retain: false,
            retain_bits: 0,
            referred_to: vec![],
            page: 0,
            data_length: 0,
        };
        output.extend_from_slice(&seg_eof.to_bytes());
    }

    Ok(output)
}
