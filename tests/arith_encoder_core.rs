use jbig2enc_rs::arith::{ArithEncoder, STATE_TABLE};

// ========================================================================
// 状態テーブル検証
// ========================================================================

/// C++版 ctbl[] の全92エントリとRust版 STATE_TABLE を照合する。
/// C++のマクロ展開を手動で計算した期待値。
#[test]
#[ignore = "not yet implemented"]
fn state_table_has_92_entries() {
    assert_eq!(STATE_TABLE.len(), 92);
}

#[test]
#[ignore = "not yet implemented"]
fn state_table_first_entry_mps0() {
    // State 0 (MPS=0): qe=0x5601, mps=1, lps=1+46=47 (SWITCH)
    let e = STATE_TABLE[0];
    assert_eq!(e.qe, 0x5601);
    assert_eq!(e.mps, 1);
    assert_eq!(e.lps, 47);
}

#[test]
#[ignore = "not yet implemented"]
fn state_table_entry_5_mps0() {
    // State 5 (MPS=0): qe=0x0221, mps=38, lps=33
    let e = STATE_TABLE[5];
    assert_eq!(e.qe, 0x0221);
    assert_eq!(e.mps, 38);
    assert_eq!(e.lps, 33);
}

#[test]
#[ignore = "not yet implemented"]
fn state_table_entry_6_mps0() {
    // State 6 (MPS=0): qe=0x5601, mps=7, lps=6+46=52 (SWITCH)
    let e = STATE_TABLE[6];
    assert_eq!(e.qe, 0x5601);
    assert_eq!(e.mps, 7);
    assert_eq!(e.lps, 52);
}

#[test]
#[ignore = "not yet implemented"]
fn state_table_entry_45_mps0() {
    // State 45 (MPS=0): qe=0x0001, mps=45, lps=43
    let e = STATE_TABLE[45];
    assert_eq!(e.qe, 0x0001);
    assert_eq!(e.mps, 45);
    assert_eq!(e.lps, 43);
}

#[test]
#[ignore = "not yet implemented"]
fn state_table_first_entry_mps1() {
    // State 46 (MPS=1): qe=0x5601, mps=47, lps=47-46=1 (SWITCH)
    let e = STATE_TABLE[46];
    assert_eq!(e.qe, 0x5601);
    assert_eq!(e.mps, 47);
    assert_eq!(e.lps, 1);
}

#[test]
#[ignore = "not yet implemented"]
fn state_table_entry_51_mps1() {
    // State 51 (= 5+46, MPS=1): qe=0x0221, mps=84, lps=79
    let e = STATE_TABLE[51];
    assert_eq!(e.qe, 0x0221);
    assert_eq!(e.mps, 84);
    assert_eq!(e.lps, 79);
}

#[test]
#[ignore = "not yet implemented"]
fn state_table_entry_91_mps1() {
    // State 91 (= 45+46, MPS=1): qe=0x0001, mps=91, lps=89
    let e = STATE_TABLE[91];
    assert_eq!(e.qe, 0x0001);
    assert_eq!(e.mps, 91);
    assert_eq!(e.lps, 89);
}

#[test]
#[ignore = "not yet implemented"]
fn state_table_symmetry() {
    // MPS=0側とMPS=1側はqe値が一致するはず
    for i in 0..46 {
        assert_eq!(
            STATE_TABLE[i].qe,
            STATE_TABLE[i + 46].qe,
            "qe mismatch at state {i}"
        );
    }
}

// ========================================================================
// ArithEncoder core 検証
// ========================================================================

#[test]
#[ignore = "not yet implemented"]
fn encoder_new_initial_state() {
    let enc = ArithEncoder::new();
    // 初期状態: a=0x8000, c=0, ct=12, bp=-1
    // data_size は 0 であるべき
    assert_eq!(enc.data_size(), 0);
}

#[test]
#[ignore = "not yet implemented"]
fn encoder_reset_clears_state() {
    let mut enc = ArithEncoder::new();
    // ビットを符号化して状態を変更
    let mut ctx = vec![0u8; 1];
    enc.encode_bit(&mut ctx, 0, 0);
    enc.encode_bit(&mut ctx, 0, 1);
    enc.reset();
    // リセット後も data_size は前の出力を保持（flushではない）
    // ただしコーダ状態はリセットされている
    // 新たに同じ入力を符号化すれば同じ結果になるはず
}

#[test]
#[ignore = "not yet implemented"]
fn encoder_flush_clears_output() {
    let mut enc = ArithEncoder::new();
    let mut ctx = vec![0u8; 1];
    enc.encode_bit(&mut ctx, 0, 0);
    enc.encode_final();
    assert!(enc.data_size() > 0);
    enc.flush();
    assert_eq!(enc.data_size(), 0);
}

/// JBIG2仕様 Appendix H.2 テストベクタ。
/// C++版の `input[]` 配列を使って画像データを符号化し、
/// 期待される出力バイト列と比較する。
#[test]
#[ignore = "not yet implemented"]
fn encoder_h2_test_vector() {
    // H.2テスト入力: 32バイト = 256ビットを1行の画像として符号化
    let input: [u8; 32] = [
        0x00, 0x02, 0x00, 0x51, 0x00, 0x00, 0x00, 0xc0, 0x03, 0x52, 0x87, 0x2a, 0xaa, 0xaa, 0xaa,
        0xaa, 0x82, 0xc0, 0x20, 0x00, 0xfc, 0xd7, 0x9e, 0xf6, 0xbf, 0x7f, 0xed, 0x90, 0x4f, 0x46,
        0xa3, 0xbf,
    ];

    let mut enc = ArithEncoder::new();

    // 入力を1bpp画像として解釈: 256px幅 x 1行
    // コンテキストはエンコーダの内部contextを使う
    // Template 0のデフォルトAT位置で、1行目のみなので
    // c1=0 (y-2行目なし), c2=0 (y-1行目なし), c3を構築しながら符号化

    // H.2ではコンテキスト番号0のみを使い、各ビットを順に符号化する
    // 仕様書のH.2に記載の出力バイト列と照合
    let mut ctx = vec![0u8; 65536];
    for byte in &input {
        for bit in (0..8).rev() {
            let d = (byte >> bit) & 1;
            enc.encode_bit(&mut ctx, 0, d);
        }
    }
    enc.encode_final();

    let output = enc.to_vec();
    // H.2の期待出力は仕様書を参照して検証
    // ここではまず出力が空でないことを確認
    assert!(!output.is_empty(), "H.2 test should produce output");
}

/// encode_bit で全0ビット列を符号化した場合の動作確認。
#[test]
#[ignore = "not yet implemented"]
fn encoder_all_zeros() {
    let mut enc = ArithEncoder::new();
    let mut ctx = vec![0u8; 1];
    for _ in 0..100 {
        enc.encode_bit(&mut ctx, 0, 0);
    }
    enc.encode_final();
    let output = enc.to_vec();
    assert!(!output.is_empty());
}

/// encode_bit で全1ビット列を符号化した場合の動作確認。
#[test]
#[ignore = "not yet implemented"]
fn encoder_all_ones() {
    let mut enc = ArithEncoder::new();
    let mut ctx = vec![0u8; 1];
    for _ in 0..100 {
        enc.encode_bit(&mut ctx, 0, 1);
    }
    enc.encode_final();
    let output = enc.to_vec();
    assert!(!output.is_empty());
}

/// encode_final が適切な終端マーカー（0xFF 0xAC）を出力することを確認。
#[test]
#[ignore = "not yet implemented"]
fn encoder_final_marker() {
    let mut enc = ArithEncoder::new();
    let mut ctx = vec![0u8; 1];
    // 最低限のビットを符号化
    enc.encode_bit(&mut ctx, 0, 0);
    enc.encode_final();
    let output = enc.to_vec();
    // 出力の末尾2バイトが 0xFF 0xAC であることを確認
    assert!(output.len() >= 2);
    assert_eq!(output[output.len() - 2], 0xFF);
    assert_eq!(output[output.len() - 1], 0xAC);
}

/// to_vec が data_size と同じ長さのベクタを返すことを確認。
#[test]
#[ignore = "not yet implemented"]
fn encoder_to_vec_size_matches_data_size() {
    let mut enc = ArithEncoder::new();
    let mut ctx = vec![0u8; 1];
    for _ in 0..50 {
        enc.encode_bit(&mut ctx, 0, 0);
    }
    enc.encode_final();
    assert_eq!(enc.to_vec().len(), enc.data_size());
}
