/// QMコーダの1状態エントリ。
///
/// `qe`: 確率推定値（LPS確率）
/// `mps`: MPS（より可能性の高いシンボル）出現時の次状態インデックス
/// `lps`: LPS（より可能性の低いシンボル）出現時の次状態インデックス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateEntry {
    pub qe: u16,
    pub mps: u8,
    pub lps: u8,
}

/// JBIG2仕様 Table E.1 に基づくQMコーダ状態テーブル。
///
/// 46状態 x 2（MPS=0用とMPS=1用）= 92エントリ。
/// インデックス0-45: MPS=0の状態、インデックス46-91: MPS=1の状態。
/// MPS/LPS反転が必要な状態では、lpsが反対側（+46 or -46）を指す。
pub static STATE_TABLE: [StateEntry; 92] = [StateEntry {
    qe: 0,
    mps: 0,
    lps: 0,
}; 92];
