use futures::Stream;
use miette::{IntoDiagnostic, Result, WrapErr};

/// Responsible for fetching a resource from the underlying Canvas API.
pub trait Fetch: Sized {
    type FetchAll: Stream<Item = Result<Self>>;
    fn fetch_all(&self, client: &canvas::Client) -> Result<Self::FetchAll>;
}

mod impls {
    use std::pin::Pin;

    use super::*;
    use canvas::resource::*;
    use futures::{stream::FuturesUnordered, Future};

    impl Fetch for Course {
        type FetchAll = Pin<Box<dyn Stream<Item = Result<Course>> + Send>>;
        fn fetch_all(&self, client: &canvas_lms::Client) -> Result<Self::FetchAll> {
            todo!()
        }
    }
}
