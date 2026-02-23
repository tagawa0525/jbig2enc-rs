use super::state_table::STATE_TABLE;

const MAX_CTX: usize = 65536;
const OUTPUT_BUFFER_SIZE: usize = 20 * 1024;

/// JBIG2算術エンコーダ（QMコーダ）。
///
/// C++版 `jbig2enc_ctx` に対応。出力はRustの `Vec<u8>` チャンクで管理。
pub struct ArithEncoder {
    c: u32,
    a: u16,
    ct: u8,
    b: u8,
    bp: i32,
    output_chunks: Vec<Vec<u8>>,
    outbuf: Vec<u8>,
    pub(crate) context: Vec<u8>,
}

impl Default for ArithEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ArithEncoder {
    /// 新しいArithEncoderを生成する。
    pub fn new() -> Self {
        Self {
            c: 0,
            a: 0x8000,
            ct: 12,
            b: 0,
            bp: -1,
            output_chunks: Vec::new(),
            outbuf: Vec::with_capacity(OUTPUT_BUFFER_SIZE),
            context: vec![0u8; MAX_CTX],
        }
    }

    /// コーダの算術状態をリセットする（出力バッファは保持）。
    pub fn reset(&mut self) {
        self.a = 0x8000;
        self.c = 0;
        self.ct = 12;
        self.bp = -1;
        self.b = 0;
        self.context.fill(0);
    }

    /// 出力バッファをすべてクリアする。
    pub fn flush(&mut self) {
        self.outbuf.clear();
        self.output_chunks.clear();
        self.bp = -1;
    }

    /// 出力バッファにバイトを追加する。バッファが満杯なら新チャンクを確保。
    fn emit(&mut self) {
        if self.outbuf.len() == OUTPUT_BUFFER_SIZE {
            let full_chunk =
                std::mem::replace(&mut self.outbuf, Vec::with_capacity(OUTPUT_BUFFER_SIZE));
            self.output_chunks.push(full_chunk);
        }
        self.outbuf.push(self.b);
    }

    /// BYTEOUT手順（仕様書準拠）。
    fn byteout(&mut self) {
        if self.b == 0xff {
            // rblock
            if self.bp >= 0 {
                self.emit();
            }
            self.b = (self.c >> 20) as u8;
            self.bp += 1;
            self.c &= 0xf_ffff;
            self.ct = 7;
            return;
        }

        if self.c < 0x800_0000 {
            // lblock
            if self.bp >= 0 {
                self.emit();
            }
            self.b = (self.c >> 19) as u8;
            self.bp += 1;
            self.c &= 0x7_ffff;
            self.ct = 8;
            return;
        }

        // carry propagation
        self.b += 1;
        if self.b != 0xff {
            // lblock
            if self.bp >= 0 {
                self.emit();
            }
            self.b = (self.c >> 19) as u8;
            self.bp += 1;
            self.c &= 0x7_ffff;
            self.ct = 8;
            return;
        }

        // b became 0xff after increment → rblock
        self.c &= 0x7ff_ffff;
        if self.bp >= 0 {
            self.emit();
        }
        self.b = (self.c >> 20) as u8;
        self.bp += 1;
        self.c &= 0xf_ffff;
        self.ct = 7;
    }

    /// 1ビットを符号化する。
    ///
    /// `context`: 状態インデックス配列（画像用は`self.context`、整数用は`intctx[proc]`）
    /// `ctx_num`: コンテキスト番号
    /// `d`: 符号化するビット（0 or 1）
    pub fn encode_bit(&mut self, context: &mut [u8], ctx_num: u32, d: u8) {
        let i = context[ctx_num as usize];
        let mps: u8 = if i > 46 { 1 } else { 0 };
        let qe = STATE_TABLE[i as usize].qe;

        if d != mps {
            // CODELPS
            self.a -= qe;
            if self.a < qe {
                self.c += qe as u32;
            } else {
                self.a = qe;
            }
            context[ctx_num as usize] = STATE_TABLE[i as usize].lps;

            // RENORME
            loop {
                self.a <<= 1;
                self.c <<= 1;
                self.ct -= 1;
                if self.ct == 0 {
                    self.byteout();
                }
                if self.a & 0x8000 != 0 {
                    break;
                }
            }
        } else {
            // CODEMPS
            self.a -= qe;
            if self.a & 0x8000 == 0 {
                if self.a < qe {
                    self.a = qe;
                } else {
                    self.c += qe as u32;
                }
                context[ctx_num as usize] = STATE_TABLE[i as usize].mps;

                // RENORME
                loop {
                    self.a <<= 1;
                    self.c <<= 1;
                    self.ct -= 1;
                    if self.ct == 0 {
                        self.byteout();
                    }
                    if self.a & 0x8000 != 0 {
                        break;
                    }
                }
            } else {
                self.c += qe as u32;
            }
        }
    }

    /// 符号化を完了し、終端バイト列を出力する（FINALISE手順）。
    pub fn encode_final(&mut self) {
        // SETBITS
        let tempc = self.c.wrapping_add(self.a as u32);
        self.c |= 0xffff;
        if self.c >= tempc {
            self.c -= 0x8000;
        }

        self.c <<= self.ct;
        self.byteout();
        self.c <<= self.ct;
        self.byteout();
        self.emit();
        if self.b != 0xff {
            self.b = 0xff;
            self.emit();
        }
        self.b = 0xac;
        self.emit();
    }

    /// 出力データの合計サイズ（バイト数）を返す。
    pub fn data_size(&self) -> usize {
        OUTPUT_BUFFER_SIZE * self.output_chunks.len() + self.outbuf.len()
    }

    /// 符号化済みデータをVec<u8>として返す。
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.data_size());
        for chunk in &self.output_chunks {
            buf.extend_from_slice(chunk);
        }
        buf.extend_from_slice(&self.outbuf);
        buf
    }
}
