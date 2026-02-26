//! Rust reimplementation of [jbig2enc](https://github.com/agl/jbig2enc), a JBIG2 encoder for
//! bi-level (1 bpp) images.
//!
//! JBIG2 is a compression standard for bi-level images that achieves better compression ratios
//! than G4 (CCITT Group 4) through symbol extraction and dictionary-based reuse. It is commonly
//! used for embedding scanned document images into PDFs.
//!
//! # Modules
//!
//! - [`arith`] — QM arithmetic coder, the core entropy coding engine for JBIG2
//! - [`comparator`] — Symbol template equivalence detection using XOR grid analysis
//! - [`encoder`] — Multi-page compression context ([`encoder::Jbig2Context`]) and generic region
//!   encoding ([`encoder::encode_generic`])
//! - [`error`] — Error types ([`error::Jbig2Error`])
//! - [`symbol`] — Symbol dictionary and text region encoding
//! - [`wire`] — JBIG2 wire format structures (file headers, segment headers, region descriptors)
//!
//! # Usage
//!
//! For single-page lossless encoding (generic region):
//!
//! ```no_run
//! use jbig2enc::encoder::encode_generic;
//! use leptonica::io::read_image;
//!
//! let pix = read_image("input.png").unwrap();
//! let data = encode_generic(&pix, true, 300, 300, false).unwrap();
//! std::fs::write("output.jbig2", &data).unwrap();
//! ```
//!
//! For multi-page symbol mode encoding:
//!
//! ```no_run
//! use jbig2enc::encoder::Jbig2Context;
//! use leptonica::io::read_image;
//!
//! let mut ctx = Jbig2Context::new(0.92, 0.5, 300, 300, true, -1).unwrap();
//! let page = read_image("page1.png").unwrap();
//! ctx.add_page(&page).unwrap();
//!
//! let symbol_table = ctx.pages_complete().unwrap();
//! let page_data = ctx.produce_page(0, None, None).unwrap();
//! ```

pub mod arith;
pub mod comparator;
pub mod encoder;
pub mod error;
pub mod symbol;
pub mod wire;
