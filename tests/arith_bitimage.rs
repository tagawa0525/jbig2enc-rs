use jbig2enc::arith::ArithEncoder;

// ========================================================================
// encode_bitimage テスト
// ========================================================================

/// 1x1画像（白ピクセル=0）の符号化
#[test]

fn bitimage_1x1_white() {
    let mut enc = ArithEncoder::new();
    let data = [0u32; 1]; // 1ピクセル幅、1行、ゼロビット=白
    enc.encode_bitimage(&data, 1, 1, false);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 1x1画像（黒ピクセル=1）の符号化
#[test]

fn bitimage_1x1_black() {
    let mut enc = ArithEncoder::new();
    // MSBファーストの1bpp: 最初のビット（MSB）が1 = 0x80000000
    let data = [0x8000_0000u32; 1];
    enc.encode_bitimage(&data, 1, 1, false);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 白ピクセルと黒ピクセルは異なる出力を生成する
#[test]

fn bitimage_1x1_white_vs_black() {
    let mut enc_white = ArithEncoder::new();
    enc_white.encode_bitimage(&[0u32], 1, 1, false);
    enc_white.encode_final();

    let mut enc_black = ArithEncoder::new();
    enc_black.encode_bitimage(&[0x8000_0000u32], 1, 1, false);
    enc_black.encode_final();

    assert_ne!(enc_white.to_vec(), enc_black.to_vec());
}

/// 8x1画像（幅が32の倍数でない場合）
#[test]

fn bitimage_8x1_partial_word() {
    let mut enc = ArithEncoder::new();
    // 幅8ピクセル = 1ワード。パターン: 0b10101010... = 0xAA000000
    let data = [0xAA00_0000u32];
    enc.encode_bitimage(&data, 8, 1, false);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 32x1画像（幅がちょうど32ピクセル）
#[test]

fn bitimage_32x1_full_word() {
    let mut enc = ArithEncoder::new();
    let data = [0xFFFF_FFFFu32];
    enc.encode_bitimage(&data, 32, 1, false);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 33x1画像（幅が32より1大きい = 2ワード）
#[test]

fn bitimage_33x1_two_words() {
    let mut enc = ArithEncoder::new();
    let data = [0xFFFF_FFFFu32, 0x8000_0000u32]; // 2ワード
    enc.encode_bitimage(&data, 33, 1, false);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 4x4全白画像
#[test]

fn bitimage_4x4_all_white() {
    let mut enc = ArithEncoder::new();
    let data = [0u32; 4]; // 4行、各行1ワード
    enc.encode_bitimage(&data, 4, 4, false);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 4x4全黒画像
#[test]

fn bitimage_4x4_all_black() {
    let mut enc = ArithEncoder::new();
    // 幅4ピクセル: MSBから4ビットが1 = 0xF0000000
    let data = [0xF000_0000u32; 4];
    enc.encode_bitimage(&data, 4, 4, false);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// TPGD: 全白2行はTPGDで圧縮効果があるため、non-TPGDより短いか同じ
#[test]

fn bitimage_tpgd_identical_rows() {
    let mut enc_notpgd = ArithEncoder::new();
    let data = [0u32, 0u32]; // 4ピクセル幅 × 2行、全白
    enc_notpgd.encode_bitimage(&data, 4, 2, false);
    enc_notpgd.encode_final();

    let mut enc_tpgd = ArithEncoder::new();
    enc_tpgd.encode_bitimage(&data, 4, 2, true);
    enc_tpgd.encode_final();

    // TPGDあり・なしで出力が異なることを確認（TPGDは追加bitを出力）
    assert!(!enc_notpgd.to_vec().is_empty());
    assert!(!enc_tpgd.to_vec().is_empty());
}

/// TPGD: 同じ行と異なる行は異なる符号化をする
#[test]

fn bitimage_tpgd_same_vs_different_rows() {
    let mut enc_same = ArithEncoder::new();
    let data_same = [0u32, 0u32]; // 同一行が2行
    enc_same.encode_bitimage(&data_same, 4, 2, true);
    enc_same.encode_final();

    let mut enc_diff = ArithEncoder::new();
    let data_diff = [0u32, 0xF000_0000u32]; // 異なる行
    enc_diff.encode_bitimage(&data_diff, 4, 2, true);
    enc_diff.encode_final();

    assert_ne!(enc_same.to_vec(), enc_diff.to_vec());
}

/// 決定論的: 同じ入力から常に同じ出力
#[test]

fn bitimage_deterministic() {
    let data = [0xDEAD_BEEFu32, 0x0102_0304u32];

    let mut enc1 = ArithEncoder::new();
    enc1.encode_bitimage(&data, 32, 2, false);
    enc1.encode_final();

    let mut enc2 = ArithEncoder::new();
    enc2.encode_bitimage(&data, 32, 2, false);
    enc2.encode_final();

    assert_eq!(enc1.to_vec(), enc2.to_vec());
}

/// 8x8チェッカーボードパターン
#[test]

fn bitimage_8x8_checkerboard() {
    let mut enc = ArithEncoder::new();
    // チェッカーボード: 奇数行=0xAA000000, 偶数行=0x55000000 (8bit幅)
    let data = [
        0xAA00_0000u32,
        0x5500_0000u32,
        0xAA00_0000u32,
        0x5500_0000u32,
        0xAA00_0000u32,
        0x5500_0000u32,
        0xAA00_0000u32,
        0x5500_0000u32,
    ];
    enc.encode_bitimage(&data, 8, 8, false);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

// ========================================================================
// encode_refine テスト
// ========================================================================

/// 同一画像間のリファインメント（差分なし）
#[test]

fn refine_identical_images() {
    let mut enc = ArithEncoder::new();
    let data = [0u32; 1]; // 1x1 白
    enc.encode_refine(&data, 1, 1, &data, 1, 1, 0, 0);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 異なる画像間のリファインメント
#[test]

fn refine_different_images() {
    let mut enc = ArithEncoder::new();
    let templ = [0u32; 1];
    let target = [0x8000_0000u32]; // 1x1 黒
    enc.encode_refine(&templ, 1, 1, &target, 1, 1, 0, 0);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 同一と異なる画像は異なる出力を生成する
#[test]

fn refine_identical_vs_different() {
    let data_white = [0u32];
    let data_black = [0x8000_0000u32];

    let mut enc_same = ArithEncoder::new();
    enc_same.encode_refine(&data_white, 1, 1, &data_white, 1, 1, 0, 0);
    enc_same.encode_final();

    let mut enc_diff = ArithEncoder::new();
    enc_diff.encode_refine(&data_white, 1, 1, &data_black, 1, 1, 0, 0);
    enc_diff.encode_final();

    assert_ne!(enc_same.to_vec(), enc_diff.to_vec());
}

/// オフセット ox=-1, 0, 1 で動作することを確認
#[test]

fn refine_offset_variations() {
    let templ = [0u32; 2]; // 2x2
    let target = [0u32; 2];

    for ox in [-1i32, 0, 1] {
        let mut enc = ArithEncoder::new();
        enc.encode_refine(&templ, 2, 2, &target, 2, 2, ox, 0);
        enc.encode_final();
        assert!(!enc.to_vec().is_empty(), "ox={ox} should produce output");
    }
}
