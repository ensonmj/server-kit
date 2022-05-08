pub type Result<T> = std::result::Result<T, Error>;

/// An error type that combines all possible errors by this library.
#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub enum Error {
    Err(String),
    MagicNum(String),
    /// Io error from tcp
    Io(#[from] std::io::Error),
    Toml(#[from] toml::de::Error),
    TraceErr(#[from] opentelemetry::trace::TraceError),
}
