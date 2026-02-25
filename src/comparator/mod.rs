use leptonica::{Pix, PixelDepth};

use crate::error::Jbig2Error;

const DIVIDER: usize = 9;

/// 2つのシンボルテンプレートが視覚的に等価かどうかを判定する。
///
/// C++版 `jbig2enc_are_equivalent()`（`jbig2comparator.cc:44-262`）に対応。
/// XOR差分の空間分布を9x9グリッドで分析し、水平線・垂直線・交差線・集中差分を検出する。
///
/// # Arguments
/// - `first` - 第1テンプレート（1bpp）
/// - `second` - 第2テンプレート（1bpp）
///
/// # Returns
/// - `Ok(true)` — 視覚的に等価
/// - `Ok(false)` — 非等価（サイズ不一致含む）
/// - `Err(...)` — 内部エラー（XOR演算失敗等）
pub fn are_equivalent(first: &Pix, second: &Pix) -> Result<bool, Jbig2Error> {
    // Stage 1: 早期棄却
    if !first.sizes_equal(second) {
        return Ok(false);
    }

    if first.wpl() != second.wpl() {
        return Ok(false);
    }

    if first.depth() != PixelDepth::Bit1 {
        return Ok(false);
    }

    // Stage 2: XOR差分の全体評価
    let xor_pix = first
        .xor(second)
        .map_err(|e| Jbig2Error::InvalidInput(format!("xor failed: {e}")))?;

    let w = xor_pix.width();
    let h = xor_pix.height();

    let pcount = first.count_pixels();

    // 25%閾値で早期棄却
    let thresh = pcount / 4;
    let above = xor_pix
        .threshold_pixel_sum(thresh)
        .map_err(|e| Jbig2Error::InvalidInput(format!("threshold_pixel_sum failed: {e}")))?;
    if above {
        return Ok(false);
    }

    // Stage 3: 9x9グリッド空間分布分析
    let vertical_part = h as usize / DIVIDER;
    let horizontal_part = w as usize / DIVIDER;

    // 閾値計算
    let (a, b) = if vertical_part < horizontal_part {
        (horizontal_part / 2, vertical_part / 2)
    } else {
        (vertical_part / 2, horizontal_part / 2)
    };
    let point_thresh = (a as f64) * (b as f64) * std::f64::consts::PI;
    let vline_thresh = ((vertical_part * (horizontal_part / 2)) as f64 * 0.9) as i64;
    let hline_thresh = ((horizontal_part * (vertical_part / 2)) as f64 * 0.9) as i64;

    // グリッドカウント配列
    let mut parsed = [[0u32; DIVIDER]; DIVIDER];
    let mut h_parsed = [[0u32; DIVIDER]; DIVIDER * 2];
    let mut v_parsed = [[0u32; DIVIDER * 2]; DIVIDER];

    let mut h_mod_counter: usize = 0;
    let mut v_mod_counter: usize = 0;

    for hp in 0..DIVIDER {
        let h_start = horizontal_part * hp + h_mod_counter;
        let h_end;
        if hp == DIVIDER - 1 {
            h_mod_counter = 0;
            h_end = w as usize;
        } else if !(w as usize - h_mod_counter).is_multiple_of(DIVIDER) {
            h_end = h_start + horizontal_part + 1;
            h_mod_counter += 1;
        } else {
            h_end = h_start + horizontal_part;
        }

        for vp in 0..DIVIDER {
            let v_start = vertical_part * vp + v_mod_counter;
            let v_end;
            if vp == DIVIDER - 1 {
                v_mod_counter = 0;
                v_end = h as usize;
            } else if !(h as usize - v_mod_counter).is_multiple_of(DIVIDER) {
                v_end = v_start + vertical_part + 1;
                v_mod_counter += 1;
            } else {
                v_end = v_start + vertical_part;
            }

            let mut left_count: u32 = 0;
            let mut right_count: u32 = 0;
            let mut up_count: u32 = 0;
            let mut down_count: u32 = 0;

            let h_center = (h_start + h_end) / 2;
            let v_center = (v_start + v_end) / 2;

            for i in h_start..h_end {
                for j in v_start..v_end {
                    if let Some(val) = xor_pix.get_pixel(i as u32, j as u32)
                        && val == 1
                    {
                        if i < h_center {
                            left_count += 1;
                        } else {
                            right_count += 1;
                        }
                        if j < v_center {
                            up_count += 1;
                        } else {
                            down_count += 1;
                        }
                    }
                }
            }

            parsed[hp][vp] = left_count + right_count;
            h_parsed[hp * 2][vp] = left_count;
            h_parsed[hp * 2 + 1][vp] = right_count;
            v_parsed[hp][vp * 2] = up_count;
            v_parsed[hp][vp * 2 + 1] = down_count;
        }
    }

    // Stage 4: パターン検出

    // Check 1: 水平線検出
    for i in 0..DIVIDER * 2 - 1 {
        for j in 0..DIVIDER - 1 {
            let mut sum: i64 = 0;
            for x in 0..2 {
                for y in 0..2 {
                    sum += h_parsed[i + x][j + y] as i64;
                }
            }
            if sum >= hline_thresh {
                return Ok(false);
            }
        }
    }

    // Check 2: 垂直線検出
    for i in 0..DIVIDER - 1 {
        for j in 0..DIVIDER * 2 - 1 {
            let mut sum: i64 = 0;
            for x in 0..2 {
                for y in 0..2 {
                    sum += v_parsed[i + x][j + y] as i64;
                }
            }
            if sum >= vline_thresh {
                return Ok(false);
            }
        }
    }

    // Check 3: 交差線検出
    for i in 0..DIVIDER - 2 {
        for j in 0..DIVIDER - 2 {
            let mut left_cross: i64 = 0;
            let mut right_cross: i64 = 0;
            for x in 0..3 {
                for y in 0..3 {
                    if x == y {
                        left_cross += parsed[i + x][j + y] as i64;
                    }
                    if (2 - x) == y {
                        right_cross += parsed[i + x][j + y] as i64;
                    }
                }
            }
            if left_cross >= hline_thresh || right_cross >= hline_thresh {
                return Ok(false);
            }
        }
    }

    // Check 4: 集中差分検出
    for i in 0..DIVIDER - 1 {
        for j in 0..DIVIDER - 1 {
            let mut sum: f64 = 0.0;
            for x in 0..2 {
                for y in 0..2 {
                    sum += parsed[i + x][j + y] as f64;
                }
            }
            if sum >= point_thresh {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
