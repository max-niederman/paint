use async_stream::stream;
use futures::Stream;
use miette::Result;

/// Responsible for fetching a resource from the underlying Canvas API.
pub trait Fetch: Sized {
    type FetchAll: Stream<Item = Result<Self>>;
    fn fetch_all(client: canvas::Client) -> Result<Self::FetchAll>;
}

mod impls {
    use std::pin::Pin;

    use super::*;
    use canvas::{
        client::{deserialize_from_slice, Pagination},
        resource::*,
    };

    use miette::{IntoDiagnostic, WrapErr};

    impl Fetch for Course {
        type FetchAll = Pin<Box<dyn Stream<Item = Result<Course>>>>;
        fn fetch_all(client: canvas::Client) -> Result<Self::FetchAll> {
            Ok(Box::pin(stream! {
                // we don't use [`RequestBuilder::query`] because it would add paramaters on each iteration
                let mut link = format!("{}/api/v1/courses?per_page=50?include[]=syllabus_body&include[]=total_scores&include[]=current_grading_period_scores", client.base_url());
                loop {
                    let resp = client
                        .http_client()
                        .get(link)
                        .send()
                        .await
                        .into_diagnostic()
                        .wrap_err("failed to fetch resources from Canvas")?;
                    log::debug!("fetched course list from <{}>", resp.url());

                    let next_link = Pagination::from_headers(resp.headers())?.next().map(ToString::to_string);
                    let body = resp
                        .bytes()
                        .await
                        .into_diagnostic()
                        .wrap_err("failed to read response body")?;
                    let courses = deserialize_from_slice::<Vec<Course>>(&body)
                        .wrap_err("failed to parse fetched resources")?; // TODO: add other metadata to wrapper (e.g. source URL)

                    for course in courses {
                        yield Ok(course);
                    }

                    if let Ok(next_link) = next_link {
                        link = next_link;
                    } else {
                        break;
                    }
                }
            }))
        }
    }
}