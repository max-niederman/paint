#![feature(specialization)]

extern crate canvas_lms as canvas;

mod cache;
mod fetch;

use fetch::Fetch;
use futures::stream::StreamExt;
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let client = canvas::Client::builder()
        .with_base_url("https://lms.pps.net")
        .with_auth(canvas::Auth::Bearer(std::env::var("CANVAS_TOKEN").unwrap()))
        .build();

    let mut courses = canvas::resource::Course::fetch_all(client.clone())?;
    while let Some(course) = courses.next().await {
        log::info!("fetched course '{}'", course?.name);
    }

    todo!("implement RPC server")
}
