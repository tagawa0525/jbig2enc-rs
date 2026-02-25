use jbig2enc_rs::encoder::Jbig2Context;
use leptonica::{PixMut, PixelDepth};

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

/// テスト用の1bpp画像を作成する。指定座標に黒矩形を配置。
fn make_page_with_rects(width: u32, height: u32, rects: &[(u32, u32, u32, u32)]) -> leptonica::Pix {
    let mut pm = PixMut::new(width, height, PixelDepth::Bit1).unwrap();
    for &(rx, ry, rw, rh) in rects {
        for dy in 0..rh {
            for dx in 0..rw {
                pm.set_pixel(rx + dx, ry + dy, 1).unwrap();
            }
        }
    }
    pm.into()
}

/// 複数の同一形状シンボルを含むページ。
fn page_with_identical_symbols() -> leptonica::Pix {
    make_page_with_rects(
        300,
        100,
        &[
            (10, 10, 20, 30),
            (50, 10, 20, 30),
            (90, 10, 20, 30),
            (130, 10, 20, 30),
            (170, 10, 20, 30),
        ],
    )
}

/// 複数種類のシンボルを含むページ。
fn page_with_varied_symbols() -> leptonica::Pix {
    make_page_with_rects(
        300,
        100,
        &[
            (10, 10, 20, 30),
            (50, 10, 15, 25),
            (90, 10, 20, 30),
            (130, 10, 10, 20),
            (170, 10, 15, 25),
        ],
    )
}

// ---------------------------------------------------------------------------
// auto_threshold 基本テスト
// ---------------------------------------------------------------------------

/// auto_threshold を呼んでもパニックしないこと。
#[test]
#[ignore = "not yet implemented"]
fn auto_threshold_no_panic() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_identical_symbols()).unwrap();
    ctx.auto_threshold();
}

/// auto_threshold 後も pages_complete + produce_page が成功すること。
#[test]
#[ignore = "not yet implemented"]
fn auto_threshold_then_encode() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_identical_symbols()).unwrap();
    ctx.auto_threshold();
    let header = ctx.pages_complete().unwrap();
    assert!(!header.is_empty());
    let page = ctx.produce_page(0, None, None).unwrap();
    assert!(!page.is_empty());
}

/// auto_threshold をマルチページで呼んでも正しく動作すること。
#[test]
#[ignore = "not yet implemented"]
fn auto_threshold_multipage() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_identical_symbols()).unwrap();
    ctx.add_page(&page_with_varied_symbols()).unwrap();
    ctx.auto_threshold();
    let header = ctx.pages_complete().unwrap();
    assert!(!header.is_empty());
    for i in 0..2 {
        let page = ctx.produce_page(i, None, None).unwrap();
        assert!(!page.is_empty(), "page {i} should not be empty");
    }
}

// ---------------------------------------------------------------------------
// auto_threshold_using_hash 基本テスト
// ---------------------------------------------------------------------------

/// auto_threshold_using_hash を呼んでもパニックしないこと。
#[test]
#[ignore = "not yet implemented"]
fn auto_threshold_using_hash_no_panic() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_identical_symbols()).unwrap();
    ctx.auto_threshold_using_hash();
}

/// auto_threshold_using_hash 後も pages_complete + produce_page が成功すること。
#[test]
#[ignore = "not yet implemented"]
fn auto_threshold_using_hash_then_encode() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_identical_symbols()).unwrap();
    ctx.auto_threshold_using_hash();
    let header = ctx.pages_complete().unwrap();
    assert!(!header.is_empty());
    let page = ctx.produce_page(0, None, None).unwrap();
    assert!(!page.is_empty());
}

/// auto_threshold_using_hash をマルチページで呼んでも正しく動作すること。
#[test]
#[ignore = "not yet implemented"]
fn auto_threshold_using_hash_multipage() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_identical_symbols()).unwrap();
    ctx.add_page(&page_with_varied_symbols()).unwrap();
    ctx.auto_threshold_using_hash();
    let header = ctx.pages_complete().unwrap();
    assert!(!header.is_empty());
    for i in 0..2 {
        let page = ctx.produce_page(i, None, None).unwrap();
        assert!(!page.is_empty(), "page {i} should not be empty");
    }
}

// ---------------------------------------------------------------------------
// 両手法の一貫性テスト
// ---------------------------------------------------------------------------

/// 両手法で符号化後の出力が同一であること（同一入力に対して）。
#[test]
#[ignore = "not yet implemented"]
fn both_methods_produce_same_output() {
    // auto_threshold
    let mut ctx1 = Jbig2Context::new(0.85, 0.5, 300, 300, false, -1).unwrap();
    ctx1.add_page(&page_with_varied_symbols()).unwrap();
    ctx1.auto_threshold();
    let h1 = ctx1.pages_complete().unwrap();
    let p1 = ctx1.produce_page(0, None, None).unwrap();

    // auto_threshold_using_hash
    let mut ctx2 = Jbig2Context::new(0.85, 0.5, 300, 300, false, -1).unwrap();
    ctx2.add_page(&page_with_varied_symbols()).unwrap();
    ctx2.auto_threshold_using_hash();
    let h2 = ctx2.pages_complete().unwrap();
    let p2 = ctx2.produce_page(0, None, None).unwrap();

    assert_eq!(h1, h2);
    assert_eq!(p1, p2);
}
