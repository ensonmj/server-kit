pub type Result<T> = std::result::Result<T, Error>;

/// An error type that combines all possible errors by this library.
#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub enum Error {
    Err(String),
    Parse(#[from] ParseError),
    /// Io error from tcp
    Io(#[from] std::io::Error),
    Toml(#[from] toml::de::Error),
    TraceErr(#[from] opentelemetry::trace::TraceError),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("try other protocol")]
    TryOther,
    #[error("unexpected eof")]
    UnexpectedEof,
}
