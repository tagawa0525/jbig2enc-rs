use super::encoder::{ArithEncoder, encode_bit_raw};

/// TPGD（Typical Prediction for Generic Direct coding）用コンテキスト番号。
///
/// C++版の `TPGDCTX = 0x9b25` に対応。
const TPGD_CTX: usize = 0x9b25;

impl ArithEncoder {
    /// パックド1bpp画像を算術符号化する。
    ///
    /// C++版 `jbig2enc_bitimage()` に対応。
    /// Leptonicaの1bppパックドフォーマット（各行は32ビットワード単位）を想定。
    ///
    /// Template 0、デフォルトAT位置を使用。
    /// コンテキストビット構成: c1(5bit, y-2行) | c2(7bit, y-1行) | c3(4bit, 現在行)
    ///
    /// `data`: パックド1bppビット列（`u32`スライスとして渡す）
    /// `mx`: 画像幅（ピクセル）
    /// `my`: 画像高さ（ピクセル）
    /// `duplicate_line_removal`: trueのときTPGDを使用
    ///
    /// *各行の末尾パッドビットはゼロでなければならない*
    pub fn encode_bitimage(
        &mut self,
        data: &[u32],
        mx: u32,
        my: u32,
        duplicate_line_removal: bool,
    ) {
        let words_per_row = mx.div_ceil(32) as usize;

        let mut ltp: u8 = 0;
        let mut sltp: u8 = 0;

        for y in 0..my as usize {
            // コンテキスト用ワード（y-2行、y-1行、現在行）
            let mut w1: u32 = if y >= 2 {
                data[(y - 2) * words_per_row]
            } else {
                0
            };
            let mut w2: u32 = if y >= 1 {
                data[(y - 1) * words_per_row]
            } else {
                0
            };

            if y >= 1 && duplicate_line_removal {
                // 前行と現在行が同じかチェック
                let same = data[y * words_per_row..y * words_per_row + words_per_row]
                    == data[(y - 1) * words_per_row..(y - 1) * words_per_row + words_per_row];
                if same {
                    sltp = ltp ^ 1;
                    ltp = 1;
                } else {
                    sltp = ltp;
                    ltp = 0;
                }
            }

            if duplicate_line_removal {
                let ctx = &mut self.context;
                encode_bit_raw(
                    &mut self.c,
                    &mut self.a,
                    &mut self.ct,
                    &mut self.b,
                    &mut self.bp,
                    &mut self.output_chunks,
                    &mut self.outbuf,
                    ctx,
                    TPGD_CTX,
                    sltp,
                );
                if ltp != 0 {
                    continue;
                }
            }

            let mut w3: u32 = data[y * words_per_row];

            // 初期コンテキストビット（各行の最初のピクセル前に先読みしてある分）
            // c1: y-2行目から3ビット（上から3列）
            // c2: y-1行目から4ビット（上から4列）
            let mut c1: u16 = (w1 >> 29) as u16; // 3ビット
            let mut c2: u16 = (w2 >> 28) as u16; // 4ビット
            let mut c3: u16 = 0; // 現在行の過去4ビット

            // 使用済みビットをシフトアウト
            w1 <<= 3;
            w2 <<= 4;

            for x in 0..mx as usize {
                let tval = ((c1 << 11) | (c2 << 4) | c3) as usize;
                let v = ((w3 & 0x8000_0000) >> 31) as u8;

                {
                    let ctx = &mut self.context;
                    encode_bit_raw(
                        &mut self.c,
                        &mut self.a,
                        &mut self.ct,
                        &mut self.b,
                        &mut self.bp,
                        &mut self.output_chunks,
                        &mut self.outbuf,
                        ctx,
                        tval,
                        v,
                    );
                }

                // コンテキストビットをシフト
                c1 = (c1 << 1) & 0x1f;
                c2 = (c2 << 1) & 0x7f;
                c3 = ((c3 << 1) | v as u16) & 0x0f;

                let m = x % 32;

                // w1: y-2行目ワードのロールイン（28ビット目でリロード）
                if m == 28 && y >= 2 {
                    let wordno = x / 32 + 1;
                    w1 = if wordno < words_per_row {
                        data[(y - 2) * words_per_row + wordno]
                    } else {
                        0
                    };
                } else {
                    w1 = w1.wrapping_shl(1);
                }
                c1 |= (w1 >> 31) as u16;

                // w2: y-1行目ワードのロールイン（27ビット目でリロード）
                if m == 27 && y >= 1 {
                    let wordno = x / 32 + 1;
                    w2 = if wordno < words_per_row {
                        data[(y - 1) * words_per_row + wordno]
                    } else {
                        0
                    };
                } else {
                    w2 = w2.wrapping_shl(1);
                }
                c2 |= (w2 >> 31) as u16;

                // w3: 現在行ワードのロールイン（31ビット目でリロード）
                if m == 31 {
                    let wordno = x / 32 + 1;
                    w3 = if wordno < words_per_row {
                        data[y * words_per_row + wordno]
                    } else {
                        0
                    };
                } else {
                    w3 <<= 1;
                }
            }
        }
    }

    /// リファインメント符号化（2画像間の差分を符号化）。
    ///
    /// C++版 `jbig2enc_refine()` に対応。
    /// テンプレート画像からターゲット画像への差分を13ピクセルテンプレートで符号化。
    ///
    /// コンテキストビット構成:
    /// c1(3bit, templ y-1行) | c2(3bit, templ y行) | c3(3bit, templ y+1行) | c4(3bit, target y-1行) | c5(1bit, 現在ピクセルの左)
    ///
    /// `ox` は -1, 0, 1 のみ有効。
    #[allow(clippy::too_many_arguments)]
    pub fn encode_refine(
        &mut self,
        templ: &[u32],
        tx: u32,
        ty: u32,
        target: &[u32],
        mx: u32,
        my: u32,
        ox: i32,
        oy: i32,
    ) {
        let templwords_per_row = tx.div_ceil(32) as usize;
        let words_per_row = mx.div_ceil(32) as usize;

        for y in 0..my as usize {
            let temply = y as i32 + oy;

            let mut w1: u32 = if temply >= 1 && temply - 1 < ty as i32 {
                templ[(temply as usize - 1) * templwords_per_row]
            } else {
                0
            };
            let mut w2: u32 = if temply >= 0 && temply < ty as i32 {
                templ[temply as usize * templwords_per_row]
            } else {
                0
            };
            let mut w3: u32 = if temply >= -1 && temply + 1 < ty as i32 {
                templ[(temply as usize + 1) * templwords_per_row]
            } else {
                0
            };

            let mut w4: u32 = if y >= 1 {
                target[(y - 1) * words_per_row]
            } else {
                0
            };
            let mut w5: u32 = target[y * words_per_row];

            let shiftoffset = (30 + ox) as u32;
            let mut c1: u16 = (w1 >> shiftoffset) as u16 & 3;
            let mut c2: u16 = (w2 >> shiftoffset) as u16 & 3;
            let mut c3: u16 = (w3 >> shiftoffset) as u16 & 3;
            let mut c4: u16 = (w4 >> 30) as u16 & 3;
            let mut c5: u16 = 0;

            let bits_to_trim = (2 - ox) as u32;
            w1 <<= bits_to_trim;
            w2 <<= bits_to_trim;
            w3 <<= bits_to_trim;
            w4 <<= 2;

            for x in 0..mx as usize {
                let tval = ((c1 << 10) | (c2 << 7) | (c3 << 4) | (c4 << 1) | c5) as usize;
                let v = (w5 >> 31) as u8;

                {
                    let ctx = &mut self.context;
                    encode_bit_raw(
                        &mut self.c,
                        &mut self.a,
                        &mut self.ct,
                        &mut self.b,
                        &mut self.bp,
                        &mut self.output_chunks,
                        &mut self.outbuf,
                        ctx,
                        tval,
                        v,
                    );
                }

                c1 = ((c1 << 1) | (w1 >> 31) as u16) & 7;
                c2 = ((c2 << 1) | (w2 >> 31) as u16) & 7;
                c3 = ((c3 << 1) | (w3 >> 31) as u16) & 7;
                c4 = ((c4 << 1) | (w4 >> 31) as u16) & 7;
                c5 = v as u16;

                let m = x % 32;
                let wordno = x / 32 + 1;

                // テンプレートワードのロールイン
                if m == (29 + ox) as usize {
                    w1 = if wordno < templwords_per_row && temply >= 1 && temply - 1 < ty as i32 {
                        templ[(temply as usize - 1) * templwords_per_row + wordno]
                    } else {
                        0
                    };
                    w2 = if wordno < templwords_per_row && temply >= 0 && temply < ty as i32 {
                        templ[temply as usize * templwords_per_row + wordno]
                    } else {
                        0
                    };
                    w3 = if wordno < templwords_per_row && temply >= -1 && temply + 1 < ty as i32 {
                        templ[(temply as usize + 1) * templwords_per_row + wordno]
                    } else {
                        0
                    };
                } else {
                    w1 <<= 1;
                    w2 <<= 1;
                    w3 <<= 1;
                }

                // ターゲット前行ワードのロールイン
                if m == 29 && y >= 1 {
                    w4 = if wordno < words_per_row {
                        target[(y - 1) * words_per_row + wordno]
                    } else {
                        0
                    };
                } else {
                    w4 <<= 1;
                }

                // 現在行ワードのロールイン
                if m == 31 {
                    w5 = if wordno < words_per_row {
                        target[y * words_per_row + wordno]
                    } else {
                        0
                    };
                } else {
                    w5 <<= 1;
                }
            }
        }
    }
}
