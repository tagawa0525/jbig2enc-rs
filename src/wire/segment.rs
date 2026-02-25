/// JBIG2セグメントヘッダ（可変長）。
///
/// C++版 `Segment` クラス（`jbig2segments.h`）に対応。
///
/// セグメントヘッダは参照先セグメント数やページ番号によってサイズが変動する。
/// `to_bytes()` で完全なバイナリ表現にシリアライズする。
pub struct SegmentHeader {
    pub number: u32,
    pub seg_type: u8,
    pub deferred_non_retain: bool,
    pub retain_bits: u8,
    pub referred_to: Vec<u32>,
    pub page: u32,
    pub data_length: u32,
}

impl SegmentHeader {
    /// 参照先セグメント番号のサイズ（バイト数）を返す。
    ///
    /// JBIG2仕様 7.2.5: セグメントは自分より若い番号のセグメントしか
    /// 参照できないため、自身の番号でサイズが決まる。
    pub fn reference_size(&self) -> usize {
        if self.number <= 256 {
            1
        } else if self.number <= 65536 {
            2
        } else {
            4
        }
    }

    /// ページ番号フィールドのサイズ（バイト数）を返す。
    ///
    /// JBIG2仕様 7.2.6。
    pub fn page_size(&self) -> usize {
        if self.page <= 255 { 1 } else { 4 }
    }

    /// シリアライズ後のバイト数を返す。
    pub fn size(&self) -> usize {
        6 + self.reference_size() * self.referred_to.len() + self.page_size() + 4
    }

    /// セグメントヘッダをバイト列にシリアライズする。
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }
}
