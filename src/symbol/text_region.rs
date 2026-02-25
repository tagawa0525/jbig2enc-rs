use std::collections::HashMap;

use leptonica::Pix;

use crate::arith::{ArithEncoder, IntProc};
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
    instances: &[SymbolInstance],
    symbols: &[Pix],
    cfg: &TextRegionConfig<'_>,
) -> Result<TextRegionResult, Jbig2Error> {
    // strip_width は 1, 2, 4, 8 のみ有効
    if !matches!(cfg.strip_width, 1 | 2 | 4 | 8) {
        return Err(Jbig2Error::InvalidInput(format!(
            "invalid strip_width {}: must be 1, 2, 4, or 8",
            cfg.strip_width
        )));
    }

    // 空リストは空データで返す
    if instances.is_empty() {
        return Ok(TextRegionResult { data: Vec::new() });
    }

    // 全インスタンスのsymidを事前に解決（エラーを早期検出）
    // class_id が symbols の範囲内か、symid が symbits に収まるかを同時に検証する。
    let max_symid: usize = if cfg.symbits < usize::BITS {
        1usize << cfg.symbits
    } else {
        usize::MAX
    };
    let symids: Vec<usize> = instances
        .iter()
        .map(|inst| {
            if inst.class_id >= symbols.len() {
                return Err(Jbig2Error::InvalidInput(format!(
                    "class_id {} out of bounds (symbols.len() = {})",
                    inst.class_id,
                    symbols.len()
                )));
            }
            let symid = resolve_symid(inst.class_id, cfg)?;
            if symid >= max_symid {
                return Err(Jbig2Error::InvalidInput(format!(
                    "resolved symid {symid} does not fit in {} bits",
                    cfg.symbits
                )));
            }
            Ok(symid)
        })
        .collect::<Result<_, _>>()?;

    // インスタンスをY座標でソート（安定ソート）
    let mut sorted: Vec<usize> = (0..instances.len()).collect();
    sorted.sort_by_key(|&i| instances[i].y);

    let mut encoder = ArithEncoder::new();

    // 初期IADT=0（C++と同一）
    encoder.encode_int(IntProc::Dt, 0);

    let sw = cfg.strip_width as i32;
    let mut stript: i32 = 0;
    let mut firsts: i32 = 0;

    let mut i = 0;
    while i < sorted.len() {
        let y0 = instances[sorted[i]].y;
        // ストリップ下端をstrip_widthの倍数に切り下げ
        let height = (y0 / sw) * sw;

        // 同一ストリップ内のインスタンスを収集
        let j = sorted[i..]
            .iter()
            .position(|&idx| instances[idx].y >= height + sw)
            .map(|p| i + p)
            .unwrap_or(sorted.len());
        let strip_slice = &sorted[i..j];

        // ストリップ内をX座標でソート
        let mut strip: Vec<usize> = strip_slice.to_vec();
        strip.sort_by_key(|&idx| instances[idx].x);

        // デルタT符号化: IADT（strip_widthで割った値）
        let delta_t = (height - stript) / sw;
        encoder.encode_int(IntProc::Dt, delta_t);
        stript = height;

        let mut first_in_strip = true;
        let mut curs: i32 = 0;

        for &idx in &strip {
            let inst = &instances[idx];
            let symid = symids[idx];

            if first_in_strip {
                first_in_strip = false;
                let delta_fs = inst.x - firsts;
                encoder.encode_int(IntProc::Fs, delta_fs);
                firsts += delta_fs;
                curs = firsts;
            } else {
                let delta_s = inst.x - curs;
                encoder.encode_int(IntProc::Ds, delta_s);
                curs += delta_s;
            }

            // strip_width > 1 の場合のみY内オフセットを符号化
            if sw > 1 {
                let delta_t_in = inst.y - stript;
                encoder.encode_int(IntProc::It, delta_t_in);
            }

            // シンボルID符号化
            encoder.encode_iaid(cfg.symbits, symid as u32);

            // cursをシンボル幅分進める（C++: curs += sym->w - 1 or unbordered_w - 1）
            let sym_w = symbols[inst.class_id].width() as i32;
            let effective_w = if cfg.unborder {
                (sym_w - 2 * cfg.border_size as i32).max(1)
            } else {
                sym_w
            };
            curs += effective_w - 1;
        }

        // ストリップ終端OOB
        encoder.encode_oob(IntProc::Ds);

        i = j;
    }

    encoder.encode_final();
    Ok(TextRegionResult {
        data: encoder.to_vec(),
    })
}

/// class_id を symmap/symmap2 から encoded symid に解決する。
fn resolve_symid(class_id: usize, cfg: &TextRegionConfig<'_>) -> Result<usize, Jbig2Error> {
    if let Some(&id) = cfg.symmap.get(&class_id) {
        return Ok(id);
    }
    if let Some(sm2) = cfg.symmap2
        && let Some(&id) = sm2.get(&class_id)
    {
        return Ok(id + cfg.global_sym_count);
    }
    Err(Jbig2Error::InvalidInput(format!(
        "class_id {class_id} not found in symmap or symmap2"
    )))
}
