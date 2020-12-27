use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("failed to parse json")]
    Parsing(#[from] ::serde_json::error::Error),
	#[error("encountered io error")]
    IOErr(#[from] ::std::io::Error),
	#[error("failed to decode text as utf-8")]
    UTF8Err(#[from] ::std::string::FromUtf8Error),
}
