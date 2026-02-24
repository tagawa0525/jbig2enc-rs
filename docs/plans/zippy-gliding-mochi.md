# jbig2enc-rs 移植計画

## Context

C++版 [jbig2enc](https://github.com/agl/jbig2enc)（約3,950行）をRustに移植する。JBIG2はbi-level（1bpp）画像をG4より高い圧縮率で符号化するフォーマット。画像処理基盤として crates.io の `leptonica = "0.1.0"` を使用する。

現状のRustプロジェクトは `src/main.rs`（Hello, world!）のみ。leptonica-rsの API互換性調査（`reference/leptonica-rs/docs/porting/jbig2enc-api-compatibility.md`）により、以前HIGHリスクだった2つのギャップ（morph_sequence の `r`/`x` 演算子、`rop_region_inplace`）は解消済み。

---

## モジュール構成

```
src/
  lib.rs                 -- ライブラリルート（公開API再エクスポート）
  error.rs               -- Jbig2Error enum
  arith/
    mod.rs               -- 算術符号化モジュール
    state_table.rs       -- QMコーダ 92状態テーブル
    encoder.rs           -- ArithEncoder（c/a/ct/b/bp、出力バッファ、コンテキスト）
    int_encode.rs        -- 整数符号化（IADH, IADW, IAFS, IADS 等13種）
    iaid.rs              -- シンボルID符号化
    image.rs             -- ビットマップ符号化（bitimage, refine）
  wire/
    mod.rs               -- JBIG2ワイヤフォーマット
    structs.rs           -- バイナリ構造体（FileHeader, PageInfo, GenericRegion等）
    segment.rs           -- セグメントヘッダ（可変長シリアライズ）
    constants.rs         -- マジックバイト、セグメントタイプ定数
  symbol/
    mod.rs               -- シンボル符号化
    dictionary.rs        -- シンボル辞書（高さ/幅ソート、デルタ符号化）
    text_region.rs       -- テキストリージョン（ストリップ、配置、リファインメント）
  comparator/
    mod.rs               -- シンボル視覚的等価性判定（9×9グリッド分析）
  encoder/
    mod.rs               -- 高レベルエンコーダ
    context.rs           -- Jbig2Context（マルチページコンテキスト）
    generic.rs           -- ジェネリックリージョン符号化（単一ページロスレス）
    multipage.rs         -- マルチページシンボルモード符号化
    auto_threshold.rs    -- シンボルクラス統合（ブルートフォース + ハッシュ）
  main.rs                -- CLIツール（最低優先度）
```

---

## 実装フェーズ

### 依存関係

```
Phase 1 (arith)     Phase 2 (wire)      Phase 6 (comparator)
     |                   |                    |
     +-------+-----------+                    |
             |                                |
Phase 3 (generic)  Phase 4 (symbol dict)      |
                        |                     |
                   Phase 5 (text region)      |
                        |                     |
                        +----------+----------+
                                   |
                              Phase 7 (multipage)
                                   |
                              Phase 8 (CLI)
```

Phase 1, 2, 6 は並行開発可能。

---

### Phase 1: 算術符号化 (`arith/`)

**計画書**: `docs/plans/100_arithmetic_coder.md`
**C++対応**: `jbig2arith.h` + `jbig2arith.cc` (970行)
**外部依存**: なし（leptonica不要）

**スコープ**:
- QMコーダ状態テーブル（46状態 × 2 = 92エントリ）
- `ArithEncoder` — コーダ状態管理、20KBチャンク出力
- `encode_bit` / `byteout` / `emit` / `encode_final` — ビット符号化のステートマシン
- 整数符号化（13種のプロシージャ、13レンジテーブル）
- `encode_oob` — OOBセンチネル
- `encode_iaid` — 固定ビット長シンボルID符号化
- `encode_bitimage` — パックド1bppビットマップ符号化（TPGD対応）
- `encode_refine` — リファインメント符号化（13ピクセルテンプレート）

**テスト**:
- 状態テーブルの全92エントリをC++の`ctbl[]`と照合
- JBIG2仕様 Appendix H.2のテストベクタ（C++に含まれるテスト入力配列）でバイト比較
- 既知整数値の符号化結果をC++出力と比較
- 小さな既知ビットマップ（8×8パターン）のTPGD on/off両方で比較

**PR**: 3-4本
1. 状態テーブル + ArithEncoder core（new/reset/flush/encode_bit/byteout/finalize/to_vec）
2. 整数符号化（encode_int/encode_oob/encode_iaid）
3. ビットマップ符号化（encode_bitimage + encode_refine）

---

### Phase 2: ワイヤフォーマット (`wire/`)

**計画書**: `docs/plans/200_wire_format.md`
**C++対応**: `jbig2structs.h` + `jbig2segments.h` (362行)
**外部依存**: なし

**スコープ**:
- `FileHeader` — マジック(8B) + flags(1B) + pages(4B)
- `PageInfo` — width/height/xres/yres(各4B) + flags
- `GenericRegion`, `SymbolDict`, `TextRegion` ヘッダ
- `SegmentHeader` — 可変長セグメントヘッダ（referred_to リスト、ページ番号）
- すべてビッグエンディアン、手動ビットパッキング（`#[repr(C, packed)]`は使わない）

**テスト**:
- FileHeaderシリアライズが期待バイト列と一致
- 各構造体のビットフィールドレイアウトをC++出力バイトと照合
- SegmentHeaderの可変長フィールド（referred_to数, ページ番号）のサイズ計算検証

**PR**: 1-2本

---

### Phase 3: ジェネリックリージョン符号化 (`encoder/generic.rs`)

**計画書**: `docs/plans/300_generic_region.md`
**C++対応**: `jbig2enc.cc` の `jbig2_encode_generic()` (約105行)
**依存**: Phase 1 + Phase 2 + leptonica

**スコープ**:
```rust
pub fn encode_generic(
    pix: &Pix,
    full_headers: bool,
    xres: u32, yres: u32,
    duplicate_line_removal: bool,
) -> Result<Vec<u8>, Jbig2Error>;
```
- ファイルヘッダ（full_headers時）、ページ情報セグメント、ジェネリックリージョンセグメント組み立て
- `Pix::data() -> &[u32]` から `ArithEncoder::encode_bitimage()` へのデータ受け渡し

**leptonica-rs API**: `Pix::data()`, `Pix::width()`, `Pix::height()`, `Pix::wpl()`, `PixMut::set_pad_bits()`

**テスト**:
- 既知の1bppテスト画像をC++版とRust版で符号化し、バイト比較
- TPGD on/off の両モード
- 幅が32の倍数でない画像のエッジケース

**PR**: 1本

---

### Phase 4: シンボル辞書符号化 (`symbol/dictionary.rs`)

**計画書**: `docs/plans/400_symbol_dictionary.md`
**C++対応**: `jbig2sym.cc` の `jbig2enc_symboltable()` (約90行)
**依存**: Phase 1 + leptonica

**スコープ**:
- シンボルを高さ順 → 幅順にソート
- デルタ高さ/幅の整数符号化（IADH/IADW）
- 各シンボルビットマップの符号化
- エクスポートテーブルの符号化
- ボーダーサイズ: `leptonica::recog::jbclass::TEMPLATE_BORDER`（= 4）を使用（C++の`kBorderSize = 6`とは異なる）

**テスト**:
- 既知サイズのシンボル群のソート順検証
- デルタ値の正確性
- C++との構造比較（ボーダーサイズ差によりバイト一致は不可、デコード結果で検証）

**PR**: 1本

---

### Phase 5: テキストリージョン符号化 (`symbol/text_region.rs`)

**計画書**: `docs/plans/500_text_region.md`
**C++対応**: `jbig2sym.cc` の `jbig2enc_textregion()` (約245行)
**依存**: Phase 1 + Phase 4 + leptonica

**スコープ**:
- シンボルをY座標でストリップに分割
- ストリップ内で左→右にソート
- 位置デルタ符号化（IADT/IAFS/IADS/IAIT）
- シンボルID符号化（IAID）
- リファインメント符号化（オプション）: IARI フラグ + IARDW/IARDH/IARDX/IARDY + refine bitmap

**注意**: リファインメントはC++版でも「broken」とコメントされている。基本実装は行うが、検証は仕様ベースで行う。

**テスト**:
- 既知配置のシンボル群でストリップ分割・ソート検証
- デルタ T/FS/S/IT 値の検証
- リファインメントなしモードをまず完成、リファインメントは別テスト

**PR**: 2本（コア + リファインメント）

---

### Phase 6: シンボル比較 (`comparator/`)

**計画書**: `docs/plans/600_comparator.md`
**C++対応**: `jbig2comparator.cc` (262行)
**依存**: leptonica のみ（Phase 1-5と並行開発可能）

**スコープ**:
```rust
pub fn are_equivalent(first: &Pix, second: &Pix) -> Result<bool, Jbig2Error>;
```
- サイズチェック、全体差分ピクセル数チェック
- 9×9グリッドの空間差分分析
- 水平線・垂直線・交差パターン検出
- 集中差分検出（楕円閾値）

**leptonica-rs API**: `Pix::xor()`, `Pix::count_pixels()`, `Pix::threshold_pixel_sum()`, `Pix::get_pixel()`, `Pix::sizes_equal()`

**テスト**:
- 同一シンボルペア → true
- 微小ノイズ付きペア → true
- 異なる文字ペア → false
- 閾値境界のエッジケース

**PR**: 1本

---

### Phase 7: マルチページエンコーダ (`encoder/context.rs`, `encoder/multipage.rs`)

**計画書**: `docs/plans/700_multipage_encoder.md`
**C++対応**: `jbig2enc.cc` の主要関数群 (約650行)
**依存**: Phase 3 + 4 + 5 + 6

**スコープ**:
```rust
pub struct Jbig2Context { ... }

impl Jbig2Context {
    pub fn new(thresh: f32, weight: f32, xres: u32, yres: u32,
               full_headers: bool, refine_level: i32) -> Result<Self>;
    pub fn add_page(&mut self, pix: &Pix) -> Result<()>;
    pub fn pages_complete(&mut self) -> Result<Vec<u8>>;
    pub fn produce_page(&mut self, page_no: usize,
                        xres: Option<u32>, yres: Option<u32>) -> Result<Vec<u8>>;
    pub fn auto_threshold(&mut self);
    pub fn auto_threshold_using_hash(&mut self);
}
```

**内部状態**:
- `JbClasser` (leptonica-rs) — シンボル分類器
- `page_comps: HashMap<usize, Vec<usize>>` — ページ→コンポーネントインデックス
- `single_use_symbols: HashMap<usize, Vec<usize>>` — ページ固有シンボル
- `sym_map: HashMap<usize, usize>` — シンボルインデックス→JBIG2シンボル番号

**設計決定**:
- `Vec<u8>` を返却（C++のmalloc+caller freeパターンではなく）
- `Option<u32>` でxres/yresオーバーライド（C++の`-1`規約ではなく）
- ボーダーサイズ: `TEMPLATE_BORDER`（= 4）を使用
- シンボル統合: `Pixa::replace()` / `remove()` による所有権ベースのロジック（C++のrefcount操作ではなく）

**テスト**:
- マルチページTIFFまたはPNG群でE2Eテスト
- 出力がデコード可能か検証（JBIG2デコーダで復元画像を入力と比較）
- auto_thresholdによるシンボル数削減の検証

**PR**: 3-4本
1. `Jbig2Context::new()` + `add_page()`
2. `pages_complete()`（シンボルテーブル生成）
3. `produce_page()`（テキストリージョン生成）
4. `auto_threshold` + `auto_threshold_using_hash`

---

### Phase 8: CLIツール (`main.rs`)

**計画書**: `docs/plans/800_cli_tool.md`
**C++対応**: `jbig2.cc` (618行)
**依存**: Phase 7

**スコープ**:
- `clap` による引数パース
- 画像読み込み（leptonica-rs I/O）
- グレースケール→二値変換
- テキスト/グラフィクスセグメンテーション（`morph_sequence` で `r11`, `r1143 + o4.4 + x4`, `d3.3`）
- 出力書き込み

**PR**: 1-2本

---

## リスク

### 1. ビット精度の互換性

算術符号化は`encode_bit`/`byteout`/`emit`のステートマシンがビット単位で正確でなければ出力全体が壊れる。

**対処**: C++テストハーネスで中間状態（c/a/ct/b/bp）をダンプし、Rust実装とステップ比較。H.2テストベクタを最初に通す。

### 2. TEMPLATE_BORDER の不一致

C版leptonica: `JB_ADDED_PIXELS = 6`, leptonica-rs: `TEMPLATE_BORDER = 4`。テンプレート画像のサイズが異なるため、C++版とのバイト一致は不可能。

**対処**: ボーダーサイズをパラメータ化。正確性はデコード結果で検証（バイト比較ではなく）。

### 3. Pix データレイアウト

`Pix::data()` は `&[u32]` で、MSBファーストのピクセル順序。C++の`encode_bitimage`は`PIX->data`を`u32*`としてアクセス。同等のはずだが検証が必要。

**対処**: 既知パターンの1bpp画像で`data()`の値を確認するテストを追加。幅が32の倍数でない場合のパッドビット処理を重点的に検証。

### 4. JbClasser の ptall 座標系

C++版は`jbGetLLCorners()`で左下角座標を取得。leptonica-rsの`ptall`もadd_page時に下端座標を格納するが、ボーダーサイズの差による座標オフセットの可能性がある。

**対処**: 既知画像で分類を行い、ptall値を検証するフォーカステストを作成。

### 5. リファインメント符号化の検証困難

C++版でも「broken」とコメントされており、動作する参照実装がない。

**対処**: 仕様ベースで実装。Phase 5のリファインメント部分は実験的とし、基本機能（リファインメントなし）を優先。

---

## 検証方法

### ユニットテスト（各Phase内）
```bash
cargo test
```

### リント・フォーマット
```bash
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

### E2E検証（Phase 3以降）
1. テスト用1bpp画像を`tests/data/`に用意
2. Rust版で符号化 → 出力ファイル生成
3. ジェネリックリージョン（Phase 3）: C++版と出力をバイト比較
4. シンボルモード（Phase 7）: JBIGデコーダで復元し入力画像と比較（ボーダーサイズ差のためバイト比較不可）

### CI
- `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt` を全フェーズで実行

---

## 重要ファイル参照

| 用途 | パス |
|------|------|
| C++ 公開API | `reference/jbig2enc/src/jbig2enc.h` |
| C++ エンコーダ本体 | `reference/jbig2enc/src/jbig2enc.cc` |
| C++ 算術符号化 | `reference/jbig2enc/src/jbig2arith.cc` |
| C++ シンボル符号化 | `reference/jbig2enc/src/jbig2sym.cc` |
| C++ シンボル比較 | `reference/jbig2enc/src/jbig2comparator.cc` |
| C++ データ構造 | `reference/jbig2enc/src/jbig2structs.h` |
| C++ セグメントヘッダ | `reference/jbig2enc/src/jbig2segments.h` |
| C++ CLIツール | `reference/jbig2enc/src/jbig2.cc` |
| JBIG2仕様書 | `reference/jbig2enc/doc/fcd14492.pdf` |
| API互換性調査 | `reference/leptonica-rs/docs/porting/jbig2enc-api-compatibility.md` |
| leptonica-rs JbClasser | `reference/leptonica-rs/src/recog/jbclass/types.rs:276` (`TEMPLATE_BORDER = 4`) |
| leptonica-rs ROP | `reference/leptonica-rs/src/core/pix/rop.rs:552` (`rop_region_inplace`) |
| leptonica-rs morph | `reference/leptonica-rs/src/morph/sequence.rs` (`RankReduce`, `BinaryExpand`) |
