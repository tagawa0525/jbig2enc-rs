const _MAX_CTX: usize = 65536;
const _OUTPUT_BUFFER_SIZE: usize = 20 * 1024;

/// JBIG2算術エンコーダ（QMコーダ）。
pub struct ArithEncoder {
    _c: u32,
    _a: u16,
    _ct: u8,
    _b: u8,
    _bp: i32,
    _output_chunks: Vec<Vec<u8>>,
    _outbuf: Vec<u8>,
    _context: Vec<u8>,
}

impl Default for ArithEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ArithEncoder {
    /// 新しいArithEncoderを生成する。
    pub fn new() -> Self {
        todo!()
    }

    /// コーダの状態をリセットする（出力バッファは保持）。
    pub fn reset(&mut self) {
        todo!()
    }

    /// 出力バッファをクリアする。
    pub fn flush(&mut self) {
        todo!()
    }

    /// 1ビットを符号化する。
    pub fn encode_bit(&mut self, _context: &mut [u8], _ctx_num: u32, _d: u8) {
        todo!()
    }

    /// 符号化を完了し、終端バイト列を出力する。
    pub fn encode_final(&mut self) {
        todo!()
    }

    /// 出力データのサイズを返す。
    pub fn data_size(&self) -> usize {
        todo!()
    }

    /// 符号化済みデータをVec<u8>として返す。
    pub fn to_vec(&self) -> Vec<u8> {
        todo!()
    }
}
