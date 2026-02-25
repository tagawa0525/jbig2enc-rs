use jbig2enc_rs::comparator::are_equivalent;
use leptonica::{Pix, PixMut, PixelDepth};

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

/// 全白1bppシンボルを作成する。
fn white_pix(w: u32, h: u32) -> Pix {
    PixMut::new(w, h, PixelDepth::Bit1).unwrap().into()
}

/// 全黒1bppシンボルを作成する。
fn black_pix(w: u32, h: u32) -> Pix {
    let mut pm = PixMut::new(w, h, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    pm.into()
}

/// 指定座標にピクセルを描画した1bppシンボルを作成する。
fn pix_with_pixels(w: u32, h: u32, pixels: &[(u32, u32)]) -> Pix {
    let mut pm = PixMut::new(w, h, PixelDepth::Bit1).unwrap();
    for &(x, y) in pixels {
        pm.set_pixel(x, y, 1).unwrap();
    }
    pm.into()
}

// ---------------------------------------------------------------------------
// 基本動作テスト
// ---------------------------------------------------------------------------

/// 同一の全白画像は等価。
#[test]
fn identical_white_images() {
    let a = white_pix(36, 36);
    let b = white_pix(36, 36);
    assert!(are_equivalent(&a, &b).unwrap());
}

/// 同一の全黒画像は等価。
#[test]
fn identical_black_images() {
    let a = black_pix(36, 36);
    let b = black_pix(36, 36);
    assert!(are_equivalent(&a, &b).unwrap());
}

/// 同一のパターンを持つ画像は等価。
#[test]
fn identical_pattern() {
    let pixels: Vec<(u32, u32)> = (0..36).map(|i| (i, i)).collect();
    let a = pix_with_pixels(36, 36, &pixels);
    let b = pix_with_pixels(36, 36, &pixels);
    assert!(are_equivalent(&a, &b).unwrap());
}

// ---------------------------------------------------------------------------
// サイズ不一致テスト
// ---------------------------------------------------------------------------

/// 幅が異なる画像は非等価。
#[test]
fn different_width() {
    let a = white_pix(36, 36);
    let b = white_pix(37, 36);
    assert!(!are_equivalent(&a, &b).unwrap());
}

/// 高さが異なる画像は非等価。
#[test]
fn different_height() {
    let a = white_pix(36, 36);
    let b = white_pix(36, 37);
    assert!(!are_equivalent(&a, &b).unwrap());
}

// ---------------------------------------------------------------------------
// XOR差分閾値テスト
// ---------------------------------------------------------------------------

/// 全白 vs 全黒は差分が大きすぎて非等価（25%閾値で棄却）。
#[test]
fn white_vs_black_rejected() {
    let a = black_pix(36, 36);
    let b = white_pix(36, 36);
    assert!(!are_equivalent(&a, &b).unwrap());
}

/// 微小差分（数ピクセル）は閾値内で等価。
/// 36x36の黒画像に対し、2ピクセルだけ異なる → XOR差分2。
/// pcount = 36*36 = 1296, threshold = 324。2 < 324 → パス。
#[test]
fn small_difference_accepted() {
    let a = black_pix(36, 36);
    // bは36x36の黒画像から2ピクセルだけ白にしたもの
    let mut pm = PixMut::new(36, 36, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    pm.set_pixel(5, 5, 0).unwrap();
    pm.set_pixel(30, 30, 0).unwrap();
    let b: Pix = pm.into();
    assert!(are_equivalent(&a, &b).unwrap());
}

// ---------------------------------------------------------------------------
// 全白画像の特殊ケース
// ---------------------------------------------------------------------------

/// 両方全白（ONピクセル0）の場合。
/// pcount=0 → threshold=0 → threshold_pixel_sum(0) は XOR=0 で false (not above)。
/// グリッド分析でも全セル0 → 全チェックパス → 等価。
#[test]
fn both_all_white() {
    let a = white_pix(36, 36);
    let b = white_pix(36, 36);
    assert!(are_equivalent(&a, &b).unwrap());
}

/// 片方全白・片方に少数ピクセル。
/// pcount=0 → threshold=0 → XOR差分が1以上ならthreshold超過 → false。
#[test]
fn white_vs_few_pixels_rejected() {
    let a = white_pix(36, 36);
    let b = pix_with_pixels(36, 36, &[(18, 18)]);
    assert!(!are_equivalent(&a, &b).unwrap());
}

// ---------------------------------------------------------------------------
// グリッド分析テスト（集中差分）
// ---------------------------------------------------------------------------

/// 1つの9x9グリッドセル内に差分が集中している場合、point_thresh チェックで棄却される。
/// 36x36画像、9x9グリッド → 各セル4x4ピクセル。
/// point_thresh = a * b * PI = 2 * 2 * 3.14... ≈ 12.56。
/// 2x2ブロック合計が13以上なら棄却。
///
/// 左上2x2セル（8x8ピクセル）内に差分を16ピクセル配置:
/// 2x2ブロック合計 = 16 >= 12.56 → 棄却。
#[test]
fn concentrated_difference_rejected() {
    // firstは全黒36x36
    let first = black_pix(36, 36);
    // secondは全黒36x36だが左上8x8領域を白にする
    let mut pm = PixMut::new(36, 36, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    for y in 0..8 {
        for x in 0..8 {
            pm.set_pixel(x, y, 0).unwrap();
        }
    }
    let second: Pix = pm.into();
    // XOR差分が左上に集中 → point_threshで棄却
    assert!(!are_equivalent(&first, &second).unwrap());
}

// ---------------------------------------------------------------------------
// 水平線パターンテスト
// ---------------------------------------------------------------------------

/// 水平方向に帯状の差分 → hline_thresh チェックで棄却。
/// 36x36画像の中央付近に水平帯（幅36、高さ4）の差分を配置。
#[test]
fn horizontal_line_difference_rejected() {
    let first = black_pix(36, 36);
    let mut pm = PixMut::new(36, 36, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    // 中央付近（y=16..20）に水平帯の差分
    for y in 16..20 {
        for x in 0..36 {
            pm.set_pixel(x, y, 0).unwrap();
        }
    }
    let second: Pix = pm.into();
    assert!(!are_equivalent(&first, &second).unwrap());
}

// ---------------------------------------------------------------------------
// 垂直線パターンテスト
// ---------------------------------------------------------------------------

/// 垂直方向に帯状の差分 → vline_thresh チェックで棄却。
/// 36x36画像の中央付近に垂直帯（高さ36、幅4）の差分を配置。
#[test]
fn vertical_line_difference_rejected() {
    let first = black_pix(36, 36);
    let mut pm = PixMut::new(36, 36, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    // 中央付近（x=16..20）に垂直帯の差分
    for y in 0..36 {
        for x in 16..20 {
            pm.set_pixel(x, y, 0).unwrap();
        }
    }
    let second: Pix = pm.into();
    assert!(!are_equivalent(&first, &second).unwrap());
}

// ---------------------------------------------------------------------------
// 交差線パターンテスト
// ---------------------------------------------------------------------------

/// 対角線方向に差分が分布 → 交差線チェック（Check 3）で棄却。
/// 36x36画像、9x9グリッド → 各セル4x4ピクセル。
/// hline_thresh = (4 * (4/2)) * 0.9 = 7.2 → 7。
/// 対角3セル (0,0),(1,1),(2,2) に各3ピクセルの差分 → left_cross = 9 >= 7 → 棄却。
#[test]
fn diagonal_difference_rejected() {
    let first = black_pix(36, 36);
    let mut pm = PixMut::new(36, 36, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    // セル(0,0): x=[0,4), y=[0,4) → 3ピクセル差分
    pm.set_pixel(0, 0, 0).unwrap();
    pm.set_pixel(1, 1, 0).unwrap();
    pm.set_pixel(2, 2, 0).unwrap();
    // セル(1,1): x=[4,8), y=[4,8) → 3ピクセル差分
    pm.set_pixel(4, 4, 0).unwrap();
    pm.set_pixel(5, 5, 0).unwrap();
    pm.set_pixel(6, 6, 0).unwrap();
    // セル(2,2): x=[8,12), y=[8,12) → 3ピクセル差分
    pm.set_pixel(8, 8, 0).unwrap();
    pm.set_pixel(9, 9, 0).unwrap();
    pm.set_pixel(10, 10, 0).unwrap();
    let second: Pix = pm.into();
    assert!(!are_equivalent(&first, &second).unwrap());
}

// ---------------------------------------------------------------------------
// エッジケース
// ---------------------------------------------------------------------------

/// 最小サイズ画像（9x9）の同一ペア。
/// divider=9 → 各セル1x1 → hline_thresh=(1*(1/2))*0.9=0。
/// 0 >= 0 が true になるため、同一画像でも非等価を返す（C++と同一動作）。
#[test]
fn minimal_size_9x9_degenerate_threshold() {
    let a = black_pix(9, 9);
    let b = black_pix(9, 9);
    assert!(!are_equivalent(&a, &b).unwrap());
}

/// dividerで割り切れないサイズ（37x37）。
/// 余りピクセルが先頭セルに分配される。
#[test]
fn non_divisible_size() {
    let a = black_pix(37, 37);
    let b = black_pix(37, 37);
    assert!(are_equivalent(&a, &b).unwrap());
}

/// 9未満のサイズ（5x5）の同一ペア。
/// vertical_part = 5/9 = 0, horizontal_part = 5/9 = 0 → 全閾値が0。
/// 0 >= 0 が true になるため非等価（C++と同一動作）。
#[test]
fn tiny_image_below_9_degenerate_threshold() {
    let a = black_pix(5, 5);
    let b = black_pix(5, 5);
    assert!(!are_equivalent(&a, &b).unwrap());
}

/// 幅が32の倍数でない場合のWPL一致確認。
#[test]
fn non_32_aligned_width() {
    let a = black_pix(33, 20);
    let b = black_pix(33, 20);
    assert!(are_equivalent(&a, &b).unwrap());
}
