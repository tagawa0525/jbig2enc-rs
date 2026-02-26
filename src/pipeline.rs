use leptonica::core::pix::RemoveColormapTarget;
use leptonica::morph::sequence::morph_sequence;
use leptonica::region::{ConnectivityType, seedfill_binary_restricted};
use leptonica::transform::scale::expand_replicate;
use leptonica::{Pix, PixelDepth};

use crate::cli::CliError;

/// 任意の Pix（グレー・カラー・1bpp）を1bpp二値画像に変換する。
#[allow(dead_code)] // PR 3 の run() で使用予定
///
/// C++ 版 `jbig2.cc` main() の画像前処理ループに対応。
///
/// 処理ステップ:
/// 1. カラーマップ除去（BasedOnSrc: 画像内容に応じた最適な除去）
/// 2. 既に 1bpp の場合はそのまま返す（パススルー）
/// 3. 深さに応じたグレースケール変換（32bpp → グレー / 4-8bpp → そのまま）
/// 4. 適応的 or グローバル前処理（`clean_background_to_white` or スキップ）
/// 5. アップサンプリング or 閾値処理（`-2`/`-4`/なし）
pub fn binarize(
    pix: Pix,
    global: bool,
    bw_threshold: u8,
    up2: bool,
    up4: bool,
) -> Result<Pix, CliError> {
    // 引数チェック: up2 と up4 は排他
    if up2 && up4 {
        return Err(CliError::InvalidArgs("cannot use both -2 and -4".into()));
    }

    // Step 1: カラーマップを除去（REMOVE_CMAP_BASED_ON_SRC 相当）
    let pix_no_cmap = pix
        .remove_colormap(RemoveColormapTarget::BasedOnSrc)
        .map_err(|e| CliError::Image(format!("failed to remove colormap: {e}")))?;

    // Step 2: 既に1bppならそのまま返す
    if pix_no_cmap.depth() == PixelDepth::Bit1 {
        return Ok(pix_no_cmap);
    }

    // Step 3: グレースケールに変換
    let gray: Pix = if pix_no_cmap.depth() == PixelDepth::Bit32 {
        // 32bpp（RGB）→ グレー
        pix_no_cmap
            .convert_rgb_to_gray_fast()
            .map_err(|e| CliError::Image(format!("failed to convert RGB to gray: {e}")))?
    } else if matches!(pix_no_cmap.depth(), PixelDepth::Bit4 | PixelDepth::Bit8) {
        // 4/8bpp → そのまま使用
        pix_no_cmap
    } else {
        return Err(CliError::Image(format!(
            "unsupported input image depth: {}",
            pix_no_cmap.depth().bits()
        )));
    };

    // Step 4: 適応的 or グローバル前処理
    let adapt: Pix = if !global {
        // 適応的: 背景ノーマライゼーション
        leptonica::filter::adaptmap::clean_background_to_white(&gray, None, None)
            .map_err(|e| CliError::Image(format!("failed to clean background: {e}")))?
    } else {
        // グローバル: 前処理なし
        gray
    };

    // Step 5: アップサンプリング or 閾値処理
    if up2 {
        leptonica::transform::scale_gray_2x_li_thresh(&adapt, bw_threshold as i32)
            .map_err(|e| CliError::Image(format!("failed to upsample 2x: {e}")))
    } else if up4 {
        leptonica::transform::scale_gray_4x_li_thresh(&adapt, bw_threshold as i32)
            .map_err(|e| CliError::Image(format!("failed to upsample 4x: {e}")))
    } else {
        leptonica::color::threshold::threshold_to_binary(&adapt, bw_threshold)
            .map_err(|e| CliError::Image(format!("failed to threshold: {e}")))
    }
}

/// テキスト/グラフィクスセグメンテーションを行い、テキスト画像とグラフィクス画像に分離する。
///
/// C++版 `segment_image()`（`jbig2.cc:141-213`）に対応。
///
/// 1bpp 二値画像（`pixb`）と元のカラー/グレー画像（`piximg`）を受け取り、
/// 形態学的処理でグラフィクス領域を検出して分離する。
///
/// # Returns
///
/// `(text, graphics)`:
/// - `text: None` → テキスト領域が少なすぎる（< 100 pixels）ためスキップ
/// - `graphics: None` → グラフィクス領域が少なすぎる（< 100 pixels）
pub fn segment_image(pixb: &Pix, piximg: &Pix) -> Result<(Option<Pix>, Option<Pix>), CliError> {
    if pixb.depth() != PixelDepth::Bit1 {
        return Err(CliError::InvalidArgs(format!(
            "segment_image requires 1bpp input, got {}bpp",
            pixb.depth().bits()
        )));
    }

    // Step 1-2: 形態学処理でマスクとシードを生成（4x縮小空間）
    // C++: pixMorphSequence(pixb, "r11", 0)
    let pixmask4 = morph_sequence(pixb, "r11")
        .map_err(|e| CliError::Image(format!("morph_sequence mask failed: {e}")))?;
    // C++: pixMorphSequence(pixb, "r1143 + o4.4 + x4", 0)
    let pixseed4 = morph_sequence(pixb, "r1143 + o4.4 + x4")
        .map_err(|e| CliError::Image(format!("morph_sequence seed failed: {e}")))?;

    // Step 3: シードフィル（8連結）
    // C++: pixSeedfillBinary(NULL, pixseed4, pixmask4, 8)
    let pixsf4 = seedfill_binary_restricted(&pixseed4, &pixmask4, ConnectivityType::EightWay, 0, 0)
        .map_err(|e| CliError::Image(format!("seedfill failed: {e}")))?;

    // Step 4: 膨張
    // C++: pixMorphSequence(pixsf4, "d3.3", 0)
    let pixd4 = morph_sequence(&pixsf4, "d3.3")
        .map_err(|e| CliError::Image(format!("morph_sequence dilation failed: {e}")))?;

    // Step 5: 元サイズに復元（4x拡大）
    // C++: pixExpandBinaryPower2(pixd4, 4)
    let pixd = expand_replicate(&pixd4, 4)
        .map_err(|e| CliError::Image(format!("expand_replicate failed: {e}")))?;

    // Step 6: テキスト = binary AND NOT graphics_mask
    // C++: pixSubtract(pixb, pixb, pixd)
    let text = pixb
        .subtract(&pixd)
        .map_err(|e| CliError::Image(format!("subtract failed: {e}")))?;

    // Step 7: ピクセル数チェック
    let graphics_count = pixd.count_pixels();
    if graphics_count < 100 {
        // グラフィクス領域が少なすぎる → テキストのみ
        return Ok((Some(text), None));
    }

    let text_count = text.count_pixels();
    let text_result = if text_count < 100 { None } else { Some(text) };

    // Step 8: 元画像の深度に合わせてグラフィクスマスクを変換し、合成
    let piximg1 = match piximg.depth() {
        PixelDepth::Bit1 | PixelDepth::Bit8 | PixelDepth::Bit32 => piximg.clone(),
        d if d.bits() > 8 => piximg
            .convert_to_32()
            .map_err(|e| CliError::Image(format!("convert_to_32 failed: {e}")))?,
        _ => piximg
            .convert_to_8()
            .map_err(|e| CliError::Image(format!("convert_to_8 failed: {e}")))?,
    };

    // グラフィクスマスクを元画像と同じ深度に変換
    let pixd1 = match piximg1.depth() {
        PixelDepth::Bit32 => pixd
            .convert_to_32()
            .map_err(|e| CliError::Image(format!("convert_to_32 mask failed: {e}")))?,
        PixelDepth::Bit8 => pixd
            .convert_to_8()
            .map_err(|e| CliError::Image(format!("convert_to_8 mask failed: {e}")))?,
        _ => pixd,
    };

    // C++: pixRasteropFullImage(pixd1, piximg1, PIX_SRC | PIX_DST) → OR
    let graphics = pixd1
        .or(&piximg1)
        .map_err(|e| CliError::Image(format!("rasterop OR failed: {e}")))?;

    Ok((text_result, Some(graphics)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica::PixMut;

    fn white_1bpp(w: u32, h: u32) -> Pix {
        PixMut::new(w, h, PixelDepth::Bit1).unwrap().into()
    }

    fn gray_8bpp(w: u32, h: u32, val: u8) -> Pix {
        let mut pm = PixMut::new(w, h, PixelDepth::Bit8).unwrap();
        pm.set_all_gray(val).unwrap();
        pm.into()
    }

    fn rgb_32bpp(w: u32, h: u32, r: u8, g: u8, b: u8) -> Pix {
        let mut pm = PixMut::new(w, h, PixelDepth::Bit32).unwrap();
        for y in 0..h {
            for x in 0..w {
                pm.set_rgb(x, y, r, g, b).unwrap();
            }
        }
        pm.into()
    }

    // --- 1bpp 画像はそのまま通過 ---

    #[test]
    fn binarize_1bpp_returns_1bpp() {
        let pix = white_1bpp(32, 32);
        let result = binarize(pix, false, 200, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn binarize_1bpp_preserves_dimensions() {
        let pix = white_1bpp(40, 20);
        let result = binarize(pix, false, 200, false, false).unwrap();
        assert_eq!(result.width(), 40);
        assert_eq!(result.height(), 20);
    }

    // --- 8bpp → 1bpp 変換 ---

    #[test]
    fn binarize_8bpp_returns_1bpp() {
        let pix = gray_8bpp(32, 32, 100);
        let result = binarize(pix, false, 200, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn binarize_8bpp_global_returns_1bpp() {
        let pix = gray_8bpp(32, 32, 100);
        let result = binarize(pix, true, 128, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    // --- 32bpp（RGB）→ 1bpp 変換 ---

    #[test]
    fn binarize_32bpp_returns_1bpp() {
        let pix = rgb_32bpp(32, 32, 200, 200, 200);
        let result = binarize(pix, false, 200, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    #[test]
    fn binarize_32bpp_global_returns_1bpp() {
        let pix = rgb_32bpp(32, 32, 200, 200, 200);
        let result = binarize(pix, true, 128, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    // --- グローバル vs 適応的閾値 ---

    #[test]
    fn binarize_adaptive_and_global_both_succeed() {
        let pix_adaptive = gray_8bpp(32, 32, 150);
        let pix_global = gray_8bpp(32, 32, 150);

        let adaptive = binarize(pix_adaptive, false, 200, false, false);
        let global = binarize(pix_global, true, 200, false, false);

        assert!(adaptive.is_ok());
        assert!(global.is_ok());
        // どちらも 1bpp
        assert_eq!(adaptive.unwrap().depth(), PixelDepth::Bit1);
        assert_eq!(global.unwrap().depth(), PixelDepth::Bit1);
    }

    // --- 2x アップサンプリング ---

    #[test]
    fn binarize_up2_doubles_dimensions() {
        let pix = gray_8bpp(20, 10, 150);
        let result = binarize(pix, true, 200, true, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert_eq!(result.width(), 40);
        assert_eq!(result.height(), 20);
    }

    // --- 4x アップサンプリング ---

    #[test]
    fn binarize_up4_quadruples_dimensions() {
        let pix = gray_8bpp(10, 8, 150);
        let result = binarize(pix, true, 200, false, true).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert_eq!(result.width(), 40);
        assert_eq!(result.height(), 32);
    }

    // --- 非対応深さのエラー ---

    #[test]
    fn binarize_16bpp_unsupported() {
        let pm = PixMut::new(16, 16, PixelDepth::Bit16).unwrap();
        let pix: Pix = pm.into();
        let result = binarize(pix, false, 200, false, false);
        // 16bpp は非対応
        assert!(result.is_err());
    }

    // --- up2/up4 排他チェック ---

    #[test]
    fn binarize_up2_and_up4_returns_error() {
        let pix = gray_8bpp(16, 16, 100);
        let result = binarize(pix, true, 200, true, true);
        assert!(result.is_err());
    }

    // --- segment_image テスト ---

    /// テキスト領域のみの画像に対し、グラフィクスが None で返ること。
    ///
    /// テキスト（小さな矩形パターン）のみの 1bpp 画像ではグラフィクス領域が
    /// 検出されず `graphics: None` となる。
    #[test]
    fn segment_text_only_returns_no_graphics() {
        // テキスト風の小さな矩形パターン（グラフィクス要素なし）
        let mut pm = PixMut::new(200, 100, PixelDepth::Bit1).unwrap();
        for &(x, y, w, h) in &[
            (10u32, 10u32, 8u32, 12u32),
            (30, 10, 8, 12),
            (50, 10, 8, 12),
        ] {
            for dy in 0..h {
                for dx in 0..w {
                    pm.set_pixel(x + dx, y + dy, 1).unwrap();
                }
            }
        }
        let pixb: Pix = pm.into();
        let piximg = pixb.clone();

        let (text, graphics) = segment_image(&pixb, &piximg).unwrap();
        // グラフィクス領域は検出されない
        assert!(
            graphics.is_none(),
            "text-only image should have no graphics"
        );
        // テキスト領域は存在する
        assert!(text.is_some(), "text-only image should have text");
        assert_eq!(text.unwrap().depth(), PixelDepth::Bit1);
    }

    /// グラフィクス領域を含む画像から text と graphics の両方が得られること。
    ///
    /// 大きなブロック（グラフィクス風）と小さな矩形（テキスト風）を含む
    /// 合成画像でセグメンテーションを行う。
    #[test]
    fn segment_mixed_returns_both() {
        let mut pm = PixMut::new(400, 200, PixelDepth::Bit1).unwrap();
        // 大きなブロック（グラフィクス風: 150x100）
        for y in 20..120 {
            for x in 200..350 {
                pm.set_pixel(x, y, 1).unwrap();
            }
        }
        // 小さな矩形（テキスト風）
        for &(bx, by) in &[(10u32, 10u32), (30, 10), (50, 10), (10, 40), (30, 40)] {
            for dy in 0..10 {
                for dx in 0..6 {
                    pm.set_pixel(bx + dx, by + dy, 1).unwrap();
                }
            }
        }
        let pixb: Pix = pm.into();
        // 8bpp グレースケール版を元画像として用意
        let piximg = pixb.convert_to_8().unwrap();

        let (text, graphics) = segment_image(&pixb, &piximg).unwrap();
        assert!(text.is_some(), "mixed image should have text");
        assert!(graphics.is_some(), "mixed image should have graphics");
        // テキスト画像は 1bpp
        assert_eq!(text.unwrap().depth(), PixelDepth::Bit1);
        // グラフィクス画像の深度は元画像と同じ
        assert_eq!(graphics.unwrap().depth(), PixelDepth::Bit8);
    }

    /// 1bpp 以外の入力画像はエラーになること。
    #[test]
    fn segment_non_binary_input_returns_error() {
        let pix = gray_8bpp(100, 100, 128);
        let piximg = pix.clone();
        let result = segment_image(&pix, &piximg);
        assert!(result.is_err());
    }
}
