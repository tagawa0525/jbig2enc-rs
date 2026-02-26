use leptonica::core::pix::RemoveColormapTarget;
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
}
