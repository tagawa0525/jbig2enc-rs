/// JBIG2ファイルマジックバイト（8バイト）。
///
/// C++版の `JBIG2_FILE_MAGIC` に対応。
pub const JBIG2_FILE_MAGIC: [u8; 8] = [0x97, 0x4a, 0x42, 0x32, 0x0d, 0x0a, 0x1a, 0x0a];

/// セグメントタイプ定数。
///
/// C++版 `jbig2structs.h` の enum に対応。
pub const SEGMENT_SYMBOL_TABLE: u8 = 0;
pub const SEGMENT_IMM_TEXT_REGION: u8 = 6;
pub const SEGMENT_IMM_GENERIC_REGION: u8 = 38;
pub const SEGMENT_PAGE_INFORMATION: u8 = 48;
pub const SEGMENT_END_OF_PAGE: u8 = 49;
pub const SEGMENT_END_OF_FILE: u8 = 51;
