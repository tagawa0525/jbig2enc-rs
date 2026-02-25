mod cli;

use clap::Parser;

use cli::Args;

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
    // 処理パイプラインは PR 3 で実装
    eprintln!("jbig2enc-rs: main pipeline not yet implemented");
    std::process::exit(1);
}
