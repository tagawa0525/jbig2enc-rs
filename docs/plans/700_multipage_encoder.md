# Phase 7: マルチページエンコーダ

**Status**: PLANNED

## 概要

C++版 `jbig2ctx` 構造体と関連関数（`jbig2enc.cc:98-893`）のRust移植。
複数の1bppページをシンボル分類し、グローバル辞書 + ページ別テキストリージョンとしてJBIG2符号化する。

## C++版対応

| C++関数/構造体 | Rust対応 |
|---|---|
| `struct jbig2ctx` | `Jbig2Context` |
| `jbig2_init()` | `Jbig2Context::new()` |
| `jbig2_add_page()` | `Jbig2Context::add_page()` |
| `jbig2_pages_complete()` | `Jbig2Context::pages_complete()` |
| `jbig2_produce_page()` | `Jbig2Context::produce_page()` |
| `log2up()` | `log2up()` |
| `jbig2enc_auto_threshold()` | `Jbig2Context::auto_threshold()`（別PR） |
| `jbig2enc_auto_threshold_using_hash()` | `Jbig2Context::auto_threshold_using_hash()`（別PR） |

## API設計

```rust
use leptonica::Pix;
use crate::error::Jbig2Error;

pub struct Jbig2Context { /* ... */ }

impl Jbig2Context {
    /// マルチページ圧縮コンテキストを生成する。
    pub fn new(
        thresh: f32,
        weight: f32,
        xres: u32,
        yres: u32,
        full_headers: bool,
        refine_level: i32,
    ) -> Result<Self, Jbig2Error>;

    /// ページを追加し、シンボル抽出・分類を実行する。
    pub fn add_page(&mut self, pix: &Pix) -> Result<(), Jbig2Error>;

    /// シンボルテーブルを符号化し、ファイルヘッダ + グローバル辞書セグメントを返す。
    pub fn pages_complete(&mut self) -> Result<Vec<u8>, Jbig2Error>;

    /// 指定ページのテキストリージョンを符号化する。
    pub fn produce_page(
        &mut self,
        page_no: usize,
        xres: Option<u32>,
        yres: Option<u32>,
    ) -> Result<Vec<u8>, Jbig2Error>;
}
```

## 内部構造

```rust
struct Jbig2Context {
    classer: JbClasser,             // leptonica シンボル分類器
    xres: u32,
    yres: u32,
    full_headers: bool,
    pdf_page_numbering: bool,       // !full_headers → 全ページ page=1
    segnum: u32,                    // 現在のセグメント番号
    symtab_segment: Option<u32>,    // グローバルシンボルテーブルのセグメント番号
    pagecomps: HashMap<usize, Vec<usize>>,      // ページ → コンポーネントインデックス
    single_use_symbols: HashMap<usize, Vec<usize>>,  // ページ固有シンボル
    num_global_symbols: usize,
    page_xres: Vec<u32>,
    page_yres: Vec<u32>,
    page_width: Vec<u32>,
    page_height: Vec<u32>,
    symmap: HashMap<usize, usize>,  // シンボル番号 → グローバル辞書インデックス
    refinement: bool,
    refine_level: i32,
    baseindexes: Vec<usize>,
}
```

## アルゴリズム

### new()

1. `JbClasser::correlation_init(ConnComps, 9999, 9999, thresh, weight)` でシンボル分類器を初期化
2. `pdf_page_numbering = !full_headers`（C++と同一）

### add_page()

1. `classer.add_page(pix)` でシンボル抽出・分類を実行
2. ページ寸法・解像度を記録
3. リファインメント使用時は `baseindex` を記録（現在は未使用）

### pages_complete()

1. **使用回数カウント**: 各シンボルの使用回数を `naclass` からカウント
2. **グローバル/ローカル分離**:
   - 2回以上使用されたシンボル → グローバル辞書（`multiuse_symbols`）
   - 1回のみ使用（マルチページ時のみ） → ページ固有辞書（`single_use_symbols`）
   - 単一ページの場合は全シンボルをグローバルに
3. **ページコンポーネントマップ構築**: `napage` からページ → コンポーネント逆マッピング
4. **グローバル辞書符号化**: `encode_symbol_table()` でシンボルテーブルを符号化
5. **セグメント組み立て**: FileHeader（full_headers時）+ SymbolDict セグメント

### produce_page()

1. **ページ情報セグメント**: PageInfo（寸法、解像度、ロスレスフラグ）
2. **ページ固有辞書（該当時）**: `single_use_symbols` がある場合、追加のSymbolDictセグメント
3. **テキストリージョン**:
   - `pagecomps[page_no]` のコンポーネントを `SymbolInstance` に変換
     - `ptall[comp_idx]` → `(x, y)`
     - `naclass[comp_idx]` → `class_id`
   - `symbits = log2up(num_global + num_page_local)`
   - `encode_text_region()` で符号化
4. **セグメント組み立て**: TextRegion ヘッダ + データ
5. **トレーラー**: EndOfPage（full_headers時）+ EndOfFile（最終ページ + full_headers時）

### log2up()

```rust
fn log2up(v: usize) -> u32 {
    if v <= 1 { return 0; }
    usize::BITS - (v - 1).leading_zeros()
}
```

C++版 `log2up()` と同等。シンボル数からIAIDビット数を算出。

## leptonica-rs API

| 関数/フィールド | シグネチャ | 用途 |
|---|---|---|
| `correlation_init` | `fn(JbComponent, i32, i32, f32, f32) -> Result<JbClasser>` | 分類器初期化 |
| `JbClasser::add_page` | `fn(&mut self, &Pix) -> Result<()>` | ページ追加 |
| `JbClasser::pixat` | `Vec<Pix>` | シンボルテンプレート |
| `JbClasser::naclass` | `Vec<usize>` | コンポーネント → クラスID |
| `JbClasser::napage` | `Vec<usize>` | コンポーネント → ページ番号 |
| `JbClasser::ptall` | `Vec<(i32, i32)>` | コンポーネント下端座標 |
| `JbClasser::npages` | `usize` | ページ数 |
| `JbClasser::base_index` | `usize` | ページ別ベースインデックス |
| `TEMPLATE_BORDER` | `i32 = 4` | テンプレートボーダーサイズ |

## 設計決定

1. **`Vec<u8>` 返却**: C++の `malloc + caller free` ではなくRustの所有権ベース
2. **`Option<u32>` でxres/yres**: C++の `-1` 規約ではなくRust慣用
3. **リファインメント未実装**: C++版でも「broken」とコメント。フラグは保持するが実際の符号化はスキップ
4. **TEMPLATE_BORDER = 4**: C++版の `kBorderSize = 6` とは異なる。leptonica-rs に合わせる
5. **auto_threshold は別PR**: コア機能と分離して段階的に実装

## テスト方針

1. **log2up**: 既知値のテーブルテスト
2. **単一ページE2E**: 1ページのシンボル画像 → 符号化 → 出力非空
3. **マルチページE2E**: 複数ページ → 符号化 → 出力非空
4. **full_headers モード**: JBIG2マジックバイトの存在確認
5. **PDF モード**: マジックバイト非存在確認
6. **ページ数**: FileHeader内のページ数フィールド検証
7. **解像度オーバーライド**: produce_page の xres/yres 上書き
8. **ページ固有シンボル**: マルチページで単一使用シンボルの辞書分離

## PR構成

1. `docs:` 計画書コミット
2. `test:` REDテスト（`#[ignore]` 付き）
3. `feat(encoder):` GREEN実装
