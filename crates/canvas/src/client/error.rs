use miette::{Diagnostic, SourceOffset};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    Hyper(#[from] hyper::Error),

    #[error(transparent)]
    Http(#[from] hyper::http::Error),

    #[error("missing `Links` header")]
    MissingLinksHeader,

    #[error("failed to parse `Links` header: {message}")]
    MalformedLinkHeader {
        #[source_code]
        src: String,
        message: &'static str,
    },

    #[error("missing pagination link with relevance `{0}`")]
    MissingPaginationLink(&'static str),

    #[error("while parsing JSON data from Canvas")]
    #[diagnostic(code(canvas_lms::malformed_json))]
    MalformedJson {
        #[source]
        source: serde_json::error::Error,
        #[source_code]
        json: String,
        #[label("here")]
        err_loc: (usize, usize),
    },
}

impl Error {
    // ATTRIBUTION: from Kat Marchán's `turron` project, which is licensed under Apache 2.0
    pub fn from_json_err(err: serde_json::Error, json: String) -> Self {
        // These json strings can get VERY LONG and miette doesn't (yet?)
        // support any "windowing" mechanism for displaying stuff, so we have
        // to manually shorten the string to only the relevant bits and
        // translate the spans accordingly.
        let err_offset = SourceOffset::from_location(&json, err.line(), err.column());
        let json_len = json.len();
        let local_offset = err_offset.offset().saturating_sub(40);
        let local_len = std::cmp::min(40, json_len - err_offset.offset());
        let snipped_json = json[local_offset..err_offset.offset() + local_len].to_string();
        Self::MalformedJson {
            source: err,
            json: snipped_json,
            err_loc: (err_offset.offset() - local_offset, 0),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
