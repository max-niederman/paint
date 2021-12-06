use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    Cache(#[from] crate::cache::Error),

    #[error(transparent)]
    Rpc(#[from] crate::rpc::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
