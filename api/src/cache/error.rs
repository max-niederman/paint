use miette::Diagnostic;
use poem::error::ResponseError;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {

}

impl ResponseError for Error {
    fn status(&self) -> reqwest::StatusCode {
        unreachable!()
    }
}