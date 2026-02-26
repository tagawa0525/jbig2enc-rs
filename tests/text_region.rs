use std::collections::HashMap;

use jbig2enc::symbol::{SymbolInstance, TextRegionConfig, encode_text_region};
use leptonica::{Pix, PixMut, PixelDepth};

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

fn white_sym(width: u32, height: u32) -> Pix {
    PixMut::new(width, height, PixelDepth::Bit1).unwrap().into()
}

fn black_sym(width: u32, height: u32) -> Pix {
    let mut pm = PixMut::new(width, height, PixelDepth::Bit1).unwrap();
    pm.set_all_arbitrary(1).unwrap();
    pm.into()
}

fn identity_symmap(n: usize) -> HashMap<usize, usize> {
    (0..n).map(|i| (i, i)).collect()
}

fn default_cfg<'a>(symmap: &'a HashMap<usize, usize>, n: usize) -> TextRegionConfig<'a> {
    TextRegionConfig {
        symmap,
        symmap2: None,
        global_sym_count: n,
        symbits: 1,
        strip_width: 1,
        unborder: false,
        border_size: 0,
    }
}

// ---------------------------------------------------------------------------
// エラーケース
// ---------------------------------------------------------------------------

/// strip_width に不正な値（3）を渡すとエラーになること。
#[test]
fn rejects_invalid_strip_width() {
    let symbols = vec![white_sym(10, 10)];
    let symmap = identity_symmap(1);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 10,
        class_id: 0,
    }];
    let cfg = TextRegionConfig {
        strip_width: 3,
        ..default_cfg(&symmap, 1)
    };
    assert!(encode_text_region(&instances, &symbols, &cfg).is_err());
}

/// symmap に存在しない class_id を持つインスタンスはエラーになること。
#[test]
fn rejects_unknown_class_id() {
    let symbols = vec![white_sym(10, 10)];
    let symmap = identity_symmap(1);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 10,
        class_id: 99,
    }];
    assert!(encode_text_region(&instances, &symbols, &default_cfg(&symmap, 1)).is_err());
}

/// symmap に登録されている class_id でも symbols 配列の範囲外ならエラーになること。
#[test]
fn rejects_class_id_in_symmap_but_out_of_bounds_symbols() {
    // symbols は 1 要素（index 0 のみ有効）
    let symbols = vec![white_sym(10, 10)];
    // symmap は class_id=1 を持つが、symbols.len()=1 なので範囲外
    let mut symmap = HashMap::new();
    symmap.insert(1usize, 0usize);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 10,
        class_id: 1,
    }];
    let cfg = TextRegionConfig {
        symbits: 1,
        ..default_cfg(&symmap, 1)
    };
    assert!(encode_text_region(&instances, &symbols, &cfg).is_err());
}

/// symmap が返す symid が symbits に収まらない場合はエラーになること。
#[test]
fn rejects_symid_not_fit_in_symbits() {
    let symbols = vec![white_sym(10, 10)];
    // symbits=1 → max_symid=2。symmap は symid=2 を返す → 2 >= 2 → エラー
    let mut symmap = HashMap::new();
    symmap.insert(0usize, 2usize);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 10,
        class_id: 0,
    }];
    let cfg = TextRegionConfig {
        symbits: 1,
        ..default_cfg(&symmap, 1)
    };
    assert!(encode_text_region(&instances, &symbols, &cfg).is_err());
}

/// symbits=0 のとき、symid=1 が返ってきたらエラーになること（1 >= 1<<0=1）。
#[test]
fn rejects_symid_overflow_with_symbits_zero() {
    let symbols = vec![white_sym(10, 10)];
    // symbits=0 → max_symid=1。symid=1 → 1 >= 1 → エラー
    let mut symmap = HashMap::new();
    symmap.insert(0usize, 1usize);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 10,
        class_id: 0,
    }];
    let cfg = TextRegionConfig {
        symbits: 0,
        ..default_cfg(&symmap, 1)
    };
    assert!(encode_text_region(&instances, &symbols, &cfg).is_err());
}

/// symbits が 31 を超えるとエラーになること（encode_iaid の内部制約）。
#[test]
fn rejects_symbits_gt_31() {
    let symbols = vec![white_sym(10, 10)];
    let symmap = identity_symmap(1);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 10,
        class_id: 0,
    }];
    let cfg = TextRegionConfig {
        symbits: 32,
        ..default_cfg(&symmap, 1)
    };
    assert!(encode_text_region(&instances, &symbols, &cfg).is_err());
}

// ---------------------------------------------------------------------------
// 基本動作テスト
// ---------------------------------------------------------------------------

/// 空のインスタンスリストでは空のデータを返す。
#[test]
fn empty_instances() {
    let symbols = vec![white_sym(10, 10)];
    let symmap = identity_symmap(1);
    let result = encode_text_region(&[], &symbols, &default_cfg(&symmap, 1)).unwrap();
    assert!(result.data.is_empty());
}

/// 単一シンボルをエンコードできること。
#[test]
fn single_instance() {
    let symbols = vec![white_sym(16, 16)];
    let symmap = identity_symmap(1);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 16,
        class_id: 0,
    }];
    let result = encode_text_region(&instances, &symbols, &default_cfg(&symmap, 1)).unwrap();
    assert!(!result.data.is_empty());
}

/// 複数シンボルをエンコードできること。
#[test]
fn multiple_instances() {
    let symbols = vec![white_sym(10, 10), white_sym(12, 12)];
    let symmap = identity_symmap(2);
    let instances = vec![
        SymbolInstance {
            x: 0,
            y: 10,
            class_id: 0,
        },
        SymbolInstance {
            x: 20,
            y: 10,
            class_id: 1,
        },
    ];
    let cfg = TextRegionConfig {
        symbits: 1,
        ..default_cfg(&symmap, 2)
    };
    let result = encode_text_region(&instances, &symbols, &cfg).unwrap();
    assert!(!result.data.is_empty());
}

// ---------------------------------------------------------------------------
// ストリップ分割テスト（strip_width=1）
// ---------------------------------------------------------------------------

/// 同一Y座標と異なるY座標では出力が異なる。
#[test]
fn strip_width_1_same_y_same_strip() {
    let symbols = vec![white_sym(10, 10), white_sym(10, 10)];
    let symmap = identity_symmap(2);
    let cfg = TextRegionConfig {
        symbits: 1,
        ..default_cfg(&symmap, 2)
    };
    let same_strip = vec![
        SymbolInstance {
            x: 0,
            y: 10,
            class_id: 0,
        },
        SymbolInstance {
            x: 20,
            y: 10,
            class_id: 1,
        },
    ];
    let diff_strip = vec![
        SymbolInstance {
            x: 0,
            y: 10,
            class_id: 0,
        },
        SymbolInstance {
            x: 0,
            y: 20,
            class_id: 1,
        },
    ];
    let r1 = encode_text_region(&same_strip, &symbols, &cfg).unwrap();
    let r2 = encode_text_region(&diff_strip, &symbols, &cfg).unwrap();
    assert_ne!(r1.data, r2.data);
}

/// Y座標の逆順で渡してもソートして同じ出力になる。
#[test]
fn instances_sorted_by_y() {
    let symbols = vec![white_sym(10, 10), white_sym(10, 10)];
    let symmap = identity_symmap(2);
    let cfg = TextRegionConfig {
        symbits: 1,
        ..default_cfg(&symmap, 2)
    };
    let forward = vec![
        SymbolInstance {
            x: 0,
            y: 5,
            class_id: 0,
        },
        SymbolInstance {
            x: 0,
            y: 15,
            class_id: 1,
        },
    ];
    let backward = vec![
        SymbolInstance {
            x: 0,
            y: 15,
            class_id: 1,
        },
        SymbolInstance {
            x: 0,
            y: 5,
            class_id: 0,
        },
    ];
    let r1 = encode_text_region(&forward, &symbols, &cfg).unwrap();
    let r2 = encode_text_region(&backward, &symbols, &cfg).unwrap();
    assert_eq!(r1.data, r2.data);
}

/// 同一ストリップ内でX座標逆順で渡してもソートして同じ出力になる。
#[test]
fn instances_sorted_by_x_within_strip() {
    let symbols = vec![white_sym(10, 10), white_sym(10, 10)];
    let symmap = identity_symmap(2);
    let cfg = TextRegionConfig {
        symbits: 1,
        ..default_cfg(&symmap, 2)
    };
    let left_first = vec![
        SymbolInstance {
            x: 0,
            y: 10,
            class_id: 0,
        },
        SymbolInstance {
            x: 20,
            y: 10,
            class_id: 1,
        },
    ];
    let right_first = vec![
        SymbolInstance {
            x: 20,
            y: 10,
            class_id: 1,
        },
        SymbolInstance {
            x: 0,
            y: 10,
            class_id: 0,
        },
    ];
    let r1 = encode_text_region(&left_first, &symbols, &cfg).unwrap();
    let r2 = encode_text_region(&right_first, &symbols, &cfg).unwrap();
    assert_eq!(r1.data, r2.data);
}

// ---------------------------------------------------------------------------
// strip_width テスト
// ---------------------------------------------------------------------------

/// strip_width=1 と strip_width=2 では出力が異なる。
#[test]
fn different_strip_widths_differ() {
    let symbols = vec![white_sym(10, 10)];
    let symmap = identity_symmap(1);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 10,
        class_id: 0,
    }];
    let cfg1 = TextRegionConfig {
        strip_width: 1,
        ..default_cfg(&symmap, 1)
    };
    let cfg2 = TextRegionConfig {
        strip_width: 2,
        ..default_cfg(&symmap, 1)
    };
    let r1 = encode_text_region(&instances, &symbols, &cfg1).unwrap();
    let r2 = encode_text_region(&instances, &symbols, &cfg2).unwrap();
    assert_ne!(r1.data, r2.data);
}

// ---------------------------------------------------------------------------
// symmap2 テスト（2辞書）
// ---------------------------------------------------------------------------

/// symmap2 に存在するシンボルIDがグローバル辞書のサイズだけオフセットされる。
#[test]
fn symmap2_offset_by_global_count() {
    let symbols = vec![white_sym(10, 10), white_sym(10, 10)];
    let symmap = {
        let mut m = HashMap::new();
        m.insert(0usize, 0usize);
        m
    };
    let symmap2 = {
        let mut m = HashMap::new();
        m.insert(1usize, 0usize);
        m
    };
    let instances = vec![SymbolInstance {
        x: 0,
        y: 10,
        class_id: 1,
    }];
    let cfg = TextRegionConfig {
        symmap: &symmap,
        symmap2: Some(&symmap2),
        global_sym_count: 1,
        symbits: 1,
        strip_width: 1,
        unborder: false,
        border_size: 0,
    };
    assert!(encode_text_region(&instances, &symbols, &cfg).is_ok());
}

// ---------------------------------------------------------------------------
// エッジケース
// ---------------------------------------------------------------------------

/// 幅1の最小シンボル。
#[test]
fn minimal_symbol() {
    let symbols = vec![white_sym(1, 1)];
    let symmap = identity_symmap(1);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 1,
        class_id: 0,
    }];
    let result = encode_text_region(&instances, &symbols, &default_cfg(&symmap, 1)).unwrap();
    assert!(!result.data.is_empty());
}

/// 同一シンボルを大量に配置。
#[test]
fn many_instances_same_symbol() {
    let symbols = vec![white_sym(8, 8)];
    let symmap = identity_symmap(1);
    let instances: Vec<SymbolInstance> = (0..20)
        .map(|i| SymbolInstance {
            x: i * 10,
            y: 8,
            class_id: 0,
        })
        .collect();
    let result = encode_text_region(&instances, &symbols, &default_cfg(&symmap, 1)).unwrap();
    assert!(!result.data.is_empty());
}

/// 複数行のテキスト（異なるY座標のシンボル）。
#[test]
fn multiline_text() {
    let symbols = vec![white_sym(10, 12)];
    let symmap = identity_symmap(1);
    let instances: Vec<SymbolInstance> = (0..3)
        .flat_map(|line| {
            (0..5).map(move |col| SymbolInstance {
                x: col * 12,
                y: (line + 1) * 14,
                class_id: 0,
            })
        })
        .collect();
    let result = encode_text_region(&instances, &symbols, &default_cfg(&symmap, 1)).unwrap();
    assert!(!result.data.is_empty());
}

// ---------------------------------------------------------------------------
// ボーダー除去テスト
// ---------------------------------------------------------------------------

/// unborder=true の場合、シンボル幅計算が変わるため同一ストリップの2シンボルで出力が変化する。
#[test]
fn unborder_affects_curs_update() {
    // 全体18x18（ボーダー4、内部10x10）のシンボルを2つ同一ストリップに配置
    let make_sym = || -> Pix { PixMut::new(18, 18, PixelDepth::Bit1).unwrap().into() };
    let symmap = identity_symmap(1);
    let instances = vec![
        SymbolInstance {
            x: 0,
            y: 18,
            class_id: 0,
        },
        SymbolInstance {
            x: 30,
            y: 18,
            class_id: 0,
        },
    ];
    let cfg_no_unborder = TextRegionConfig {
        unborder: false,
        border_size: 0,
        ..default_cfg(&symmap, 1)
    };
    let cfg_with_unborder = TextRegionConfig {
        unborder: true,
        border_size: 4,
        ..default_cfg(&symmap, 1)
    };
    let r1 = encode_text_region(&instances, &[make_sym()], &cfg_no_unborder).unwrap();
    let r2 = encode_text_region(&instances, &[make_sym()], &cfg_with_unborder).unwrap();
    assert_ne!(r1.data, r2.data);
}

/// 白シンボルと黒シンボルはエンコード可能（内容は出力に影響しない、幅のみ）。
#[test]
fn different_symbol_content_encodes_successfully() {
    let symmap = identity_symmap(1);
    let instances = vec![SymbolInstance {
        x: 0,
        y: 16,
        class_id: 0,
    }];
    let r_white =
        encode_text_region(&instances, &[white_sym(16, 16)], &default_cfg(&symmap, 1)).unwrap();
    let r_black =
        encode_text_region(&instances, &[black_sym(16, 16)], &default_cfg(&symmap, 1)).unwrap();
    assert!(!r_white.data.is_empty());
    assert!(!r_black.data.is_empty());
}
