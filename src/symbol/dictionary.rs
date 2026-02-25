use std::collections::HashMap;

use leptonica::Pix;

use crate::error::Jbig2Error;

/// シンボルテーブルの算術符号化結果。
pub struct SymbolTableResult {
    /// 算術符号化されたデータ
    pub data: Vec<u8>,
    /// 元のシンボルインデックス → 符号化番号のマッピング
    pub symmap: HashMap<usize, usize>,
}

/// シンボル辞書を算術符号化する。
///
/// C++版 `jbig2enc_symboltable()`（`jbig2sym.cc:91-180`）に対応。
///
/// シンボルを高さ順→幅順にソートし、デルタ高さ/幅を整数符号化、
/// 各シンボルビットマップを算術符号化する。
///
/// # Arguments
/// - `symbols` - シンボルテンプレート配列
/// - `symbol_indices` - 符号化するシンボルのインデックス（`symbols` へのインデックス）
/// - `unborder` - true→各シンボルから `border_size` ピクセルのボーダーを除去して符号化
/// - `border_size` - ボーダーサイズ（`unborder=true` の場合のみ使用）
pub fn encode_symbol_table(
    _symbols: &[Pix],
    _symbol_indices: &[usize],
    _unborder: bool,
    _border_size: u32,
) -> Result<SymbolTableResult, Jbig2Error> {
    todo!()
}
