# Phase 8: CLI ツール

**Status**: IMPLEMENTED

## Context

Phase 1〜7 で JBIG2 符号化の全コンポーネント（算術符号化、ワイヤフォーマット、ジェネリック領域、シンボル辞書、テキストリージョン、シンボル比較、マルチページエンコーダ）が完成した。Phase 8 はこれらを統合する CLI ツールで、C++ 版 `jbig2.cc`（618 行）に対応する。

## C++ 版対応

| C++ 処理                           | Rust 対応                                                  |
| ---------------------------------- | ---------------------------------------------------------- |
| `main()` 引数解析                  | `clap` derive API                                          |
| `pixRead()`                        | `leptonica::io::read_image()`                              |
| `pixRemoveColormap()`              | `pix.remove_colormap(BasedOnSrc)`                          |
| `pixConvertRGBToGrayFast()`        | `pix.convert_rgb_to_gray_fast()`                           |
| `pixCleanBackgroundToWhite()`      | `leptonica::filter::adaptmap::clean_background_to_white()` |
| `pixThresholdToBinary()`           | `leptonica::color::threshold::threshold_to_binary()`       |
| `pixScaleGray2xLIThresh()`         | `leptonica::transform::scale_gray_2x_li_thresh()`          |
| `pixScaleGray4xLIThresh()`         | `leptonica::transform::scale_gray_4x_li_thresh()`          |
| `segment_image()`                  | 後続 PR で実装（morph_sequence + seedfill_morph）          |
| `jbig2_encode_generic()`           | `encode_generic()`                                         |
| `jbig2_init()` 〜 `produce_page()` | `Jbig2Context` API                                         |

## モジュール構成

```text
src/
  main.rs       -- エントリポイント（parse → run）
  cli.rs        -- clap 引数定義 + バリデーション
  pipeline.rs   -- 画像前処理（読み込み、カラーマップ除去、グレースケール変換、二値化、アップサンプリング）
```

## コマンドラインフラグ

| フラグ              | 型     | デフォルト         | 説明                                 |
| ------------------- | ------ | ------------------ | ------------------------------------ |
| `-b <name>`         | String | `"output"`         | 出力ファイルベース名                 |
| `-d`                | bool   | false              | TPGD 重複行除去                      |
| `-p, --pdf`         | bool   | false              | PDF フラグメントモード               |
| `-s, --symbol-mode` | bool   | false              | シンボル/テキストリージョン符号化    |
| `-t <val>`          | f32    | 0.92               | シンボル分類閾値（0.4–0.97）         |
| `-w <val>`          | f32    | 0.5                | 分類重み（0.1–0.9）                  |
| `-T <val>`          | u8     | 200（`-G` 時 128） | 二値化閾値                           |
| `-G, --global`      | bool   | false              | グローバル閾値（適応的でない）       |
| `-r, --refine`      | bool   | false              | リファインメント（エラーで拒否）     |
| `-O <file>`         | String | なし               | 二値化画像のデバッグ保存             |
| `-2`                | bool   | false              | 2x アップサンプリング                |
| `-4`                | bool   | false              | 4x アップサンプリング                |
| `-S`                | bool   | false              | テキスト/グラフィクス分離（後続 PR） |
| `-j`                | bool   | false              | 分離画像を JPEG で保存（`-S` 依存）  |
| `-a, --auto-thresh` | bool   | false              | 自動シンボル閾値                     |
| `--no-hash`         | bool   | false              | ハッシュ無効化                       |
| `-D <dpi>`          | u32    | なし               | DPI 強制（1–9600）                   |
| `-v`                | bool   | false              | 詳細出力                             |

## バリデーション

- `threshold`: 0.4 ≤ t ≤ 0.97
- `weight`: 0.1 ≤ w ≤ 0.9
- `-2` と `-4` は排他
- `-r` は `-s` を要求（かつ「broken in recent releases」エラー）
- `-D`: 1 ≤ dpi ≤ 9600
- `-S`: "not yet implemented" エラー（初期実装）

## 処理フロー

```text
1. 引数解析 → バリデーション
2. シンボルモード時: Jbig2Context::new()
3. 各入力ファイル:
   a. 画像読み込み（TIFF マルチページ対応: tiff_page_count + read_tiff_page）
   b. DPI 強制（-D かつ画像に DPI なし）
   c. カラーマップ除去: pix.remove_colormap(BasedOnSrc)
   d. 二値化（depth > 1 の場合）:
      - depth > 8: convert_rgb_to_gray_fast()
      - depth 4/8: そのまま
      - 適応的（デフォルト）: clean_background_to_white()
      - グローバル（-G）: スキップ
      - -2: scale_gray_2x_li_thresh()
      - -4: scale_gray_4x_li_thresh()
      - それ以外: threshold_to_binary()
   e. -O: write_image() でデバッグ保存
   f. ジェネリックモード（-s なし）: encode_generic() → stdout, return
   g. シンボルモード: ctx.add_page()
4. シンボルモード後処理:
   a. -a: auto_threshold_using_hash() or auto_threshold()
   b. pages_complete() → シンボルテーブル書き出し
   c. 各ページ: produce_page() → ページデータ書き出し
```

## 出力モード

| モード                  | シンボルテーブル | ページデータ                    |
| ----------------------- | ---------------- | ------------------------------- |
| スタンドアロン（`!-p`） | stdout           | stdout                          |
| PDF（`-p`）             | `{basename}.sym` | `{basename}.0000`, `.0001`, ... |

## エラー型

```rust
enum CliError {
    InvalidArgs(String),
    Io(std::io::Error),
    Image(String),           // leptonica エラーのラップ
    Encode(Jbig2Error),
    NotImplemented(String),  // -S 等の未実装機能
}
```

`Jbig2Error` は `From` で変換。ライブラリの `Jbig2Error` に CLI 固有のエラーを混入させない。

## 依存追加

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

## PR 構成

### PR 1: 引数解析 + バリデーション

1. `docs:` Phase 8 計画書コミット
2. `test:` 引数バリデーションテスト（RED、`#[ignore]`）
   - デフォルト値検証
   - threshold/weight 範囲
   - `-2`/`-4` 排他
   - `-r` は `-s` 必須
   - DPI 範囲
   - `-G` 時の bw_threshold デフォルト変更
3. `feat(cli):` clap 定義 + `Args::validate()`（GREEN）

### PR 2: 画像前処理パイプライン

1. `test:` 前処理テスト（RED、`#[ignore]`）
   - 1bpp 画像はそのまま通過
   - 8bpp → 1bpp 変換
   - 32bpp（RGB）→ 1bpp 変換
   - カラーマップ除去
   - グローバル vs 適応的閾値
   - 2x/4x アップサンプリング
2. `feat(cli):` `pipeline.rs`（load_and_prepare, binarize）（GREEN）

### PR 3: メインパイプライン統合

1. `test:` 統合テスト（RED、`#[ignore]`）
   - ジェネリックモード: 1bpp PNG → stdout、JBIG2 マジックバイト確認
   - シンボルモード スタンドアロン: マルチページ、ファイルヘッダ確認
   - シンボルモード PDF: `.sym` と `.0000` ファイル生成確認
   - auto-threshold 実行確認
   - TIFF マルチページ入力
2. `feat(cli):` `main.rs` の `run()` 完成（GREEN）
3. `docs:` Phase 8 ステータスを IMPLEMENTED に更新

## テスト方針

- **ユニットテスト**（cli.rs, pipeline.rs）: 合成 Pix で引数バリデーションと前処理ロジックを検証
- **統合テスト**（tests/cli.rs）: `std::process::Command` でバイナリを起動し、出力ファイルの存在・マジックバイト・非空を検証
- 符号化の正確性は Phase 1〜7 テストで検証済み。CLI テストでは E2E パイプラインの結合を検証

## 後続対応（別 PR）

- `-S` テキスト/グラフィクス分離: morph_sequence + seedfill_morph + subtract の合成パイプライン
- `-j` JPEG 出力: `-S` 依存
- `-r` リファインメント: C++ でも broken。エラーメッセージのみ

## リスク

| リスク                                                  | 対処                                                                      |
| ------------------------------------------------------- | ------------------------------------------------------------------------- |
| `clean_background_to_white` のパラメータが C++ と異なる | leptonica-rs は内部で同等デフォルト値を使用。C++ の `(1.0, 90, 190)` 相当 |
| `threshold_to_binary` の極性                            | テストで既知ピクセル値の入出力を検証                                      |
| stdout バイナリ出力（Windows）                          | 当面 Unix 前提。Windows 対応は必要時に追加                                |
