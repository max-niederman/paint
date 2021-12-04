use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// An error returned by a handler.
    #[error("in request handler")]
    #[diagnostic(code(pigment::rpc::request_handler))]
    Handler {
        #[source]
        source: Box<dyn std::error::Error>,
    },

    /// A transport error.
    #[error("in rpc transport")]
    #[diagnostic(code(pigment::rpc::transport))]
    Transport(#[source] Box<dyn std::error::Error>),
}

pub type Result<T> = std::result::Result<T, Error>;
