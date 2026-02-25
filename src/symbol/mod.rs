mod dictionary;
mod text_region;

pub use dictionary::{SymbolTableResult, encode_symbol_table};
pub use text_region::{SymbolInstance, TextRegionConfig, TextRegionResult, encode_text_region};
