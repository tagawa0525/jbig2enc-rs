use jbig2enc_rs::wire::{
    FileHeader, GenericRegion, PageInfo, SymbolDict, TextRegion, TextRegionAtFlags,
    TextRegionSymInsts,
};

// ---------------------------------------------------------------------------
// FileHeader (13 bytes)
// ---------------------------------------------------------------------------

/// ジェネリックリージョン符号化時のファイルヘッダ（jbig2enc.cc:908-914）。
///
/// ```c
/// header.n_pages = htonl(1);
/// header.organisation_type = 1;
/// memcpy(&header.id, JBIG2_FILE_MAGIC, 8);
/// ```
///
/// バイトレイアウト:
/// [0..8]  = magic: 97 4a 42 32 0d 0a 1a 0a
/// [8]     = flags: bit0=organisation_type(1), bit1=unknown_n_pages(0), bit2-7=reserved(0)
///           → 0b0000_0001 = 0x01
/// [9..13] = n_pages: htonl(1) = 00 00 00 01
#[test]

fn file_header_generic_single_page() {
    let header = FileHeader {
        organisation_type: true,
        unknown_n_pages: false,
        n_pages: 1,
    };
    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), 13);
    #[rustfmt::skip]
    let expected: [u8; 13] = [
        0x97, 0x4a, 0x42, 0x32, 0x0d, 0x0a, 0x1a, 0x0a, // magic
        0x01,                                             // flags
        0x00, 0x00, 0x00, 0x01,                           // n_pages BE
    ];
    assert_eq!(bytes, expected);
}

/// マルチページ時のファイルヘッダ（jbig2enc.cc:672-678）。
/// n_pages = 3 の場合。
#[test]

fn file_header_multipage() {
    let header = FileHeader {
        organisation_type: true,
        unknown_n_pages: false,
        n_pages: 3,
    };
    let bytes = header.to_bytes();
    #[rustfmt::skip]
    let expected: [u8; 13] = [
        0x97, 0x4a, 0x42, 0x32, 0x0d, 0x0a, 0x1a, 0x0a, // magic
        0x01,                                             // flags
        0x00, 0x00, 0x00, 0x03,                           // n_pages BE
    ];
    assert_eq!(bytes, expected);
}

/// unknown_n_pages フラグ。
#[test]

fn file_header_unknown_pages() {
    let header = FileHeader {
        organisation_type: true,
        unknown_n_pages: true,
        n_pages: 0,
    };
    let bytes = header.to_bytes();
    // flags byte: bit0=1(org_type), bit1=1(unknown_n_pages) → 0b0000_0011 = 0x03
    assert_eq!(bytes[8], 0x03);
}

// ---------------------------------------------------------------------------
// PageInfo (19 bytes)
// ---------------------------------------------------------------------------

/// ジェネリックリージョン符号化時のページ情報（jbig2enc.cc:921-935）。
///
/// ```c
/// pageinfo.width = htonl(bw->w);   // e.g. 640
/// pageinfo.height = htonl(bw->h);  // e.g. 480
/// pageinfo.xres = htonl(xres);     // e.g. 300
/// pageinfo.yres = htonl(yres);     // e.g. 300
/// pageinfo.is_lossless = 1;
/// ```
///
/// バイトレイアウト:
/// [0..4]   width  BE: 00 00 02 80
/// [4..8]   height BE: 00 00 01 E0
/// [8..12]  xres   BE: 00 00 01 2C
/// [12..16] yres   BE: 00 00 01 2C
/// [16]     flags byte:
///          bit0=is_lossless(1), bit1=contains_refinements(0), bit2=default_pixel(0),
///          bit3-4=default_operator(0), bit5=aux_buffers(0), bit6=operator_override(0),
///          bit7=reserved(0)
///          → 0b0000_0001 = 0x01
/// [17..19] segment_flags: 0x0000
#[test]

fn page_info_generic_lossless() {
    let info = PageInfo {
        width: 640,
        height: 480,
        xres: 300,
        yres: 300,
        is_lossless: true,
        contains_refinements: false,
        default_pixel: false,
        default_operator: 0,
        aux_buffers: false,
        operator_override: false,
        segment_flags: 0,
    };
    let bytes = info.to_bytes();
    assert_eq!(bytes.len(), 19);
    #[rustfmt::skip]
    let expected: [u8; 19] = [
        0x00, 0x00, 0x02, 0x80, // width=640
        0x00, 0x00, 0x01, 0xE0, // height=480
        0x00, 0x00, 0x01, 0x2C, // xres=300
        0x00, 0x00, 0x01, 0x2C, // yres=300
        0x01,                   // flags: is_lossless=1
        0x00, 0x00,             // segment_flags
    ];
    assert_eq!(bytes, expected);
}

/// テキストリージョン用ページ情報（contains_refinements=true）。
#[test]

fn page_info_with_refinement() {
    let info = PageInfo {
        width: 1024,
        height: 768,
        xres: 72,
        yres: 72,
        is_lossless: true,
        contains_refinements: true,
        default_pixel: false,
        default_operator: 0,
        aux_buffers: false,
        operator_override: false,
        segment_flags: 0,
    };
    let bytes = info.to_bytes();
    // flags: bit0=1(is_lossless), bit1=1(contains_refinements) → 0x03
    assert_eq!(bytes[16], 0x03);
}

// ---------------------------------------------------------------------------
// GenericRegion (26 bytes)
// ---------------------------------------------------------------------------

/// ジェネリックリージョン（jbig2enc.cc:923-967 の典型的な使用）。
///
/// ```c
/// genreg.width = htonl(640);
/// genreg.height = htonl(480);
/// genreg.tpgdon = true;
/// genreg.a1x = 3; genreg.a1y = -1;
/// genreg.a2x = -3; genreg.a2y = -1;
/// genreg.a3x = 2; genreg.a3y = -2;
/// genreg.a4x = -2; genreg.a4y = -2;
/// ```
///
/// バイトレイアウト:
/// [0..4]   width:  00 00 02 80
/// [4..8]   height: 00 00 01 E0
/// [8..12]  x: 00 00 00 00
/// [12..16] y: 00 00 00 00
/// [16]     comb_operator: 0x00
/// [17]     flags byte:
///          bit0=mmr(0), bit1-2=gbtemplate(0), bit3=tpgdon(1), bit4-7=reserved(0)
///          → 0b0000_1000 = 0x08
/// [18..26] AT flags: 03 FF FD FF 02 FE FE FE
#[test]

fn generic_region_tpgd_on() {
    let region = GenericRegion {
        width: 640,
        height: 480,
        x: 0,
        y: 0,
        comb_operator: 0,
        mmr: false,
        gbtemplate: 0,
        tpgdon: true,
        a1x: 3,
        a1y: -1,
        a2x: -3,
        a2y: -1,
        a3x: 2,
        a3y: -2,
        a4x: -2,
        a4y: -2,
    };
    let bytes = region.to_bytes();
    assert_eq!(bytes.len(), 26);
    #[rustfmt::skip]
    let expected: [u8; 26] = [
        0x00, 0x00, 0x02, 0x80, // width=640
        0x00, 0x00, 0x01, 0xE0, // height=480
        0x00, 0x00, 0x00, 0x00, // x=0
        0x00, 0x00, 0x00, 0x00, // y=0
        0x00,                   // comb_operator=0
        0x08,                   // flags: tpgdon=1 at bit3
        0x03, 0xFF,             // a1x=3, a1y=-1
        0xFD, 0xFF,             // a2x=-3, a2y=-1
        0x02, 0xFE,             // a3x=2, a3y=-2
        0xFE, 0xFE,             // a4x=-2, a4y=-2
    ];
    assert_eq!(bytes, expected);
}

/// TPGD無効のジェネリックリージョン。
#[test]

fn generic_region_tpgd_off() {
    let region = GenericRegion {
        width: 100,
        height: 200,
        x: 0,
        y: 0,
        comb_operator: 0,
        mmr: false,
        gbtemplate: 0,
        tpgdon: false,
        a1x: 3,
        a1y: -1,
        a2x: -3,
        a2y: -1,
        a3x: 2,
        a3y: -2,
        a4x: -2,
        a4y: -2,
    };
    let bytes = region.to_bytes();
    // flags byte: all bits 0
    assert_eq!(bytes[17], 0x00);
}

// ---------------------------------------------------------------------------
// SymbolDict (18 bytes)
// ---------------------------------------------------------------------------

/// シンボル辞書（jbig2enc.cc:681-697 の典型的な使用）。
///
/// ```c
/// symtab.a1x = 3; symtab.a1y = -1;
/// symtab.a2x = -3; symtab.a2y = -1;
/// symtab.a3x = 2; symtab.a3y = -2;
/// symtab.a4x = -2; symtab.a4y = -2;
/// symtab.exsyms = symtab.newsyms = htonl(42);
/// ```
///
/// バイトレイアウト:
/// [0..2]   flags: 全ビット0 → 0x00 0x00
/// [2..10]  AT flags: 03 FF FD FF 02 FE FE FE
/// [10..14] exsyms:  00 00 00 2A (BE)
/// [14..18] newsyms: 00 00 00 2A (BE)
#[test]

fn symbol_dict_typical() {
    let dict = SymbolDict {
        sdhuff: false,
        sdrefagg: false,
        sdhuffdh: 0,
        sdhuffdw: 0,
        sdhuffbmsize: false,
        sdhuffagginst: false,
        bmcontext: false,
        bmcontextretained: false,
        sdtemplate: 0,
        sdrtemplate: false,
        a1x: 3,
        a1y: -1,
        a2x: -3,
        a2y: -1,
        a3x: 2,
        a3y: -2,
        a4x: -2,
        a4y: -2,
        exsyms: 42,
        newsyms: 42,
    };
    let bytes = dict.to_bytes();
    assert_eq!(bytes.len(), 18);
    #[rustfmt::skip]
    let expected: [u8; 18] = [
        0x00, 0x00,             // flags: all zero
        0x03, 0xFF,             // a1x=3, a1y=-1
        0xFD, 0xFF,             // a2x=-3, a2y=-1
        0x02, 0xFE,             // a3x=2, a3y=-2
        0xFE, 0xFE,             // a4x=-2, a4y=-2
        0x00, 0x00, 0x00, 0x2A, // exsyms=42 BE
        0x00, 0x00, 0x00, 0x2A, // newsyms=42 BE
    ];
    assert_eq!(bytes, expected);
}

// ---------------------------------------------------------------------------
// TextRegion (19 bytes)
// ---------------------------------------------------------------------------

/// テキストリージョン（jbig2enc.cc:738-813 の典型的な使用）。
///
/// ```c
/// textreg.width = htonl(1024);
/// textreg.height = htonl(768);
/// textreg.logsbstrips = 0;
/// textreg.sbrefine = 0; // no refinement
/// ```
///
/// バイトレイアウト:
/// [0..4]   width:  00 00 04 00
/// [4..8]   height: 00 00 03 00
/// [8..12]  x: 00 00 00 00
/// [12..16] y: 00 00 00 00
/// [16]     comb_operator: 0x00
/// [17]     flags byte 1 (LE packed):
///          bit0=sbcombop2(0), bit1=sbdefpixel(0), bit2-6=sbdsoffset(0),
///          bit7=sbrtemplate(0)
///          → 0x00
/// [18]     flags byte 2 (LE packed):
///          bit0=sbhuff(0), bit1=sbrefine(0), bit2-3=logsbstrips(0),
///          bit4-5=refcorner(0), bit6=transposed(0), bit7=sbcombop1(0)
///          → 0x00
#[test]

fn text_region_no_refinement() {
    let region = TextRegion {
        width: 1024,
        height: 768,
        x: 0,
        y: 0,
        comb_operator: 0,
        sbhuff: false,
        sbrefine: false,
        logsbstrips: 0,
        refcorner: 0,
        transposed: false,
        sbcombop: 0,
        sbdefpixel: false,
        sbdsoffset: 0,
        sbrtemplate: false,
    };
    let bytes = region.to_bytes();
    assert_eq!(bytes.len(), 19);
    #[rustfmt::skip]
    let expected: [u8; 19] = [
        0x00, 0x00, 0x04, 0x00, // width=1024
        0x00, 0x00, 0x03, 0x00, // height=768
        0x00, 0x00, 0x00, 0x00, // x=0
        0x00, 0x00, 0x00, 0x00, // y=0
        0x00,                   // comb_operator=0
        0x00,                   // flags byte 1
        0x00,                   // flags byte 2
    ];
    assert_eq!(bytes, expected);
}

/// リファインメント有りのテキストリージョン。
#[test]

fn text_region_with_refinement() {
    let region = TextRegion {
        width: 800,
        height: 600,
        x: 0,
        y: 0,
        comb_operator: 0,
        sbhuff: false,
        sbrefine: true,
        logsbstrips: 0,
        refcorner: 0,
        transposed: false,
        sbcombop: 0,
        sbdefpixel: false,
        sbdsoffset: 0,
        sbrtemplate: false,
    };
    let bytes = region.to_bytes();
    // flags byte 2: bit1=sbrefine(1) → 0b0000_0010 = 0x02
    assert_eq!(bytes[18], 0x02);
}

// ---------------------------------------------------------------------------
// TextRegionAtFlags (4 bytes)
// ---------------------------------------------------------------------------

/// テキストリージョンATフラグ（jbig2enc.cc:809-812）。
///
/// ```c
/// textreg_atflags.a1x = -1; textreg_atflags.a1y = -1;
/// textreg_atflags.a2x = -1; textreg_atflags.a2y = -1;
/// ```
#[test]

fn text_region_at_flags() {
    let flags = TextRegionAtFlags {
        a1x: -1,
        a1y: -1,
        a2x: -1,
        a2y: -1,
    };
    let bytes = flags.to_bytes();
    assert_eq!(bytes.len(), 4);
    assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF]);
}

// ---------------------------------------------------------------------------
// TextRegionSymInsts (4 bytes)
// ---------------------------------------------------------------------------

/// テキストリージョンシンボルインスタンス数。
///
/// ```c
/// textreg_syminsts.sbnuminstances = htonl(256);
/// ```
#[test]

fn text_region_sym_insts() {
    let insts = TextRegionSymInsts {
        sbnuminstances: 256,
    };
    let bytes = insts.to_bytes();
    assert_eq!(bytes.len(), 4);
    assert_eq!(bytes, [0x00, 0x00, 0x01, 0x00]);
}
