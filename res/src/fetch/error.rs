use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// An error returned by the Canvas API Client.
    #[error(transparent)]
    Canvas(#[from] canvas::client::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
