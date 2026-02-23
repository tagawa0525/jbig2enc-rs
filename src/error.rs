use std::fmt;

#[derive(Debug)]
pub enum Jbig2Error {
    InvalidInput(String),
}

impl fmt::Display for Jbig2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Jbig2Error::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
        }
    }
}

impl std::error::Error for Jbig2Error {}
