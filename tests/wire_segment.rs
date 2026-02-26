use jbig2enc::wire::SegmentHeader;

// ---------------------------------------------------------------------------
// SegmentHeader サイズ計算
// ---------------------------------------------------------------------------

/// reference_size: number ≤ 256 → 1バイト。
#[test]
fn segment_reference_size_1byte() {
    let seg = SegmentHeader {
        number: 0,
        seg_type: 0,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 1,
        data_length: 0,
    };
    assert_eq!(seg.reference_size(), 1);

    let seg256 = SegmentHeader { number: 256, ..seg };
    assert_eq!(seg256.reference_size(), 1);
}

/// reference_size: number ≤ 65536 → 2バイト。
#[test]
fn segment_reference_size_2bytes() {
    let seg = SegmentHeader {
        number: 257,
        seg_type: 0,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 1,
        data_length: 0,
    };
    assert_eq!(seg.reference_size(), 2);

    let seg65536 = SegmentHeader {
        number: 65536,
        ..seg
    };
    assert_eq!(seg65536.reference_size(), 2);
}

/// reference_size: number > 65536 → 4バイト。
#[test]
fn segment_reference_size_4bytes() {
    let seg = SegmentHeader {
        number: 65537,
        seg_type: 0,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 1,
        data_length: 0,
    };
    assert_eq!(seg.reference_size(), 4);
}

/// page_size: page ≤ 255 → 1バイト。
#[test]
fn segment_page_size_1byte() {
    let seg = SegmentHeader {
        number: 0,
        seg_type: 0,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 255,
        data_length: 0,
    };
    assert_eq!(seg.page_size(), 1);
}

/// page_size: page > 255 → 4バイト。
#[test]
fn segment_page_size_4bytes() {
    let seg = SegmentHeader {
        number: 0,
        seg_type: 0,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 256,
        data_length: 0,
    };
    assert_eq!(seg.page_size(), 4);
}

/// size(): 参照なし、1バイトページ → 6 + 0 + 1 + 4 = 11。
#[test]
fn segment_total_size_minimal() {
    let seg = SegmentHeader {
        number: 0,
        seg_type: 0,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 1,
        data_length: 100,
    };
    assert_eq!(seg.size(), 11);
}

/// size(): 2参照、2バイトref、4バイトページ → 6 + 2*2 + 4 + 4 = 18。
#[test]
fn segment_total_size_with_refs() {
    let seg = SegmentHeader {
        number: 300,
        seg_type: 6,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![1, 2],
        page: 1000,
        data_length: 500,
    };
    assert_eq!(seg.size(), 18);
}

// ---------------------------------------------------------------------------
// SegmentHeader to_bytes()
// ---------------------------------------------------------------------------

/// ページ情報セグメント（参照なし、page=1）の典型的なシリアライズ。
///
/// C++使用例（jbig2enc.cc:734-757）:
/// ```c
/// seg.number = ctx->segnum;  // e.g. 0
/// seg.type = segment_page_information;  // 48
/// seg.page = 1;
/// seg.len = sizeof(struct jbig2_page_info);  // 19
/// ```
///
/// jbig2_segment 構造体（6バイト）:
/// [0..4]  number: htonl(0) = 00 00 00 00
/// [4]     flags:
///         bit0-5: type=48 (0b110000)
///         bit6: page_assoc_size=0 (page ≤ 255)
///         bit7: deferred_non_retain=0
///         → 0b00_110000 = 0x30
/// [5]     referred count:
///         bit0-4: retain_bits=0
///         bit5-7: segment_count=0
///         → 0x00
///
/// referred_to: (none)
///
/// page: 1バイト → 0x01
///
/// data_length: htonl(19) = 00 00 00 13
#[test]

fn segment_page_info_simple() {
    let seg = SegmentHeader {
        number: 0,
        seg_type: 48, // segment_page_information
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 1,
        data_length: 19,
    };
    let bytes = seg.to_bytes();
    assert_eq!(bytes.len(), seg.size());
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x00, // number=0 BE
        0x30,                   // flags: type=48(0b110000), page_assoc_size=0, deferred=0
        0x00,                   // referred: retain=0, count=0
        0x01,                   // page=1 (1 byte)
        0x00, 0x00, 0x00, 0x13, // data_length=19 BE
    ];
    assert_eq!(bytes, expected);
}

/// シンボルテーブルセグメント（参照なし、page=0、retain_bits=1）。
///
/// C++使用例（jbig2enc.cc:680-699）:
/// ```c
/// seg.number = 0;
/// seg.type = segment_symbol_table;  // 0
/// seg.len = sizeof(symtab) + symdatasize;  // e.g. 18+100=118
/// seg.page = 0;
/// seg.retain_bits = 1;
/// ```
#[test]

fn segment_symbol_table() {
    let seg = SegmentHeader {
        number: 0,
        seg_type: 0, // segment_symbol_table
        deferred_non_retain: false,
        retain_bits: 1,
        referred_to: vec![],
        page: 0,
        data_length: 118,
    };
    let bytes = seg.to_bytes();
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x00, // number=0 BE
        0x00,                   // flags: type=0, page_assoc_size=0, deferred=0
        0x01,                   // referred: retain_bits=1(bit0), count=0
        0x00,                   // page=0 (1 byte)
        0x00, 0x00, 0x00, 0x76, // data_length=118 BE
    ];
    assert_eq!(bytes, expected);
}

/// テキストリージョンセグメント（1つの参照、retain_bits=2）。
///
/// C++使用例（jbig2enc.cc:791-813）:
/// ```c
/// segr.number = 2;
/// segr.type = segment_imm_text_region;  // 6
/// segr.referred_to.push_back(0);  // symtab_segment
/// segr.retain_bits = 2;
/// segr.page = 1;
/// segr.len = 500;
/// ```
#[test]

fn segment_text_region_one_ref() {
    let seg = SegmentHeader {
        number: 2,
        seg_type: 6, // segment_imm_text_region
        deferred_non_retain: false,
        retain_bits: 2,
        referred_to: vec![0],
        page: 1,
        data_length: 500,
    };
    let bytes = seg.to_bytes();
    // size: 6 + 1*1 + 1 + 4 = 12
    assert_eq!(bytes.len(), 12);
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x02, // number=2 BE
        0x06,                   // flags: type=6(0b000110), page_assoc_size=0, deferred=0
        0x22,                   // referred: retain_bits=2(bit1), count=1(bit5) → 0b001_00010
        0x00,                   // referred_to[0]=0 (1 byte ref)
        0x01,                   // page=1 (1 byte)
        0x00, 0x00, 0x01, 0xF4, // data_length=500 BE
    ];
    assert_eq!(bytes, expected);
}

/// テキストリージョン（2つの参照: symtab + extra_symtab）。
#[test]

fn segment_text_region_two_refs() {
    let seg = SegmentHeader {
        number: 3,
        seg_type: 6, // segment_imm_text_region
        deferred_non_retain: false,
        retain_bits: 2,
        referred_to: vec![0, 2], // symtab=0, extra_symtab=2
        page: 1,
        data_length: 800,
    };
    let bytes = seg.to_bytes();
    // size: 6 + 1*2 + 1 + 4 = 13
    assert_eq!(bytes.len(), 13);
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x03, // number=3 BE
        0x06,                   // flags: type=6
        0x42,                   // referred: retain_bits=2(bit1), count=2(bit5-6) → 0b010_00010
        0x00,                   // referred_to[0]=0
        0x02,                   // referred_to[1]=2
        0x01,                   // page=1
        0x00, 0x00, 0x03, 0x20, // data_length=800 BE
    ];
    assert_eq!(bytes, expected);
}

/// end-of-page セグメント（data_length=0）。
#[test]

fn segment_end_of_page() {
    let seg = SegmentHeader {
        number: 4,
        seg_type: 49, // segment_end_of_page
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 1,
        data_length: 0,
    };
    let bytes = seg.to_bytes();
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x04, // number=4 BE
        0x31,                   // flags: type=49(0b110001)
        0x00,                   // referred: retain=0, count=0
        0x01,                   // page=1
        0x00, 0x00, 0x00, 0x00, // data_length=0
    ];
    assert_eq!(bytes, expected);
}

/// end-of-file セグメント（page=0）。
#[test]

fn segment_end_of_file() {
    let seg = SegmentHeader {
        number: 5,
        seg_type: 51, // segment_end_of_file
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 0,
        data_length: 0,
    };
    let bytes = seg.to_bytes();
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x05, // number=5 BE
        0x33,                   // flags: type=51(0b110011)
        0x00,                   // referred: retain=0, count=0
        0x00,                   // page=0
        0x00, 0x00, 0x00, 0x00, // data_length=0
    ];
    assert_eq!(bytes, expected);
}

/// 大きなセグメント番号（> 256）で2バイトリファレンス。
#[test]

fn segment_large_number_2byte_ref() {
    let seg = SegmentHeader {
        number: 300,
        seg_type: 6,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![100],
        page: 1,
        data_length: 50,
    };
    let bytes = seg.to_bytes();
    // size: 6 + 2*1 + 1 + 4 = 13
    assert_eq!(bytes.len(), 13);
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x01, 0x2C, // number=300 BE
        0x06,                   // type=6
        0x20,                   // retain=0, count=1 → 0b001_00000
        0x00, 0x64,             // referred_to[0]=100 BE (2 bytes)
        0x01,                   // page=1
        0x00, 0x00, 0x00, 0x32, // data_length=50 BE
    ];
    assert_eq!(bytes, expected);
}

/// 4バイトページアソシエーション。
#[test]

fn segment_4byte_page() {
    let seg = SegmentHeader {
        number: 0,
        seg_type: 48,
        deferred_non_retain: false,
        retain_bits: 0,
        referred_to: vec![],
        page: 256,
        data_length: 19,
    };
    let bytes = seg.to_bytes();
    // size: 6 + 0 + 4 + 4 = 14
    assert_eq!(bytes.len(), 14);
    #[rustfmt::skip]
    let expected: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x00, // number=0
        0x70,                   // flags: type=48, page_assoc_size=1(bit6) → 0b01_110000
        0x00,                   // referred: retain=0, count=0
        0x00, 0x00, 0x01, 0x00, // page=256 BE (4 bytes)
        0x00, 0x00, 0x00, 0x13, // data_length=19
    ];
    assert_eq!(bytes, expected);
}
