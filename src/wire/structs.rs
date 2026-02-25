/// JBIG2ファイルヘッダ（13バイト）。
///
/// C++版 `jbig2_file_header` に対応。
///
/// レイアウト:
/// - `[0..8]`  マジックバイト
/// - `[8]`     フラグ（bit0: organisation_type, bit1: unknown_n_pages）
/// - `[9..13]` ページ数（BE u32）
pub struct FileHeader {
    pub organisation_type: bool,
    pub unknown_n_pages: bool,
    pub n_pages: u32,
}

/// JBIG2ページ情報セグメント（19バイト）。
///
/// C++版 `jbig2_page_info` に対応。
pub struct PageInfo {
    pub width: u32,
    pub height: u32,
    pub xres: u32,
    pub yres: u32,
    pub is_lossless: bool,
    pub contains_refinements: bool,
    pub default_pixel: bool,
    pub default_operator: u8,
    pub aux_buffers: bool,
    pub operator_override: bool,
    pub segment_flags: u16,
}

/// JBIG2ジェネリックリージョンセグメント（26バイト）。
///
/// C++版 `jbig2_generic_region` に対応。
pub struct GenericRegion {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub comb_operator: u8,
    pub mmr: bool,
    pub gbtemplate: u8,
    pub tpgdon: bool,
    pub a1x: i8,
    pub a1y: i8,
    pub a2x: i8,
    pub a2y: i8,
    pub a3x: i8,
    pub a3y: i8,
    pub a4x: i8,
    pub a4y: i8,
}

/// JBIG2シンボル辞書セグメント（18バイト）。
///
/// C++版 `jbig2_symbol_dict` に対応。
pub struct SymbolDict {
    pub sdhuff: bool,
    pub sdrefagg: bool,
    pub sdhuffdh: u8,
    pub sdhuffdw: u8,
    pub sdhuffbmsize: bool,
    pub sdhuffagginst: bool,
    pub bmcontext: bool,
    pub bmcontextretained: bool,
    pub sdtemplate: u8,
    pub sdrtemplate: bool,
    pub a1x: i8,
    pub a1y: i8,
    pub a2x: i8,
    pub a2y: i8,
    pub a3x: i8,
    pub a3y: i8,
    pub a4x: i8,
    pub a4y: i8,
    pub exsyms: u32,
    pub newsyms: u32,
}

/// JBIG2テキストリージョンセグメント（19バイト）。
///
/// C++版 `jbig2_text_region` に対応。
pub struct TextRegion {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub comb_operator: u8,
    pub sbhuff: bool,
    pub sbrefine: bool,
    pub logsbstrips: u8,
    pub refcorner: u8,
    pub transposed: bool,
    pub sbcombop: u8,
    pub sbdefpixel: bool,
    pub sbdsoffset: u8,
    pub sbrtemplate: bool,
}

/// JBIG2テキストリージョンATフラグ（4バイト）。
///
/// C++版 `jbig2_text_region_atflags` に対応。
pub struct TextRegionAtFlags {
    pub a1x: i8,
    pub a1y: i8,
    pub a2x: i8,
    pub a2y: i8,
}

/// JBIG2テキストリージョンシンボルインスタンス数（4バイト）。
///
/// C++版 `jbig2_text_region_syminsts` に対応。
pub struct TextRegionSymInsts {
    pub sbnuminstances: u32,
}

impl FileHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

impl PageInfo {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

impl GenericRegion {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

impl SymbolDict {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

impl TextRegion {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

impl TextRegionAtFlags {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

impl TextRegionSymInsts {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}
