pub type Result<T> = std::result::Result<T, Error>;

/// An error type that combines all possible errors by this library.
#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub enum Error {
    StrErr(String),
    Parse(#[from] ParseErr),
    Svc(#[from] SvcErr),
    PbErr(#[from] protobuf::Error),
    /// Io error from tcp
    Io(#[from] std::io::Error),
    Toml(#[from] toml::de::Error),
    TraceErr(#[from] opentelemetry::trace::TraceError),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseErr {
    #[error("try other protocol")]
    TryOther,
    #[error("unexpected eof")]
    UnexpectedEof,
}

#[derive(thiserror::Error, Debug)]
pub enum SvcErr {
    #[error("service {0} exist")]
    Exist(String),
    #[error("service {0} not exist")]
    NotExist(String),
}
