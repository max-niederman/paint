use async_stream::stream;
use canvas::{
    client::{deserialize_from_slice, Pagination},
    resource,
};
use futures::Stream;
use miette::{IntoDiagnostic, Result, WrapErr};
use std::pin::Pin;

/// Responsible for fetching a resource from the underlying Canvas API.
pub trait Fetch: Sized {
    type FetchAll: Stream<Item = Result<Self>> + Send + Sync;
    fn fetch_all(client: canvas::Client) -> Self::FetchAll;
}

impl Fetch for resource::Course {
    type FetchAll = Pin<Box<dyn Stream<Item = Result<Self>> + Send + Sync>>;
    fn fetch_all(client: canvas::Client) -> Self::FetchAll {
        Box::pin(stream! {
            // we don't use [`RequestBuilder::query`] because it would add paramaters on each iteration
            let mut link = format!("{}/api/v1/courses?per_page=50?include[]=syllabus_body&include[]=total_scores&include[]=current_grading_period_scores", client.base_url());
            loop {
                let resp = client
                    .http_client()
                    .get(link)
                    .send()
                    .await
                    .into_diagnostic()
                    .wrap_err("while fetching resources from Canvas")?;
                log::debug!("fetched course list from <{}>", resp.url());

                let next_link = Pagination::from_headers(resp.headers())?.next().map(ToString::to_string);
                let body = resp
                    .bytes()
                    .await
                    .into_diagnostic()
                    .wrap_err("while reading response body")?;
                let courses = deserialize_from_slice::<Vec<Self>>(&body)
                    .wrap_err("while parsing fetched resources")?; // TODO: add other metadata to wrapper (e.g. source URL)

                for course in courses {
                    yield Ok(course);
                }

                if let Ok(next_link) = next_link {
                    link = next_link;
                } else {
                    break;
                }
            }
        })
    }
}
