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
    ///
    /// バイトレイアウト（C++版 `jbig2_segment` + 可変長フィールド）:
    /// - `[0..4]`  セグメント番号（BE u32）
    /// - `[4]`     フラグ: bit0-5=type, bit6=page_assoc_size, bit7=deferred_non_retain
    /// - `[5]`     参照カウント: bit0-4=retain_bits, bit5-7=segment_count
    /// - 参照先セグメント番号（各 reference_size バイト、BE）
    /// - ページ番号（page_size バイト、BE）
    /// - データ長（4バイト、BE u32）
    pub fn to_bytes(&self) -> Vec<u8> {
        let ref_size = self.reference_size();
        let pg_size = self.page_size();
        let mut buf = Vec::with_capacity(self.size());

        // segment number (BE u32)
        buf.extend_from_slice(&self.number.to_be_bytes());

        // flags byte: type(6) | page_assoc_size(1) | deferred_non_retain(1)
        let page_assoc_size_bit = if pg_size == 4 { 1u8 } else { 0u8 };
        let flags = (self.seg_type & 0x3F)
            | (page_assoc_size_bit << 6)
            | (u8::from(self.deferred_non_retain) << 7);
        buf.push(flags);

        // referred-to count byte: retain_bits(5) | segment_count(3)
        let count = self.referred_to.len() as u8;
        let referred_byte = (self.retain_bits & 0x1F) | ((count & 0x07) << 5);
        buf.push(referred_byte);

        // referred-to segment numbers
        for &seg_num in &self.referred_to {
            match ref_size {
                1 => buf.push(seg_num as u8),
                2 => buf.extend_from_slice(&(seg_num as u16).to_be_bytes()),
                4 => buf.extend_from_slice(&seg_num.to_be_bytes()),
                _ => unreachable!(),
            }
        }

        // page association
        match pg_size {
            1 => buf.push(self.page as u8),
            4 => buf.extend_from_slice(&self.page.to_be_bytes()),
            _ => unreachable!(),
        }

        // data length (BE u32)
        buf.extend_from_slice(&self.data_length.to_be_bytes());

        debug_assert_eq!(buf.len(), self.size());
        buf
    }
}
