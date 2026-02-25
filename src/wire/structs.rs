use super::constants::JBIG2_FILE_MAGIC;

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
        let mut buf = Vec::with_capacity(13);
        buf.extend_from_slice(&JBIG2_FILE_MAGIC);
        let flags = u8::from(self.organisation_type) | (u8::from(self.unknown_n_pages) << 1);
        buf.push(flags);
        buf.extend_from_slice(&self.n_pages.to_be_bytes());
        buf
    }
}

impl PageInfo {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(19);
        buf.extend_from_slice(&self.width.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf.extend_from_slice(&self.xres.to_be_bytes());
        buf.extend_from_slice(&self.yres.to_be_bytes());
        let flags = u8::from(self.is_lossless)
            | (u8::from(self.contains_refinements) << 1)
            | (u8::from(self.default_pixel) << 2)
            | ((self.default_operator & 0x03) << 3)
            | (u8::from(self.aux_buffers) << 5)
            | (u8::from(self.operator_override) << 6);
        buf.push(flags);
        buf.extend_from_slice(&self.segment_flags.to_be_bytes());
        buf
    }
}

impl GenericRegion {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(26);
        buf.extend_from_slice(&self.width.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.push(self.comb_operator);
        let flags =
            u8::from(self.mmr) | ((self.gbtemplate & 0x03) << 1) | (u8::from(self.tpgdon) << 3);
        buf.push(flags);
        buf.push(self.a1x as u8);
        buf.push(self.a1y as u8);
        buf.push(self.a2x as u8);
        buf.push(self.a2y as u8);
        buf.push(self.a3x as u8);
        buf.push(self.a3y as u8);
        buf.push(self.a4x as u8);
        buf.push(self.a4y as u8);
        buf
    }
}

impl SymbolDict {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(18);
        // flags byte 1 (LE packed, bit0 = LSB):
        // bit0: sdhuff, bit1: sdrefagg, bit2-3: sdhuffdh, bit4-5: sdhuffdw,
        // bit6: sdhuffbmsize, bit7: sdhuffagginst
        let flags0 = u8::from(self.sdhuff)
            | (u8::from(self.sdrefagg) << 1)
            | ((self.sdhuffdh & 0x03) << 2)
            | ((self.sdhuffdw & 0x03) << 4)
            | (u8::from(self.sdhuffbmsize) << 6)
            | (u8::from(self.sdhuffagginst) << 7);
        buf.push(flags0);
        // flags byte 2:
        // bit0: bmcontext, bit1: bmcontextretained, bit2-3: sdtemplate,
        // bit4: sdrtemplate, bit5-7: reserved
        let flags1 = u8::from(self.bmcontext)
            | (u8::from(self.bmcontextretained) << 1)
            | ((self.sdtemplate & 0x03) << 2)
            | (u8::from(self.sdrtemplate) << 4);
        buf.push(flags1);
        buf.push(self.a1x as u8);
        buf.push(self.a1y as u8);
        buf.push(self.a2x as u8);
        buf.push(self.a2y as u8);
        buf.push(self.a3x as u8);
        buf.push(self.a3y as u8);
        buf.push(self.a4x as u8);
        buf.push(self.a4y as u8);
        buf.extend_from_slice(&self.exsyms.to_be_bytes());
        buf.extend_from_slice(&self.newsyms.to_be_bytes());
        buf
    }
}

impl TextRegion {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(19);
        buf.extend_from_slice(&self.width.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.push(self.comb_operator);
        // C++のビットフィールドレイアウト（LE packed）:
        // byte 1 (offset 17):
        //   bit0: sbcombop bit2 (sbcombop の上位ビット)
        //   bit1: sbdefpixel
        //   bit2-6: sbdsoffset (5 bits)
        //   bit7: sbrtemplate
        let flags0 = ((self.sbcombop >> 1) & 0x01)
            | (u8::from(self.sbdefpixel) << 1)
            | ((self.sbdsoffset & 0x1F) << 2)
            | (u8::from(self.sbrtemplate) << 7);
        buf.push(flags0);
        // byte 2 (offset 18):
        //   bit0: sbhuff
        //   bit1: sbrefine
        //   bit2-3: logsbstrips (2 bits)
        //   bit4-5: refcorner (2 bits)
        //   bit6: transposed
        //   bit7: sbcombop bit1 (sbcombop の下位ビット)
        let flags1 = u8::from(self.sbhuff)
            | (u8::from(self.sbrefine) << 1)
            | ((self.logsbstrips & 0x03) << 2)
            | ((self.refcorner & 0x03) << 4)
            | (u8::from(self.transposed) << 6)
            | ((self.sbcombop & 0x01) << 7);
        buf.push(flags1);
        buf
    }
}

impl TextRegionAtFlags {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.a1x as u8,
            self.a1y as u8,
            self.a2x as u8,
            self.a2y as u8,
        ]
    }
}

impl TextRegionSymInsts {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.sbnuminstances.to_be_bytes().to_vec()
    }
}
