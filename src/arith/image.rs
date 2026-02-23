use super::encoder::ArithEncoder;

/// TPGD（Typical Prediction for Generic Direct coding）用コンテキスト番号。
///
/// C++版の `TPGDCTX = 0x9b25` に対応。
#[allow(dead_code)]
const TPGD_CTX: u32 = 0x9b25;

impl ArithEncoder {
    /// パックド1bpp画像を算術符号化する。
    ///
    /// C++版 `jbig2enc_bitimage()` に対応。
    /// Leptonicaの1bppパックドフォーマット（各行は32ビットワード単位）を想定。
    ///
    /// `data`: パックド1bppビット列（`u32`スライスとして渡す）
    /// `mx`: 画像幅（ピクセル）
    /// `my`: 画像高さ（ピクセル）
    /// `duplicate_line_removal`: trueのときTPGDを使用
    ///
    /// *各行の末尾パッドビットはゼロでなければならない*
    pub fn encode_bitimage(
        &mut self,
        _data: &[u32],
        _mx: u32,
        _my: u32,
        _duplicate_line_removal: bool,
    ) {
        todo!()
    }

    /// リファインメント符号化（2画像間の差分を符号化）。
    ///
    /// C++版 `jbig2enc_refine()` に対応。
    /// テンプレート画像からターゲット画像への差分を13ピクセルテンプレートで符号化。
    ///
    /// `templ`: テンプレート画像（1bppパックド）
    /// `tx`, `ty`: テンプレートのサイズ
    /// `target`: ターゲット画像（1bppパックド）
    /// `mx`, `my`: ターゲットのサイズ
    /// `ox`: X軸オフセット（-1, 0, 1 のみ）
    /// `oy`: Y軸オフセット
    #[allow(clippy::too_many_arguments)]
    pub fn encode_refine(
        &mut self,
        _templ: &[u32],
        _tx: u32,
        _ty: u32,
        _target: &[u32],
        _mx: u32,
        _my: u32,
        _ox: i32,
        _oy: i32,
    ) {
        todo!()
    }
}
