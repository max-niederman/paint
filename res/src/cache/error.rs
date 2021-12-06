use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    Sled(#[from] sled::Error),

    #[error("while deserializing cache entry")]
    Deserialization(#[source] bincode::Error),

    #[error("while serializing {value:#?}")]
    Serialization {
        #[source]
        source: bincode::Error,
        value: Box<dyn std::fmt::Debug>,
    },

    #[error("expected {expected} but got {actual}")]
    UnexpectedStreamYield {
        expected: &'static str,
        actual: &'static str,
    },

    #[error("Canvas base URL was illegal: {problem}")]
    IllegalCanvasBaseUrl {
        #[source_code]
        base_url: String,
        #[label = "here"]
        location: Option<(usize, usize)>,
        problem: &'static str,
    },

    #[error("illegal Viewer discriminant: {discriminant}")]
    IllegalViewerDiscriminant { discriminant: u8 },
}
