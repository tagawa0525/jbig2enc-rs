use leptonica::Pix;

use crate::cli::CliError;

/// 任意の Pix（グレー・カラー・1bpp）を1bpp二値画像に変換する。
///
/// C++ 版 `jbig2.cc` main() の画像前処理ループに対応。
///
/// 処理ステップ:
/// 1. カラーマップ除去（BasedOnSrc: 画像内容に応じた最適な除去）
/// 2. 深さに応じたグレースケール変換（32bpp → グレー / 4-8bpp → そのまま）
/// 3. 適応的 or グローバル前処理（`clean_background_to_white` or スキップ）
/// 4. アップサンプリング or 閾値処理（`-2`/`-4`/なし）
pub fn binarize(
    pix: Pix,
    global: bool,
    bw_threshold: u8,
    up2: bool,
    up4: bool,
) -> Result<Pix, CliError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptonica::{PixMut, PixelDepth};

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
    #[ignore = "not yet implemented"]
    fn binarize_1bpp_returns_1bpp() {
        let pix = white_1bpp(32, 32);
        let result = binarize(pix, false, 200, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn binarize_1bpp_preserves_dimensions() {
        let pix = white_1bpp(40, 20);
        let result = binarize(pix, false, 200, false, false).unwrap();
        assert_eq!(result.width(), 40);
        assert_eq!(result.height(), 20);
    }

    // --- 8bpp → 1bpp 変換 ---

    #[test]
    #[ignore = "not yet implemented"]
    fn binarize_8bpp_returns_1bpp() {
        let pix = gray_8bpp(32, 32, 100);
        let result = binarize(pix, false, 200, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn binarize_8bpp_global_returns_1bpp() {
        let pix = gray_8bpp(32, 32, 100);
        let result = binarize(pix, true, 128, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    // --- 32bpp（RGB）→ 1bpp 変換 ---

    #[test]
    #[ignore = "not yet implemented"]
    fn binarize_32bpp_returns_1bpp() {
        let pix = rgb_32bpp(32, 32, 200, 200, 200);
        let result = binarize(pix, false, 200, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn binarize_32bpp_global_returns_1bpp() {
        let pix = rgb_32bpp(32, 32, 200, 200, 200);
        let result = binarize(pix, true, 128, false, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
    }

    // --- グローバル vs 適応的閾値 ---

    #[test]
    #[ignore = "not yet implemented"]
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
    #[ignore = "not yet implemented"]
    fn binarize_up2_doubles_dimensions() {
        let pix = gray_8bpp(20, 10, 150);
        let result = binarize(pix, true, 200, true, false).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert_eq!(result.width(), 40);
        assert_eq!(result.height(), 20);
    }

    // --- 4x アップサンプリング ---

    #[test]
    #[ignore = "not yet implemented"]
    fn binarize_up4_quadruples_dimensions() {
        let pix = gray_8bpp(10, 8, 150);
        let result = binarize(pix, true, 200, false, true).unwrap();
        assert_eq!(result.depth(), PixelDepth::Bit1);
        assert_eq!(result.width(), 40);
        assert_eq!(result.height(), 32);
    }

    // --- 非対応深さのエラー ---

    #[test]
    #[ignore = "not yet implemented"]
    fn binarize_16bpp_unsupported() {
        let pm = PixMut::new(16, 16, PixelDepth::Bit16).unwrap();
        let pix: Pix = pm.into();
        let result = binarize(pix, false, 200, false, false);
        // 16bpp は非対応
        assert!(result.is_err());
    }
}
