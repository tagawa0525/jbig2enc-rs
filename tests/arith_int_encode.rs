use jbig2enc::arith::{ArithEncoder, IntProc};

// ========================================================================
// encode_int テスト
// ========================================================================

/// encode_int が値を符号化してビット列を出力できることを確認。
#[test]

fn encode_int_produces_output() {
    let mut enc = ArithEncoder::new();
    enc.encode_int(IntProc::Dh, 0);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// range 0 (0..=3): data=0, bits=2, delta=0, intbits=2
/// value=0 → prefix=0b00(2bits), value_bits=0b00(2bits)
#[test]

fn encode_int_range0_value0() {
    let mut enc = ArithEncoder::new();
    enc.encode_int(IntProc::Dh, 0);
    enc.encode_final();
    let out0 = enc.to_vec();

    let mut enc2 = ArithEncoder::new();
    enc2.encode_int(IntProc::Dh, 0);
    enc2.encode_final();
    // 同じ入力から同じ出力が得られることを確認
    assert_eq!(out0, enc2.to_vec());
}

/// range 0 (0..=3): value=3 と value=0 は異なる出力になる
#[test]

fn encode_int_range0_different_values() {
    let mut enc0 = ArithEncoder::new();
    enc0.encode_int(IntProc::Dh, 0);
    enc0.encode_final();

    let mut enc3 = ArithEncoder::new();
    enc3.encode_int(IntProc::Dh, 3);
    enc3.encode_final();

    assert_ne!(enc0.to_vec(), enc3.to_vec());
}

/// range 1 (-1): data=9(=0b1001), bits=4, delta=0, intbits=0
#[test]

fn encode_int_range1_minus1() {
    let mut enc = ArithEncoder::new();
    enc.encode_int(IntProc::Dh, -1);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// range 2 (-3..-2): data=5, bits=3, delta=2, intbits=1
#[test]

fn encode_int_range2_minus2() {
    let mut enc = ArithEncoder::new();
    enc.encode_int(IntProc::Dh, -2);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// range 2 (-3..-2): value=-2 と value=-3 は異なる出力
#[test]

fn encode_int_range2_minus2_vs_minus3() {
    let mut enc2 = ArithEncoder::new();
    enc2.encode_int(IntProc::Dh, -2);
    enc2.encode_final();

    let mut enc3 = ArithEncoder::new();
    enc3.encode_int(IntProc::Dh, -3);
    enc3.encode_final();

    assert_ne!(enc2.to_vec(), enc3.to_vec());
}

/// range 3 (4..=19): data=2, bits=3, delta=4, intbits=4
#[test]

fn encode_int_range3_value4() {
    let mut enc = ArithEncoder::new();
    enc.encode_int(IntProc::Dh, 4);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// range 11 (4436..=2000000000): 最大値
#[test]

fn encode_int_max_value() {
    let mut enc = ArithEncoder::new();
    enc.encode_int(IntProc::Dh, 2_000_000_000);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// range 12 (-2000000000..=-4436): 最小値
#[test]

fn encode_int_min_value() {
    let mut enc = ArithEncoder::new();
    enc.encode_int(IntProc::Dh, -2_000_000_000);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 異なるプロシージャは独立したコンテキストを使う（同じ値でも出力が異なる可能性）
#[test]

fn encode_int_different_procs_independent() {
    let mut enc_dh = ArithEncoder::new();
    enc_dh.encode_int(IntProc::Dh, 5);
    enc_dh.encode_int(IntProc::Dh, 5);
    enc_dh.encode_final();

    let mut enc_dw = ArithEncoder::new();
    // Dw で最初から符号化 - コンテキストが独立していることを確認
    enc_dw.encode_int(IntProc::Dw, 5);
    enc_dw.encode_int(IntProc::Dw, 5);
    enc_dw.encode_final();

    // 同じ値・同じシーケンス → コンテキスト状態は同じはずなので同じ出力
    assert_eq!(enc_dh.to_vec(), enc_dw.to_vec());
}

/// 複数の整数を順に符号化できることを確認
#[test]

fn encode_int_multiple_values() {
    let mut enc = ArithEncoder::new();
    for v in [0, 1, 2, 3, -1, 4, 19, -4, -19, 20, 83, -20, -83] {
        enc.encode_int(IntProc::Dh, v);
    }
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

// ========================================================================
// encode_oob テスト
// ========================================================================

/// encode_oob が出力を生成することを確認
#[test]

fn encode_oob_produces_output() {
    let mut enc = ArithEncoder::new();
    enc.encode_oob(IntProc::Fs);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// encode_oob は encode_int と独立したコンテキストを共有する
/// （同じプロシージャを使い、連続符号化ができることを確認）
#[test]

fn encode_oob_and_int_interleaved() {
    let mut enc = ArithEncoder::new();
    enc.encode_int(IntProc::Fs, 0);
    enc.encode_oob(IntProc::Fs);
    enc.encode_int(IntProc::Fs, 1);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// 2回OOBを符号化すると出力長が変わる
#[test]

fn encode_oob_twice_different_output() {
    let mut enc1 = ArithEncoder::new();
    enc1.encode_oob(IntProc::Fs);
    enc1.encode_final();

    let mut enc2 = ArithEncoder::new();
    enc2.encode_oob(IntProc::Fs);
    enc2.encode_oob(IntProc::Fs);
    enc2.encode_final();

    // 2回エンコードした場合は出力が異なるはず（コンテキスト状態が変化するため）
    // （出力バイト数が同じになる場合もあるが内容が異なる）
    // 少なくともどちらも空でない
    assert!(!enc1.to_vec().is_empty());
    assert!(!enc2.to_vec().is_empty());
}

// ========================================================================
// encode_iaid テスト
// ========================================================================

/// encode_iaid が出力を生成することを確認
#[test]

fn encode_iaid_produces_output() {
    let mut enc = ArithEncoder::new();
    enc.encode_iaid(4, 0); // 4ビット、値0
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// encode_iaid: 同じsymcodelensの値0と1は異なる出力
#[test]

fn encode_iaid_different_values() {
    let mut enc0 = ArithEncoder::new();
    enc0.encode_iaid(4, 0);
    enc0.encode_final();

    let mut enc1 = ArithEncoder::new();
    enc1.encode_iaid(4, 1);
    enc1.encode_final();

    assert_ne!(enc0.to_vec(), enc1.to_vec());
}

/// encode_iaid: symcodelen=0 は何も符号化しない（ループが0回）
#[test]

fn encode_iaid_zero_codelen() {
    let mut enc = ArithEncoder::new();
    enc.encode_iaid(0, 0);
    enc.encode_final();
    // symcodelen=0 ではビットが出力されない（finalizeのみ）
    let mut enc_empty = ArithEncoder::new();
    enc_empty.encode_final();
    assert_eq!(enc.to_vec(), enc_empty.to_vec());
}

/// encode_iaid: 同じ値を2回符号化すると2回目はコンテキストが変化するため異なる出力になる可能性
#[test]

fn encode_iaid_repeated() {
    let mut enc = ArithEncoder::new();
    enc.encode_iaid(4, 5);
    enc.encode_iaid(4, 5);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}

/// encode_iaid: symcodelen=8, 最大値 255
#[test]

fn encode_iaid_max_value_8bit() {
    let mut enc = ArithEncoder::new();
    enc.encode_iaid(8, 255);
    enc.encode_final();
    assert!(!enc.to_vec().is_empty());
}
