use std::fmt::Display;
use hyper::StatusCode;
use miette::Diagnostic;
use poem::{error::ResponseError, Body, Response};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("database error while {action}")]
    #[diagnostic(code(oil::database), help("contact the site administrator"))]
    Database {
        action: String,
        #[source]
        source: mongodb::error::Error,
    },

    #[error("upstream canvas error while {action}")]
    #[diagnostic(code(oil::upstream::canvas), help("ensure your view is valid"))]
    Canvas {
        action: String,
        #[source]
        source: canvas_lms::client::Error,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn database_while(action: impl Display, source: mongodb::error::Error) -> Self {
        Self::Database {
            action: action.to_string(),
            source,
        }
    }
    
    pub fn canvas_while(action: impl Display, source: canvas_lms::client::Error) -> Self {
        Self::Canvas {
            action: action.to_string(),
            source,
        }
    }
}

impl ResponseError for Error {
    fn status(&self) -> StatusCode {
        match self {
            Self::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Canvas { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn as_response(&self) -> Response {
        #[derive(Debug, Serialize)]
        struct ErrResponse {
            description: String,
            source: Option<Box<Self>>,

            code: Option<String>,
            help: Option<String>,
            url: Option<String>,
        }

        impl ErrResponse {
            fn from_diagnostic<E: Diagnostic>(err: &E) -> Self {
                Self {
                    description: err.to_string(),
                    source: err.source().map(Self::from_err).map(Box::new),
                    code: err.code().map(|code| code.to_string()),
                    help: err.help().map(|help| help.to_string()),
                    url: err.url().map(|url| url.to_string()),
                }
            }

            fn from_err<E: std::error::Error>(err: E) -> Self {
                Self {
                    description: err.to_string(),
                    source: err.source().map(Self::from_err).map(Box::new),
                    code: None,
                    help: None,
                    url: None,
                }
            }
        }

        Response::builder().status(self.status()).body(
            Body::from_json(ErrResponse::from_diagnostic(self)).unwrap_or_else(|ser_err| {
                Body::from_string(format!(
                    r#"
                    CRITICAL: failed to serialize error response
                    
                    Error: {:#?}
                    Response: {:#?}
                    Serialization Error: {}
                    "#,
                    self,
                    ErrResponse::from_diagnostic(self),
                    ser_err,
                ))
            }),
        )
    }
}
