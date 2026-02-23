use super::encoder::{ArithEncoder, INT_CTX_SIZE, encode_bit_raw};

/// 整数符号化プロシージャの種別（JBIG2仕様 6.4節）。
///
/// C++版の `JBIG2_IA*` 定数に対応。各プロシージャは独立した
/// コンテキスト（512バイト）を持つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IntProc {
    Ai = 0,
    Dh = 1,
    Ds = 2,
    Dt = 3,
    Dw = 4,
    Ex = 5,
    Fs = 6,
    It = 7,
    Rdh = 8,
    Rdw = 9,
    Rdx = 10,
    Rdy = 11,
    Ri = 12,
}

/// 整数符号化レンジの1エントリ。
///
/// C++版 `intencrange_s` に対応。
struct IntEncRange {
    bot: i32,
    top: i32,
    data: u8,
    bits: u8,
    delta: u32,
    intbits: u8,
}

/// 13エントリの整数符号化レンジテーブル。
///
/// C++版 `intencrange[]` に対応。
static INT_ENC_RANGE: [IntEncRange; 13] = [
    IntEncRange {
        bot: 0,
        top: 3,
        data: 0,
        bits: 2,
        delta: 0,
        intbits: 2,
    },
    IntEncRange {
        bot: -1,
        top: -1,
        data: 9,
        bits: 4,
        delta: 0,
        intbits: 0,
    },
    IntEncRange {
        bot: -3,
        top: -2,
        data: 5,
        bits: 3,
        delta: 2,
        intbits: 1,
    },
    IntEncRange {
        bot: 4,
        top: 19,
        data: 2,
        bits: 3,
        delta: 4,
        intbits: 4,
    },
    IntEncRange {
        bot: -19,
        top: -4,
        data: 3,
        bits: 3,
        delta: 4,
        intbits: 4,
    },
    IntEncRange {
        bot: 20,
        top: 83,
        data: 6,
        bits: 4,
        delta: 20,
        intbits: 6,
    },
    IntEncRange {
        bot: -83,
        top: -20,
        data: 7,
        bits: 4,
        delta: 20,
        intbits: 6,
    },
    IntEncRange {
        bot: 84,
        top: 339,
        data: 14,
        bits: 5,
        delta: 84,
        intbits: 8,
    },
    IntEncRange {
        bot: -339,
        top: -84,
        data: 15,
        bits: 5,
        delta: 84,
        intbits: 8,
    },
    IntEncRange {
        bot: 340,
        top: 4435,
        data: 30,
        bits: 6,
        delta: 340,
        intbits: 12,
    },
    IntEncRange {
        bot: -4435,
        top: -340,
        data: 31,
        bits: 6,
        delta: 340,
        intbits: 12,
    },
    IntEncRange {
        bot: 4436,
        top: 2000000000,
        data: 62,
        bits: 6,
        delta: 4436,
        intbits: 32,
    },
    IntEncRange {
        bot: -2000000000,
        top: -4436,
        data: 63,
        bits: 6,
        delta: 4436,
        intbits: 32,
    },
];

/// `prev` の更新ロジック（C++版と同一）。
#[inline]
fn update_prev(prev: u32, v: u32) -> u32 {
    if prev & 0x100 != 0 {
        (((prev << 1) | v) & 0x1ff) | 0x100
    } else {
        (prev << 1) | v
    }
}

impl ArithEncoder {
    /// 整数値をJBIG2算術符号化で符号化する。
    ///
    /// C++版 `jbig2enc_int()` に対応。
    /// `intctx[proc * INT_CTX_SIZE .. (proc+1) * INT_CTX_SIZE]` コンテキストを使用。
    pub fn encode_int(&mut self, proc: IntProc, value: i32) {
        assert!(
            (-2_000_000_000..=2_000_000_000).contains(&value),
            "value out of range: {value}"
        );

        let ctx_start = proc as usize * INT_CTX_SIZE;

        let range_idx = INT_ENC_RANGE
            .iter()
            .position(|r| r.bot <= value && r.top >= value)
            .expect("value must fit in a range");

        let range = &INT_ENC_RANGE[range_idx];

        let abs_value = value.unsigned_abs();
        let mut encoded_value = abs_value - range.delta;

        let mut prev: u32 = 1;

        // プレフィクスビットをLSBファーストで符号化
        let mut data = range.data;
        for _ in 0..range.bits {
            let v = data & 1;
            let ctx = &mut self.intctx[ctx_start..ctx_start + INT_CTX_SIZE];
            encode_bit_raw(
                &mut self.c,
                &mut self.a,
                &mut self.ct,
                &mut self.b,
                &mut self.bp,
                &mut self.output_chunks,
                &mut self.outbuf,
                ctx,
                prev as usize,
                v,
            );
            data >>= 1;
            prev = update_prev(prev, v as u32);
        }

        // 値ビットをMSBファーストで符号化（intbits=0の場合はスキップ）
        if range.intbits > 0 {
            encoded_value <<= 32 - range.intbits;
        }
        for _ in 0..range.intbits {
            let v = ((encoded_value & 0x8000_0000) >> 31) as u8;
            let ctx = &mut self.intctx[ctx_start..ctx_start + INT_CTX_SIZE];
            encode_bit_raw(
                &mut self.c,
                &mut self.a,
                &mut self.ct,
                &mut self.b,
                &mut self.bp,
                &mut self.output_chunks,
                &mut self.outbuf,
                ctx,
                prev as usize,
                v,
            );
            encoded_value <<= 1;
            prev = update_prev(prev, v as u32);
        }
    }

    /// OOBセンチネルを符号化する。
    ///
    /// C++版 `jbig2enc_oob()` に対応。
    /// コンテキスト番号 1, 3, 6, 12 に対して 1, 0, 0, 0 をエンコード。
    pub fn encode_oob(&mut self, proc: IntProc) {
        let ctx_start = proc as usize * INT_CTX_SIZE;

        for (ctx_num, bit) in [(1usize, 1u8), (3, 0), (6, 0), (12, 0)] {
            let ctx = &mut self.intctx[ctx_start..ctx_start + INT_CTX_SIZE];
            encode_bit_raw(
                &mut self.c,
                &mut self.a,
                &mut self.ct,
                &mut self.b,
                &mut self.bp,
                &mut self.output_chunks,
                &mut self.outbuf,
                ctx,
                ctx_num,
                bit,
            );
        }
    }

    /// シンボルIDを固定ビット長で符号化する。
    ///
    /// C++版 `jbig2enc_iaid()` に対応。
    /// `iaidctx` は `1 << symcodelen` バイトのコンテキスト配列。
    pub fn encode_iaid(&mut self, symcodelen: u32, value: u32) {
        if symcodelen == 0 {
            return;
        }
        self.ensure_iaid_ctx(symcodelen);

        let mask = (1u32 << (symcodelen + 1)) - 1;
        let mut shifted = value << (32 - symcodelen);
        let mut prev: u32 = 1;

        for _ in 0..symcodelen {
            let tval = (prev & mask) as usize;
            let v = ((shifted & 0x8000_0000) >> 31) as u8;

            // iaidctx を分離借用
            let (c, a, ct, b, bp) = (
                &mut self.c,
                &mut self.a,
                &mut self.ct,
                &mut self.b,
                &mut self.bp,
            );
            encode_bit_raw(
                c,
                a,
                ct,
                b,
                bp,
                &mut self.output_chunks,
                &mut self.outbuf,
                &mut self.iaidctx,
                tval,
                v,
            );

            prev = (prev << 1) | v as u32;
            shifted <<= 1;
        }
    }
}
