use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
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
