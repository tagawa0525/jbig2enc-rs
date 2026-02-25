use std::collections::HashMap;

use leptonica::Pix;

use crate::error::Jbig2Error;

/// テキストリージョンに配置するシンボルインスタンス。
pub struct SymbolInstance {
    /// X座標（左端）
    pub x: i32,
    /// Y座標（下端、lower-left convention）
    pub y: i32,
    /// シンボルクラスID（シンボル辞書のインデックス）
    pub class_id: usize,
}

/// テキストリージョン符号化の設定。
pub struct TextRegionConfig<'a> {
    /// グローバル辞書のマッピング（class_id → encoded_id）
    pub symmap: &'a HashMap<usize, usize>,
    /// ページ固有辞書のマッピング（class_id → encoded_id）。Optional。
    pub symmap2: Option<&'a HashMap<usize, usize>>,
    /// グローバル辞書のシンボル数（symmap2のオフセット計算用）
    pub global_sym_count: usize,
    /// シンボルID符号化に必要なビット数（log2(total_symbols) の切り上げ）
    pub symbits: u32,
    /// ストリップ高さ（1, 2, 4, 8 のいずれか）
    pub strip_width: u32,
    /// true → シンボル幅計算時にボーダーを除去
    pub unborder: bool,
    /// ボーダーサイズ（unborder=true の場合のみ使用）
    pub border_size: u32,
}

/// テキストリージョンの算術符号化結果。
pub struct TextRegionResult {
    /// 算術符号化されたデータ
    pub data: Vec<u8>,
}

/// テキストリージョンを算術符号化する。
///
/// C++版 `jbig2enc_textregion()`（`jbig2sym.cc:218-461`）の非リファインメントパスに対応。
///
/// # Arguments
/// - `instances` - 配置するシンボルインスタンスの配列
/// - `symbols` - シンボルテンプレート配列（幅取得用）
/// - `cfg` - 符号化設定
pub fn encode_text_region(
    _instances: &[SymbolInstance],
    _symbols: &[Pix],
    _cfg: &TextRegionConfig<'_>,
) -> Result<TextRegionResult, Jbig2Error> {
    todo!()
}
