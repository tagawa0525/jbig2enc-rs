// CLI 統合テスト。
//
// `std::process::Command` でビルド済みバイナリを起動し、
// E2E パイプラインの結合を検証する。

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use leptonica::io::{ImageFormat, write_image};
use leptonica::{Pix, PixMut, PixelDepth};

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

fn jbig2_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jbig2enc"))
}

/// プロセス固有のテンポラリパスを生成する。
fn tmp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("jbig2enc_test_{}_{}", std::process::id(), name));
    p
}

/// 1bpp テスト用 PNG を書き出す。
///
/// 中央付近に黒いブロックを配置してシンボルモードのコンポーネント検出に対応する。
fn write_test_png(path: &PathBuf) {
    let mut pm = PixMut::new(64, 64, PixelDepth::Bit1).unwrap();
    // 20..44 x 20..44 の領域を黒に塗る（シンボル検出用）
    for y in 20u32..44 {
        for x in 20u32..44 {
            pm.set_pixel(x, y, 1).unwrap();
        }
    }
    let pix: Pix = pm.into();
    write_image(&pix, path, ImageFormat::Png).unwrap();
}

// ---------------------------------------------------------------------------
// Generic モード
// ---------------------------------------------------------------------------

/// Generic モード（デフォルト）: stdout に JBIG2 ファイルヘッダが出力される。
#[test]
fn generic_mode_outputs_jbig2_magic() {
    let img = tmp_path("generic.png");
    write_test_png(&img);

    let out = Command::new(jbig2_bin()).arg(&img).output().unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // JBIG2 ファイルヘッダのマジックバイト: 0x97 'J' 'B' '2'
    assert!(
        out.stdout.len() >= 4,
        "stdout too short: {} bytes",
        out.stdout.len()
    );
    assert_eq!(&out.stdout[0..4], b"\x97\x4a\x42\x32");

    let _ = fs::remove_file(&img);
}

/// Generic PDF モード: stdout に非空のデータが出力される（ファイルヘッダなし）。
#[test]
fn generic_pdf_mode_outputs_nonempty() {
    let img = tmp_path("generic_pdf.png");
    write_test_png(&img);

    let out = Command::new(jbig2_bin())
        .arg("-p")
        .arg(&img)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.stdout.is_empty());

    let _ = fs::remove_file(&img);
}

// ---------------------------------------------------------------------------
// Symbol モード（スタンドアロン）
// ---------------------------------------------------------------------------

/// Symbol スタンドアロンモード: stdout に JBIG2 ファイルヘッダが出力される。
#[test]
fn symbol_mode_standalone_outputs_jbig2_magic() {
    let img = tmp_path("symbol_sa.png");
    write_test_png(&img);

    let out = Command::new(jbig2_bin())
        .arg("-s")
        .arg(&img)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.len() >= 4);
    assert_eq!(&out.stdout[0..4], b"\x97\x4a\x42\x32");

    let _ = fs::remove_file(&img);
}

/// Symbol スタンドアロンモード マルチページ: 複数ページ入力でも成功する。
#[test]
fn symbol_mode_multipage_outputs_nonempty() {
    let img1 = tmp_path("mp1.png");
    let img2 = tmp_path("mp2.png");
    write_test_png(&img1);
    write_test_png(&img2);

    let out = Command::new(jbig2_bin())
        .arg("-s")
        .arg(&img1)
        .arg(&img2)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.stdout.is_empty());

    let _ = fs::remove_file(&img1);
    let _ = fs::remove_file(&img2);
}

// ---------------------------------------------------------------------------
// Symbol モード（PDF 出力）
// ---------------------------------------------------------------------------

/// Symbol PDF モード: `.sym` と `.0000` ファイルが生成される。
#[test]
fn symbol_pdf_mode_creates_output_files() {
    let img = tmp_path("pdf_mode.png");
    write_test_png(&img);

    let basename = tmp_path("pdf_out").to_string_lossy().to_string();
    let sym_path = format!("{basename}.sym");
    let page_path = format!("{basename}.0000");

    let out = Command::new(jbig2_bin())
        .arg("-s")
        .arg("-p")
        .arg("-b")
        .arg(&basename)
        .arg(&img)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(fs::metadata(&sym_path).is_ok(), "{sym_path} not found");
    assert!(fs::metadata(&page_path).is_ok(), "{page_path} not found");

    let _ = fs::remove_file(&img);
    let _ = fs::remove_file(&sym_path);
    let _ = fs::remove_file(&page_path);
}

// ---------------------------------------------------------------------------
// auto-threshold
// ---------------------------------------------------------------------------

/// `-a` フラグでの自動閾値処理が成功する。
#[test]
fn auto_threshold_mode_succeeds() {
    let img = tmp_path("auto_thresh.png");
    write_test_png(&img);

    let out = Command::new(jbig2_bin())
        .arg("-s")
        .arg("-a")
        .arg(&img)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.stdout.is_empty());

    let _ = fs::remove_file(&img);
}

/// `--no-hash` フラグ付き自動閾値処理が成功する。
#[test]
fn auto_threshold_no_hash_succeeds() {
    let img = tmp_path("auto_no_hash.png");
    write_test_png(&img);

    let out = Command::new(jbig2_bin())
        .arg("-s")
        .arg("-a")
        .arg("--no-hash")
        .arg(&img)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.stdout.is_empty());

    let _ = fs::remove_file(&img);
}

// ---------------------------------------------------------------------------
// -O（デバッグ PNG）
// ---------------------------------------------------------------------------

/// `-O` フラグでデバッグ PNG ファイルが出力される。
#[test]
fn output_threshold_writes_debug_png() {
    let img = tmp_path("thresh_in.png");
    let debug_png = tmp_path("thresh_debug.png");
    write_test_png(&img);

    let out = Command::new(jbig2_bin())
        .arg("-s")
        .arg("-a")
        .arg("-O")
        .arg(&debug_png)
        .arg(&img)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        fs::metadata(&debug_png).is_ok(),
        "debug PNG not found at {}",
        debug_png.to_string_lossy()
    );

    let _ = fs::remove_file(&img);
    let _ = fs::remove_file(&debug_png);
}

// ---------------------------------------------------------------------------
// -D（DPI 強制）
// ---------------------------------------------------------------------------

/// `-D` フラグで DPI を指定しても処理が成功する。
#[test]
fn dpi_flag_succeeds_on_image_without_dpi() {
    let img = tmp_path("dpi_in.png");
    write_test_png(&img);

    let basename = tmp_path("dpi_out").to_string_lossy().to_string();
    let sym_path = format!("{basename}.sym");
    let page_path = format!("{basename}.0000");

    let out = Command::new(jbig2_bin())
        .arg("-s")
        .arg("-p")
        .arg("-D")
        .arg("300")
        .arg("-b")
        .arg(&basename)
        .arg(&img)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(fs::metadata(&page_path).is_ok(), "{page_path} not found");

    let _ = fs::remove_file(&img);
    let _ = fs::remove_file(&sym_path);
    let _ = fs::remove_file(&page_path);
}

// ---------------------------------------------------------------------------
// エラーケース
// ---------------------------------------------------------------------------

/// 存在しない入力ファイルを指定した場合にエラー終了する。
#[test]
fn nonexistent_input_file_fails() {
    let img = tmp_path("no_such_file.png");
    // ファイルは作成しない

    let out = Command::new(jbig2_bin()).arg(&img).output().unwrap();

    assert!(
        !out.status.success(),
        "expected failure for nonexistent input"
    );
    assert!(!out.stderr.is_empty(), "expected error message on stderr");
}

/// 画像として不正なバイト列のファイルを指定した場合にエラー終了する。
#[test]
fn invalid_image_format_fails() {
    let img = tmp_path("bad_image.dat");
    fs::write(&img, b"this is not a valid image format").unwrap();

    let out = Command::new(jbig2_bin()).arg(&img).output().unwrap();

    assert!(!out.status.success(), "expected failure for invalid image");
    assert!(!out.stderr.is_empty(), "expected error message on stderr");

    let _ = fs::remove_file(&img);
}
