use std::collections::HashMap;

use leptonica::Pix;

use crate::arith::{ArithEncoder, IntProc};
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
    symbols: &[Pix],
    symbol_indices: &[usize],
    unborder: bool,
    border_size: u32,
) -> Result<SymbolTableResult, Jbig2Error> {
    // 範囲外インデックスの検証
    for &idx in symbol_indices {
        if idx >= symbols.len() {
            return Err(Jbig2Error::InvalidInput(format!(
                "symbol index {idx} out of bounds (symbols.len() = {})",
                symbols.len()
            )));
        }
    }

    let n = symbol_indices.len();

    // 空リストの場合はすぐ返す
    if n == 0 {
        return Ok(SymbolTableResult {
            data: Vec::new(),
            symmap: HashMap::new(),
        });
    }

    // シンボルの有効寸法を取得（unborder時はボーダー除去後のサイズ）
    let effective_dims: Vec<(u32, u32)> = symbol_indices
        .iter()
        .map(|&idx| {
            let pix = &symbols[idx];
            if unborder {
                (
                    pix.width().saturating_sub(2 * border_size),
                    pix.height().saturating_sub(2 * border_size),
                )
            } else {
                (pix.width(), pix.height())
            }
        })
        .collect();

    // インデックス列を (height, width) でソート。
    // C++版: 1) 全体を高さ順にソート、2) 各高さクラス内を幅順にソート。
    // Rustでは (height, width) のタプルで一括ソート（同等）。
    // 同一 (height, width) の場合は入力順を保持（安定ソート）。
    let mut sorted_positions: Vec<usize> = (0..n).collect();
    sorted_positions.sort_by_key(|&pos| {
        let (w, h) = effective_dims[pos];
        (h, w)
    });

    let mut encoder = ArithEncoder::new();
    let mut symmap = HashMap::with_capacity(n);
    let mut encoded_num: usize = 0;

    // 高さクラスごとに処理
    let mut i = 0;
    let mut prev_height: u32 = 0;
    while i < n {
        let (_, height) = effective_dims[sorted_positions[i]];

        // 同一高さのシンボルをまとめる
        let j = sorted_positions[i..].partition_point(|&pos| effective_dims[pos].1 == height) + i;

        // デルタ高さを符号化
        let delta_height = height as i32 - prev_height as i32;
        encoder.encode_int(IntProc::Dh, delta_height);
        prev_height = height;

        // 高さクラス内のシンボルを幅順で処理（sorted_positions はすでに幅順）
        let mut prev_width: u32 = 0;
        for &pos in &sorted_positions[i..j] {
            let orig_idx = symbol_indices[pos];
            let (width, _) = effective_dims[pos];

            // デルタ幅を符号化
            let delta_width = width as i32 - prev_width as i32;
            encoder.encode_int(IntProc::Dw, delta_width);
            prev_width = width;

            // シンボルビットマップを符号化
            let unbordered_pix;
            let pix_to_encode = if unborder {
                unbordered_pix = symbols[orig_idx]
                    .remove_border(border_size)
                    .map_err(|e| Jbig2Error::InvalidInput(e.to_string()))?;
                &unbordered_pix
            } else {
                &symbols[orig_idx]
            };

            let mut pix_mut = pix_to_encode.to_mut();
            pix_mut.set_pad_bits(0);
            encoder.encode_bitimage(pix_mut.data(), width, height, false);

            // シンボルマッピングを記録
            symmap.insert(orig_idx, encoded_num);
            encoded_num += 1;
        }

        // 高さクラス終端のOOBマーカー
        encoder.encode_oob(IntProc::Dw);

        i = j;
    }

    // エクスポートテーブル: [0, n] で全シンボルをエクスポート
    encoder.encode_int(IntProc::Ex, 0);
    encoder.encode_int(IntProc::Ex, n as i32);

    encoder.encode_final();
    let data = encoder.to_vec();

    Ok(SymbolTableResult { data, symmap })
}
