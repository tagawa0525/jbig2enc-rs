use jbig2enc::encoder::Jbig2Context;
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

/// "A" 風の矩形パターン（20x30）を含むページ。
fn page_with_a_pattern() -> leptonica::Pix {
    make_page_with_rects(
        200,
        100,
        &[(10, 10, 20, 30), (50, 10, 20, 30), (90, 10, 20, 30)],
    )
}

/// "B" 風の矩形パターン（15x30）を含むページ。
fn page_with_b_pattern() -> leptonica::Pix {
    make_page_with_rects(200, 100, &[(10, 10, 15, 30), (50, 10, 15, 30)])
}

/// 複数種類のシンボルを含むページ。
fn page_with_mixed_symbols() -> leptonica::Pix {
    make_page_with_rects(
        300,
        100,
        &[
            (10, 10, 20, 30),  // "A" 型
            (50, 10, 15, 30),  // "B" 型
            (90, 10, 20, 30),  // "A" 型
            (130, 10, 10, 25), // "C" 型
        ],
    )
}

// ---------------------------------------------------------------------------
// log2up テスト
// ---------------------------------------------------------------------------

/// log2up のテーブルテスト。
#[test]
fn log2up_table() {
    use jbig2enc::encoder::log2up;
    assert_eq!(log2up(0), 0);
    assert_eq!(log2up(1), 0);
    assert_eq!(log2up(2), 1);
    assert_eq!(log2up(3), 2);
    assert_eq!(log2up(4), 2);
    assert_eq!(log2up(5), 3);
    assert_eq!(log2up(8), 3);
    assert_eq!(log2up(9), 4);
    assert_eq!(log2up(16), 4);
    assert_eq!(log2up(17), 5);
    assert_eq!(log2up(256), 8);
    assert_eq!(log2up(257), 9);
}

// ---------------------------------------------------------------------------
// コンテキスト生成テスト
// ---------------------------------------------------------------------------

/// 有効なパラメータでコンテキストを生成できること。
#[test]
fn new_valid_params() {
    let ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1);
    assert!(ctx.is_ok());
}

/// thresh が範囲外（0.3）のときエラーになること。
#[test]
fn new_invalid_thresh() {
    let ctx = Jbig2Context::new(0.3, 0.5, 300, 300, true, -1);
    assert!(ctx.is_err());
}

/// weight が範囲外（1.5）のときエラーになること。
#[test]
fn new_invalid_weight() {
    let ctx = Jbig2Context::new(0.85, 1.5, 300, 300, true, -1);
    assert!(ctx.is_err());
}

// ---------------------------------------------------------------------------
// 単一ページ E2E テスト
// ---------------------------------------------------------------------------

/// 単一ページの符号化が成功し、非空の出力を生成すること。
#[test]
fn single_page_produces_output() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    let header = ctx.pages_complete().unwrap();
    assert!(!header.is_empty());
    let page = ctx.produce_page(0, None, None).unwrap();
    assert!(!page.is_empty());
}

/// full_headers=true のとき出力がJBIG2マジックバイトで始まること。
#[test]
fn full_headers_starts_with_magic() {
    let magic: [u8; 8] = [0x97, 0x4a, 0x42, 0x32, 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    let header = ctx.pages_complete().unwrap();
    assert!(header.starts_with(&magic));
}

/// full_headers=false のとき出力がJBIG2マジックバイトで始まらないこと。
#[test]
fn pdf_mode_no_magic() {
    let magic: [u8; 8] = [0x97, 0x4a, 0x42, 0x32, 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, false, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    let header = ctx.pages_complete().unwrap();
    assert!(!header.starts_with(&magic));
}

/// full_headers のとき FileHeader 内のページ数フィールドが正しいこと。
#[test]
fn page_count_in_header() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    ctx.add_page(&page_with_b_pattern()).unwrap();
    let header = ctx.pages_complete().unwrap();
    // FileHeader: magic(8) + flags(1) + n_pages(4)
    // n_pages は BE u32 で offset 9..13
    let n_pages = u32::from_be_bytes(header[9..13].try_into().unwrap());
    assert_eq!(n_pages, 2);
}

// ---------------------------------------------------------------------------
// マルチページ E2E テスト
// ---------------------------------------------------------------------------

/// 複数ページの符号化が成功すること。
#[test]
fn multipage_produces_output() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    ctx.add_page(&page_with_b_pattern()).unwrap();
    let header = ctx.pages_complete().unwrap();
    assert!(!header.is_empty());
    let page0 = ctx.produce_page(0, None, None).unwrap();
    assert!(!page0.is_empty());
    let page1 = ctx.produce_page(1, None, None).unwrap();
    assert!(!page1.is_empty());
}

/// 異なるシンボルパターンを持つ3ページの符号化が成功すること。
#[test]
fn three_pages_mixed_symbols() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    ctx.add_page(&page_with_b_pattern()).unwrap();
    ctx.add_page(&page_with_mixed_symbols()).unwrap();
    let header = ctx.pages_complete().unwrap();
    assert!(!header.is_empty());
    for i in 0..3 {
        let page = ctx.produce_page(i, None, None).unwrap();
        assert!(!page.is_empty(), "page {i} should not be empty");
    }
}

// ---------------------------------------------------------------------------
// 解像度オーバーライドテスト
// ---------------------------------------------------------------------------

/// produce_page で xres/yres を上書きすると出力が変わること。
#[test]
fn resolution_override_changes_output() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    ctx.pages_complete().unwrap();

    let page_default = ctx.produce_page(0, None, None).unwrap();

    // 新しいコンテキストで同じページを異なる解像度で符号化
    let mut ctx2 = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx2.add_page(&page_with_a_pattern()).unwrap();
    ctx2.pages_complete().unwrap();
    let page_override = ctx2.produce_page(0, Some(600), Some(600)).unwrap();

    assert_ne!(page_default, page_override);
}

// ---------------------------------------------------------------------------
// PDF モードのページ番号テスト
// ---------------------------------------------------------------------------

/// full_headers=false のとき全ページが page=1 で符号化されること。
/// full_headers=true のとき各ページが異なる page 値で符号化されること。
#[test]
fn pdf_mode_vs_full_headers_page_numbering() {
    // PDF mode
    let mut ctx_pdf = Jbig2Context::new(0.85, 0.5, 300, 300, false, -1).unwrap();
    ctx_pdf.add_page(&page_with_a_pattern()).unwrap();
    ctx_pdf.add_page(&page_with_b_pattern()).unwrap();
    ctx_pdf.pages_complete().unwrap();
    let page0_pdf = ctx_pdf.produce_page(0, None, None).unwrap();
    let page1_pdf = ctx_pdf.produce_page(1, None, None).unwrap();

    // full_headers mode
    let mut ctx_full = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx_full.add_page(&page_with_a_pattern()).unwrap();
    ctx_full.add_page(&page_with_b_pattern()).unwrap();
    ctx_full.pages_complete().unwrap();
    let page0_full = ctx_full.produce_page(0, None, None).unwrap();
    let page1_full = ctx_full.produce_page(1, None, None).unwrap();

    // 出力が異なることを確認（ページ番号が異なるため）
    assert_ne!(page0_pdf, page0_full);
    assert_ne!(page1_pdf, page1_full);
}

// ---------------------------------------------------------------------------
// エッジケース
// ---------------------------------------------------------------------------

/// pages_complete 前に produce_page を呼ぶとエラーになること。
#[test]
fn produce_page_before_complete_fails() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    assert!(ctx.produce_page(0, None, None).is_err());
}

/// 存在しないページ番号で produce_page を呼ぶとエラーになること。
#[test]
fn produce_page_invalid_page_no() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.add_page(&page_with_a_pattern()).unwrap();
    ctx.pages_complete().unwrap();
    assert!(ctx.produce_page(99, None, None).is_err());
}

// ---------------------------------------------------------------------------
// verbose テスト
// ---------------------------------------------------------------------------

/// verbose モードで pages_complete を実行しても正常な出力が得られること。
///
/// C++版 `jbig2enc.cc:662-665` に対応する統計出力のテスト。
/// verbose=true でも符号化結果はバイト一致する。
#[test]
fn verbose_pages_complete_produces_same_output() {
    // verbose=false で符号化
    let mut ctx1 = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx1.add_page(&page_with_a_pattern()).unwrap();
    let output1 = ctx1.pages_complete().unwrap();

    // verbose=true で符号化
    let mut ctx2 = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx2.set_verbose(true);
    ctx2.add_page(&page_with_a_pattern()).unwrap();
    let output2 = ctx2.pages_complete().unwrap();

    // verbose は stderr 出力のみ影響し、符号化結果は同一
    assert_eq!(output1, output2);
}

/// verbose モードの pages_complete がフォーマット通りの統計を stderr に出力すること。
///
/// 期待フォーマット:
/// "JBIG2 compression complete. pages:N symbols:M log2:L"
#[test]
fn verbose_output_format() {
    let mut ctx = Jbig2Context::new(0.85, 0.5, 300, 300, true, -1).unwrap();
    ctx.set_verbose(true);
    ctx.add_page(&page_with_a_pattern()).unwrap();

    let output = ctx.pages_complete().unwrap();
    assert!(!output.is_empty());

    // verbose 出力フォーマットの検証:
    // pages_complete 後に compression_stats() で統計文字列を取得できる
    let stats = ctx.compression_stats();
    assert!(
        stats.contains("pages:1"),
        "stats should contain page count: {stats}"
    );
    assert!(
        stats.contains("symbols:"),
        "stats should contain symbol count: {stats}"
    );
    assert!(
        stats.contains("log2:"),
        "stats should contain log2: {stats}"
    );
}
