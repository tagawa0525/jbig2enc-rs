mod cli;
mod pipeline;

use clap::Parser;

use cli::{Args, CliError};

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
    if let Err(e) = run(&args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// メインパイプラインを実行する。
///
/// 引数に従い、入力画像を読み込み、二値化し、JBIG2 形式で出力する。
#[allow(unused_variables)] // GREEN で実装
fn run(_args: &Args) -> Result<(), CliError> {
    todo!("PR 3 で実装")
}
