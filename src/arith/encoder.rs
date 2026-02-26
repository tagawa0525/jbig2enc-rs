use super::state_table::STATE_TABLE;

pub(crate) const MAX_CTX: usize = 65536;
const OUTPUT_BUFFER_SIZE: usize = 20 * 1024;
pub(crate) const INT_CTX_COUNT: usize = 13;
pub(crate) const INT_CTX_SIZE: usize = 512;

/// JBIG2算術エンコーダ（QMコーダ）。
///
/// C++版 `jbig2enc_ctx` に対応。出力はRustの `Vec<u8>` チャンクで管理。
pub struct ArithEncoder {
    pub(crate) c: u32,
    pub(crate) a: u16,
    pub(crate) ct: u8,
    pub(crate) b: u8,
    pub(crate) bp: i32,
    pub(crate) output_chunks: Vec<Vec<u8>>,
    pub(crate) outbuf: Vec<u8>,
    /// 画像符号化用コンテキスト（65536エントリ）
    pub(crate) context: Vec<u8>,
    /// 整数符号化用コンテキスト（13プロシージャ × 512バイト）
    pub(crate) intctx: Vec<u8>,
    /// シンボルID符号化用コンテキスト（1 << symcodelen バイト）
    pub(crate) iaidctx: Vec<u8>,
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
            intctx: vec![0u8; INT_CTX_COUNT * INT_CTX_SIZE],
            iaidctx: Vec::new(),
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
        self.intctx.fill(0);
        self.iaidctx.clear();
    }

    /// 出力バッファをすべてクリアする。
    pub fn flush(&mut self) {
        self.outbuf.clear();
        self.output_chunks.clear();
        self.bp = -1;
    }

    /// 1ビットを符号化する。
    ///
    /// `context`: 状態インデックス配列（画像用は`self.context`、整数用は`intctx[proc]`）
    /// `ctx_num`: コンテキスト番号（0-based）
    /// `d`: 符号化するビット（0 or 1）
    pub fn encode_bit(&mut self, context: &mut [u8], ctx_num: u32, d: u8) {
        encode_bit_raw(
            &mut self.c,
            &mut self.a,
            &mut self.ct,
            &mut self.b,
            &mut self.bp,
            &mut self.output_chunks,
            &mut self.outbuf,
            context,
            ctx_num as usize,
            d,
        );
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
        byteout_raw(
            &mut self.c,
            &mut self.ct,
            &mut self.b,
            &mut self.bp,
            &mut self.output_chunks,
            &mut self.outbuf,
        );
        self.c <<= self.ct;
        byteout_raw(
            &mut self.c,
            &mut self.ct,
            &mut self.b,
            &mut self.bp,
            &mut self.output_chunks,
            &mut self.outbuf,
        );
        emit_byte(
            self.b,
            &mut self.bp,
            &mut self.output_chunks,
            &mut self.outbuf,
        );
        if self.b != 0xff {
            self.b = 0xff;
            emit_byte(
                self.b,
                &mut self.bp,
                &mut self.output_chunks,
                &mut self.outbuf,
            );
        }
        self.b = 0xac;
        emit_byte(
            self.b,
            &mut self.bp,
            &mut self.output_chunks,
            &mut self.outbuf,
        );
    }

    /// 出力データの合計サイズ（バイト数）を返す。
    pub fn data_size(&self) -> usize {
        OUTPUT_BUFFER_SIZE * self.output_chunks.len() + self.outbuf.len()
    }

    /// 符号化済みデータを`Vec<u8>`として返す。
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.data_size());
        for chunk in &self.output_chunks {
            buf.extend_from_slice(chunk);
        }
        buf.extend_from_slice(&self.outbuf);
        buf
    }

    /// `iaidctx` を `symcodelen` ビット分確保する（未確保の場合のみ）。
    pub(crate) fn ensure_iaid_ctx(&mut self, symcodelen: u32) {
        let needed = 1usize << symcodelen;
        if self.iaidctx.len() < needed {
            self.iaidctx.resize(needed, 0);
        }
    }
}

// ============================================================
// 内部フリー関数（借用分離のため pub(crate) で公開）
// ============================================================

/// 出力バッファに1バイト追加する。バッファが満杯なら新チャンクを確保。
#[inline]
pub(crate) fn emit_byte(
    byte: u8,
    bp: &mut i32,
    output_chunks: &mut Vec<Vec<u8>>,
    outbuf: &mut Vec<u8>,
) {
    // bp >= 0 のときのみ emit（bp=-1 は初期バイトをスキップするフラグ）
    if *bp < 0 {
        return;
    }
    if outbuf.len() == OUTPUT_BUFFER_SIZE {
        let full_chunk = std::mem::replace(outbuf, Vec::with_capacity(OUTPUT_BUFFER_SIZE));
        output_chunks.push(full_chunk);
    }
    outbuf.push(byte);
}

/// BYTEOUT手順（仕様書準拠）。
#[inline]
pub(crate) fn byteout_raw(
    c: &mut u32,
    ct: &mut u8,
    b: &mut u8,
    bp: &mut i32,
    output_chunks: &mut Vec<Vec<u8>>,
    outbuf: &mut Vec<u8>,
) {
    if *b == 0xff {
        // rblock
        if *bp >= 0 {
            if outbuf.len() == OUTPUT_BUFFER_SIZE {
                let full_chunk = std::mem::replace(outbuf, Vec::with_capacity(OUTPUT_BUFFER_SIZE));
                output_chunks.push(full_chunk);
            }
            outbuf.push(*b);
        }
        *b = (*c >> 20) as u8;
        *bp += 1;
        *c &= 0xf_ffff;
        *ct = 7;
        return;
    }

    if *c < 0x800_0000 {
        // lblock
        if *bp >= 0 {
            if outbuf.len() == OUTPUT_BUFFER_SIZE {
                let full_chunk = std::mem::replace(outbuf, Vec::with_capacity(OUTPUT_BUFFER_SIZE));
                output_chunks.push(full_chunk);
            }
            outbuf.push(*b);
        }
        *b = (*c >> 19) as u8;
        *bp += 1;
        *c &= 0x7_ffff;
        *ct = 8;
        return;
    }

    // carry propagation
    *b += 1;
    if *b != 0xff {
        // lblock after increment
        if *bp >= 0 {
            if outbuf.len() == OUTPUT_BUFFER_SIZE {
                let full_chunk = std::mem::replace(outbuf, Vec::with_capacity(OUTPUT_BUFFER_SIZE));
                output_chunks.push(full_chunk);
            }
            outbuf.push(*b);
        }
        *b = (*c >> 19) as u8;
        *bp += 1;
        *c &= 0x7_ffff;
        *ct = 8;
        return;
    }

    // b became 0xff after increment → rblock
    *c &= 0x7ff_ffff;
    if *bp >= 0 {
        if outbuf.len() == OUTPUT_BUFFER_SIZE {
            let full_chunk = std::mem::replace(outbuf, Vec::with_capacity(OUTPUT_BUFFER_SIZE));
            output_chunks.push(full_chunk);
        }
        outbuf.push(*b);
    }
    *b = (*c >> 20) as u8;
    *bp += 1;
    *c &= 0xf_ffff;
    *ct = 7;
}

/// 1ビット符号化のコア（ENCODE/CODELPS/CODEMPS統合）。
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_bit_raw(
    c: &mut u32,
    a: &mut u16,
    ct: &mut u8,
    b: &mut u8,
    bp: &mut i32,
    output_chunks: &mut Vec<Vec<u8>>,
    outbuf: &mut Vec<u8>,
    context: &mut [u8],
    ctx_num: usize,
    d: u8,
) {
    let i = context[ctx_num];
    let mps: u8 = if i > 46 { 1 } else { 0 };
    let qe = STATE_TABLE[i as usize].qe;

    if d != mps {
        // CODELPS
        *a -= qe;
        if *a < qe {
            *c += qe as u32;
        } else {
            *a = qe;
        }
        context[ctx_num] = STATE_TABLE[i as usize].lps;

        // RENORME
        loop {
            *a <<= 1;
            *c <<= 1;
            *ct -= 1;
            if *ct == 0 {
                byteout_raw(c, ct, b, bp, output_chunks, outbuf);
            }
            if *a & 0x8000 != 0 {
                break;
            }
        }
    } else {
        // CODEMPS
        *a -= qe;
        if *a & 0x8000 == 0 {
            if *a < qe {
                *a = qe;
            } else {
                *c += qe as u32;
            }
            context[ctx_num] = STATE_TABLE[i as usize].mps;

            // RENORME
            loop {
                *a <<= 1;
                *c <<= 1;
                *ct -= 1;
                if *ct == 0 {
                    byteout_raw(c, ct, b, bp, output_chunks, outbuf);
                }
                if *a & 0x8000 != 0 {
                    break;
                }
            }
        } else {
            *c += qe as u32;
        }
    }
}
