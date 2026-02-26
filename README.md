# jbig2enc

[日本語版はこちら](README.ja.md)

A Rust reimplementation of [jbig2enc](https://github.com/agl/jbig2enc), a JBIG2 encoder for bi-level (1 bpp) images.

[JBIG2](https://www.itu.int/rec/T-REC-T.88/en) is a compression standard for bi-level images that achieves better compression ratios than G4 (CCITT Group 4) through symbol extraction and dictionary-based reuse. It is commonly used for embedding scanned document images into PDFs.

This crate provides both a library API and a command-line tool. Written entirely in Rust with no C/C++ dependencies -- the image processing foundation [leptonica](https://github.com/tagawa0525/leptonica-rs) is also a pure Rust reimplementation, so the entire toolchain builds with `cargo` alone.

## Installation

### Library

```toml
[dependencies]
jbig2enc = "0.1"
```

To use only as a library without the CLI binary:

```toml
[dependencies]
jbig2enc = { version = "0.1", default-features = false }
```

### CLI

```bash
cargo install jbig2enc
```

## Library Usage

Single-page lossless encoding (generic region):

```rust,no_run
use jbig2enc::encoder::encode_generic;
use leptonica::io::read_image;

let pix = read_image("input.png").unwrap();
let data = encode_generic(&pix, true, 300, 300, false).unwrap();
std::fs::write("output.jbig2", &data).unwrap();
```

Multi-page symbol mode encoding:

```rust,no_run
use jbig2enc::encoder::Jbig2Context;
use leptonica::io::read_image;

let mut ctx = Jbig2Context::new(0.92, 0.5, 300, 300, true, -1).unwrap();
let page = read_image("page1.png").unwrap();
ctx.add_page(&page).unwrap();

let symbol_table = ctx.pages_complete().unwrap();
let page_data = ctx.produce_page(0, None, None).unwrap();
```

### Modules

- `arith` -- QM arithmetic coder, the core entropy coding engine
- `comparator` -- Symbol template equivalence detection
- `encoder` -- Multi-page compression context and generic region encoding
- `error` -- Error types
- `symbol` -- Symbol dictionary and text region encoding
- `wire` -- JBIG2 wire format structures

## CLI Usage

```bash
# Single-page generic encoding (output to stdout)
jbig2enc input.png > output.jbig2

# Symbol mode with PDF-ready output
jbig2enc -s -p input.png

# Multi-page symbol mode
jbig2enc -s -p page1.png page2.png page3.png

# With duplicate line removal
jbig2enc -d input.png > output.jbig2

# Custom threshold and DPI
jbig2enc -s -p -t 0.85 -D 300 input.png
```

Run `jbig2enc --help` for the full list of options.

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `cli` | Yes | Builds the `jbig2enc` command-line binary (depends on `clap`) |

## Minimum Supported Rust Version

Rust 2024 edition (1.87+).

## License

This project is distributed under the [Apache License 2.0](LICENSE), the same license as the original [jbig2enc](https://github.com/agl/jbig2enc).

## Acknowledgments

This project relies on the source code and design of the original C++ jbig2enc by Adam Langley. It also depends on [leptonica](https://github.com/tagawa0525/leptonica-rs) as its image processing foundation, which in turn is a reimplementation of [Leptonica](http://www.leptonica.org/) by Dan Bloomberg.

## How This Project Is Built

The porting work is carried out primarily by AI coding agents, including [Claude Code](https://docs.anthropic.com/en/docs/claude-code). A human maintainer defines the overall architecture, process rules, and acceptance criteria, while the agents read the original C++ source, write Rust code, and run tests under those constraints. Every commit goes through CI and automated review before merging.
