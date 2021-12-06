use thiserror::Error;
use miette::Diagnostic;

#[derive(Debug, Error, Diagnostic)]
pub struct Error {
    /// An error returned by a handler.
    #[error("in request handler")]
    #[diagnostic(code(pigment::rpc::request_handler))]
    Handler(#[source] Box<dyn std::error::Error>),

    /// A transport error.
    #[error("in rpc transport")]
    #[diagnostic(code(pigment::rpc::transport))]
    Transport(#[source] tokio::io::Error),   
}