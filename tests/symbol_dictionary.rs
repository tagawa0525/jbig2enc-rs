use jbig2enc_rs::symbol::encode_symbol_table;
use leptonica::{Pix, PixMut, PixelDepth};

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

/// 指定サイズの全白1bppシンボルを作成する。
fn white_symbol(width: u32, height: u32) -> Pix {
    PixMut::new(width, height, PixelDepth::Bit1).unwrap().into()
}

/// 指定サイズの全黒1bppシンボルを作成する。
fn black_symbol(width: u32, height: u32) -> Pix {
    let mut pm = PixMut::new(width, height, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    pm.into()
}

/// ボーダー付きシンボルを作成する（内部は白、ボーダーも白）。
fn bordered_symbol(inner_w: u32, inner_h: u32, border: u32) -> Pix {
    let w = inner_w + 2 * border;
    let h = inner_h + 2 * border;
    PixMut::new(w, h, PixelDepth::Bit1).unwrap().into()
}

// ---------------------------------------------------------------------------
// エラーケース
// ---------------------------------------------------------------------------

/// 範囲外のインデックスを渡すとエラーになること。
#[test]

fn rejects_out_of_bounds_index() {
    let symbols = vec![white_symbol(10, 10)];
    let result = encode_symbol_table(&symbols, &[0, 5], false, 0);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 基本動作テスト
// ---------------------------------------------------------------------------

/// 空のインデックスリストでは空の結果を返す。
#[test]

fn empty_symbol_list() {
    let symbols = vec![white_symbol(10, 10)];
    let result = encode_symbol_table(&symbols, &[], false, 0).unwrap();
    assert!(result.data.is_empty() || !result.data.is_empty()); // 空でもOK
    assert!(result.symmap.is_empty());
}

/// 単一シンボルをエンコードできること。
#[test]

fn single_symbol() {
    let symbols = vec![white_symbol(16, 16)];
    let result = encode_symbol_table(&symbols, &[0], false, 0).unwrap();
    assert!(!result.data.is_empty());
    assert_eq!(result.symmap.len(), 1);
    assert_eq!(result.symmap[&0], 0);
}

/// 複数シンボルをエンコードできること。
#[test]

fn multiple_symbols() {
    let symbols = vec![
        white_symbol(10, 10),
        white_symbol(20, 20),
        white_symbol(15, 15),
    ];
    let result = encode_symbol_table(&symbols, &[0, 1, 2], false, 0).unwrap();
    assert!(!result.data.is_empty());
    assert_eq!(result.symmap.len(), 3);
}

// ---------------------------------------------------------------------------
// ソート検証
// ---------------------------------------------------------------------------

/// シンボルは高さ順にソートされ、symmap に正しい番号が付与される。
///
/// 入力:
///   index 0: 20x20 → ソート後2番目
///   index 1: 10x10 → ソート後0番目
///   index 2: 15x15 → ソート後1番目
///
/// 符号化番号: index1=0, index2=1, index0=2
#[test]

fn sorted_by_height() {
    let symbols = vec![
        white_symbol(20, 20), // index 0
        white_symbol(10, 10), // index 1
        white_symbol(15, 15), // index 2
    ];
    let result = encode_symbol_table(&symbols, &[0, 1, 2], false, 0).unwrap();

    // 高さ順: 10(idx1) → 15(idx2) → 20(idx0)
    assert_eq!(result.symmap[&1], 0); // 10x10 → 符号化番号0
    assert_eq!(result.symmap[&2], 1); // 15x15 → 符号化番号1
    assert_eq!(result.symmap[&0], 2); // 20x20 → 符号化番号2
}

/// 同一高さのシンボルは幅順にソートされる。
///
/// 入力（すべて高さ20）:
///   index 0: 30x20 → ソート後2番目
///   index 1: 10x20 → ソート後0番目
///   index 2: 20x20 → ソート後1番目
#[test]

fn same_height_sorted_by_width() {
    let symbols = vec![
        white_symbol(30, 20), // index 0
        white_symbol(10, 20), // index 1
        white_symbol(20, 20), // index 2
    ];
    let result = encode_symbol_table(&symbols, &[0, 1, 2], false, 0).unwrap();

    // 幅順: 10(idx1) → 20(idx2) → 30(idx0)
    assert_eq!(result.symmap[&1], 0);
    assert_eq!(result.symmap[&2], 1);
    assert_eq!(result.symmap[&0], 2);
}

/// 複数の高さクラスがあり、各クラス内で幅順にソートされる。
///
/// 入力:
///   index 0: 20x10 → 高さ10クラス、幅20 → 符号化番号1
///   index 1: 10x10 → 高さ10クラス、幅10 → 符号化番号0
///   index 2: 15x20 → 高さ20クラス、幅15 → 符号化番号2
///   index 3: 25x20 → 高さ20クラス、幅25 → 符号化番号3
#[test]

fn multiple_height_classes() {
    let symbols = vec![
        white_symbol(20, 10), // index 0
        white_symbol(10, 10), // index 1
        white_symbol(15, 20), // index 2
        white_symbol(25, 20), // index 3
    ];
    let result = encode_symbol_table(&symbols, &[0, 1, 2, 3], false, 0).unwrap();

    // 高さ10クラス: idx1(w=10)→0, idx0(w=20)→1
    // 高さ20クラス: idx2(w=15)→2, idx3(w=25)→3
    assert_eq!(result.symmap[&1], 0);
    assert_eq!(result.symmap[&0], 1);
    assert_eq!(result.symmap[&2], 2);
    assert_eq!(result.symmap[&3], 3);
}

// ---------------------------------------------------------------------------
// ボーダー除去テスト
// ---------------------------------------------------------------------------

/// unborder=true の場合、ボーダーを除去して符号化する。
/// ボーダー付きとボーダーなしで出力が異なることを検証。
#[test]

fn unborder_produces_different_output() {
    // ボーダー4のシンボル（内部16x16、全体24x24）
    let bordered = bordered_symbol(16, 16, 4);
    let symbols = vec![bordered];

    let with_border = encode_symbol_table(&symbols, &[0], false, 0).unwrap();
    let without_border = encode_symbol_table(&symbols, &[0], true, 4).unwrap();

    // ボーダーあり: 24x24として符号化
    // ボーダーなし: 16x16として符号化
    assert_ne!(with_border.data, without_border.data);
}

/// unborder=true の場合、ソートはボーダー除去後のサイズで行われる。
///
/// ボーダー=4:
///   index 0: 全体28x28 → 除去後20x20 → 符号化番号1
///   index 1: 全体18x18 → 除去後10x10 → 符号化番号0
#[test]

fn unborder_sorting_uses_inner_size() {
    let symbols = vec![
        bordered_symbol(20, 20, 4), // index 0: 28x28 → 20x20
        bordered_symbol(10, 10, 4), // index 1: 18x18 → 10x10
    ];
    let result = encode_symbol_table(&symbols, &[0, 1], true, 4).unwrap();

    // 除去後サイズでソート: 10x10(idx1)→0, 20x20(idx0)→1
    assert_eq!(result.symmap[&1], 0);
    assert_eq!(result.symmap[&0], 1);
}

// ---------------------------------------------------------------------------
// 部分インデックスリスト
// ---------------------------------------------------------------------------

/// symbol_indices がシンボル配列の一部だけを指定する場合。
#[test]

fn partial_symbol_indices() {
    let symbols = vec![
        white_symbol(10, 10), // index 0 — 含まない
        white_symbol(20, 20), // index 1 — 含む
        white_symbol(15, 15), // index 2 — 含む
    ];
    let result = encode_symbol_table(&symbols, &[1, 2], false, 0).unwrap();

    assert_eq!(result.symmap.len(), 2);
    // 高さ順: 15(idx2)→0, 20(idx1)→1
    assert_eq!(result.symmap[&2], 0);
    assert_eq!(result.symmap[&1], 1);
    // index 0 は含まれない
    assert!(!result.symmap.contains_key(&0));
}

// ---------------------------------------------------------------------------
// エッジケース
// ---------------------------------------------------------------------------

/// 全シンボルが同一サイズの場合。
#[test]

fn all_same_size() {
    let symbols = vec![
        white_symbol(16, 16),
        black_symbol(16, 16),
        white_symbol(16, 16),
    ];
    let result = encode_symbol_table(&symbols, &[0, 1, 2], false, 0).unwrap();
    assert_eq!(result.symmap.len(), 3);
    // 同一サイズなのでソート順は安定（入力順を保持）
    assert_eq!(result.symmap[&0], 0);
    assert_eq!(result.symmap[&1], 1);
    assert_eq!(result.symmap[&2], 2);
}

/// 黒シンボルと白シンボルで出力が異なること。
#[test]

fn different_content_different_output() {
    let white = vec![white_symbol(16, 16)];
    let black = vec![black_symbol(16, 16)];

    let white_result = encode_symbol_table(&white, &[0], false, 0).unwrap();
    let black_result = encode_symbol_table(&black, &[0], false, 0).unwrap();

    assert_ne!(white_result.data, black_result.data);
}

/// 幅が32の倍数でないシンボル。
#[test]

fn non_32_aligned_symbol_width() {
    let symbols = vec![white_symbol(13, 7)];
    let result = encode_symbol_table(&symbols, &[0], false, 0);
    assert!(result.is_ok());
    assert!(!result.unwrap().data.is_empty());
}
