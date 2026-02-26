mod cli;
mod pipeline;

use std::fs;
use std::io::Write;
use std::process;

use clap::Parser;
use jbig2enc_rs::encoder::{Jbig2Context, encode_generic};
use leptonica::Pix;
use leptonica::io::{ImageFormat, write_image};

use cli::{Args, CliError};

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("Error: {e}");
        process::exit(1);
    }
    if let Err(e) = run(&args) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

/// 入力ファイルを読み込んで Pix のリストを返す。
///
/// TIFF マルチページの場合は全ページを展開する。
/// TIFF 以外（または単一ページ TIFF）は 1 要素のベクタとして返す。
fn load_pages(file_path: &str) -> Result<Vec<Pix>, CliError> {
    use std::io::{BufReader, Cursor};

    // ファイルを一度だけ読み込み、TIFF 検出と読み出しの両方で共有する
    let data = fs::read(file_path).map_err(CliError::Io)?;

    // TIFF マルチページ検出: ページ数が取得できれば TIFF として処理する
    let page_count = leptonica::io::tiff::tiff_page_count(BufReader::new(Cursor::new(&data[..])))
        .map_err(|e| CliError::Image(e.to_string()));

    match page_count {
        Ok(n) if n > 1 => {
            leptonica::io::tiff::read_tiff_multipage(BufReader::new(Cursor::new(&data[..])))
                .map_err(|e| CliError::Image(e.to_string()))
        }
        _ => leptonica::io::read_image_mem(&data)
            .map(|p| vec![p])
            .map_err(|e| CliError::Image(e.to_string())),
    }
}

/// メインパイプラインを実行する。
///
/// 引数に従い、入力画像を読み込み、二値化し、JBIG2 形式で出力する。
fn run(args: &Args) -> Result<(), CliError> {
    let bw_threshold = args.effective_bw_threshold();

    // シンボルモード: コンテキストを初期化
    let mut ctx_opt: Option<Jbig2Context> = if args.symbol_mode {
        Some(
            Jbig2Context::new(args.threshold, args.weight, 0, 0, !args.pdf, -1)
                .map_err(|e| CliError::Image(e.to_string()))?,
        )
    } else {
        None
    };

    let mut num_pages = 0usize;

    for file_path in &args.files {
        let pages = load_pages(file_path)
            .map_err(|e| CliError::Image(format!("failed to read '{file_path}': {e}")))?;

        for source in pages {
            // DPI 強制（画像に DPI 情報がない場合のみ）
            let source = if let Some(dpi) = args.dpi {
                if source.xres() == 0 && source.yres() == 0 {
                    // ロード直後は refcount=1 なのでゼロコピー変換できる。
                    // 参照カウントが増えている場合は DPI 設定をスキップして続行する。
                    match source.try_into_mut() {
                        Ok(mut pm) => {
                            pm.set_resolution(dpi as i32, dpi as i32);
                            pm.into()
                        }
                        Err(source) => source,
                    }
                } else {
                    source
                }
            } else {
                source
            };

            // 二値化
            let pixt = pipeline::binarize(source, args.global, bw_threshold, args.up2, args.up4)?;

            // -O: デバッグ用 PNG 出力（複数ページ処理時は最後のページで上書き、C++ 版と同動作）
            if let Some(ref out_path) = args.output_threshold {
                write_image(&pixt, out_path, ImageFormat::Png)
                    .map_err(|e| CliError::Image(format!("failed to write '{out_path}': {e}")))?;
            }

            if !args.symbol_mode {
                // Generic モード: 1 ページのみ処理して stdout に出力
                let xres = pixt.xres().max(0) as u32;
                let yres = pixt.yres().max(0) as u32;
                let data =
                    encode_generic(&pixt, !args.pdf, xres, yres, args.duplicate_line_removal)
                        .map_err(|e| CliError::Image(e.to_string()))?;
                std::io::stdout().write_all(&data).map_err(CliError::Io)?;
                return Ok(());
            }

            // Symbol モード: ページをコンテキストに追加
            if let Some(ref mut ctx) = ctx_opt {
                ctx.add_page(&pixt)
                    .map_err(|e| CliError::Image(e.to_string()))?;
                num_pages += 1;
            }
        }
    }

    if let Some(mut ctx) = ctx_opt {
        // 自動閾値処理
        if args.auto_thresh {
            if !args.no_hash {
                ctx.auto_threshold_using_hash();
            } else {
                ctx.auto_threshold();
            }
        }

        // シンボルテーブルを書き出す
        let sym_data = ctx
            .pages_complete()
            .map_err(|e| CliError::Image(e.to_string()))?;
        if args.pdf {
            let sym_path = format!("{}.sym", args.basename);
            fs::write(&sym_path, &sym_data).map_err(CliError::Io)?;
        } else {
            std::io::stdout()
                .write_all(&sym_data)
                .map_err(CliError::Io)?;
        }

        // 各ページを書き出す
        for i in 0..num_pages {
            let page_data = ctx
                .produce_page(i, None, None)
                .map_err(|e| CliError::Image(e.to_string()))?;
            if args.pdf {
                let page_path = format!("{}.{i:04}", args.basename);
                fs::write(&page_path, &page_data).map_err(CliError::Io)?;
            } else {
                std::io::stdout()
                    .write_all(&page_data)
                    .map_err(CliError::Io)?;
            }
        }
    }

    Ok(())
}
