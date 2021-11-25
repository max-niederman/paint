use super::*;
use canvas_lms::resource::*;

#[async_trait]
impl<S: Selector<Course>> Fetch<Course, S> for Fetcher {
    async fn fetch_superset(&self) -> {
        unimplemented!()
    }
}