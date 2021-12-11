use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// An error returned by a handler.
    #[error("request handler failed")]
    #[diagnostic(code(pigment::rpc::request_handler))]
    Handler(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// A transport error.
    #[error("rpc transport failed")]
    #[diagnostic(code(pigment::rpc::transport))]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),
}

macro_rules! boxed_source_constructor {
    ($name:ident -> Self::$var:ident) => {
        #[inline]
        pub fn $name<T>(err: T) -> Self
        where
            T: std::error::Error + Send + Sync + 'static,
        {
            Self::$var(Box::new(err))
        }
    };
}

impl Error {
    boxed_source_constructor!(handler -> Self::Handler);
    boxed_source_constructor!(transport -> Self::Transport);
}
