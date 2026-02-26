# jbig2enc

[jbig2enc](https://github.com/agl/jbig2enc)（C++製 JBIG2 エンコーダ）の Rust 移植です。1bpp（二値）画像を対象とします。

[JBIG2](https://www.itu.int/rec/T-REC-T.88/en) はシンボル抽出と辞書ベースの再利用により、G4（CCITT Group 4）より高い圧縮率を実現する二値画像の圧縮規格です。スキャン文書の PDF 埋め込みに広く使用されています。

本クレートはライブラリ API とコマンドラインツールの両方を提供します。C/C++ ライブラリへの依存はありません。画像処理基盤の [leptonica](https://github.com/tagawa0525/leptonica-rs) も pure Rust で移植されているため、ツールチェイン全体が `cargo` だけでビルドできます。

## インストール

### ライブラリ

```toml
[dependencies]
jbig2enc = "0.1"
```

CLI バイナリなしでライブラリのみ使用する場合:

```toml
[dependencies]
jbig2enc = { version = "0.1", default-features = false }
```

### CLI

```bash
cargo install jbig2enc
```

## ライブラリの使い方

単一ページのロスレス符号化（ジェネリックリージョン）:

```rust,no_run
use jbig2enc::encoder::encode_generic;
use leptonica::io::read_image;

let pix = read_image("input.png").unwrap();
let data = encode_generic(&pix, true, 300, 300, false).unwrap();
std::fs::write("output.jbig2", &data).unwrap();
```

マルチページのシンボルモード符号化:

```rust,no_run
use jbig2enc::encoder::Jbig2Context;
use leptonica::io::read_image;

let mut ctx = Jbig2Context::new(0.92, 0.5, 300, 300, true, -1).unwrap();
let page = read_image("page1.png").unwrap();
ctx.add_page(&page).unwrap();

let symbol_table = ctx.pages_complete().unwrap();
let page_data = ctx.produce_page(0, None, None).unwrap();
```

### モジュール構成

- `arith` -- QM 算術符号化エンジン
- `comparator` -- シンボルテンプレートの等価性判定
- `encoder` -- マルチページ圧縮コンテキストとジェネリックリージョン符号化
- `error` -- エラー型
- `symbol` -- シンボル辞書・テキストリージョン符号化
- `wire` -- JBIG2 ワイヤフォーマット構造体

## CLI の使い方

```bash
# 単一ページのジェネリック符号化（標準出力に出力）
jbig2enc input.png > output.jbig2

# シンボルモード + PDF 用出力
jbig2enc -s -p input.png

# マルチページのシンボルモード
jbig2enc -s -p page1.png page2.png page3.png

# 重複行除去あり
jbig2enc -d input.png > output.jbig2

# 閾値と DPI を指定
jbig2enc -s -p -t 0.85 -D 300 input.png
```

全オプションは `jbig2enc --help` で確認できます。

## Feature フラグ

| Feature | デフォルト | 説明 |
|---------|-----------|------|
| `cli` | 有効 | `jbig2enc` コマンドラインバイナリをビルド（`clap` に依存） |

## 最低サポート Rust バージョン

Rust 2024 edition（1.87 以上）。

## ライセンス

本プロジェクトは [Apache License 2.0](LICENSE) の下で配布されています。オリジナルの [jbig2enc](https://github.com/agl/jbig2enc) と同じライセンスです。

## 謝辞

本プロジェクトは Adam Langley によるオリジナルの C++ 版 jbig2enc のソースコードと設計に依拠しています。また、画像処理基盤として [leptonica](https://github.com/tagawa0525/leptonica-rs) に依存しています。leptonica は Dan Bloomberg による [Leptonica](http://www.leptonica.org/) の Rust 移植です。

## このプロジェクトの構築方法

移植作業は主に [Claude Code](https://docs.anthropic.com/en/docs/claude-code) を含む AI コーディングエージェントによって行われています。人間のメンテナーがアーキテクチャ、プロセスルール、受け入れ基準を定義し、エージェントがオリジナルの C++ ソースを読み、Rust コードを書き、テストを実行します。すべてのコミットは CI と自動レビューを経てからマージされます。
