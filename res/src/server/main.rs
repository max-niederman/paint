extern crate canvas_lms as canvas;

mod fetch;

use fetch::Fetch;
use futures::{future, stream::StreamExt};
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let client = canvas::Client::new(
        "https://lms.pps.net",
        Some(canvas::Auth::Bearer(std::env::var("CANVAS_TOKEN").unwrap())),
    );

    let mut courses = canvas::resource::Course::fetch_all(client.clone())?;
    while let Some(course) = courses.next().await {
        println!("{:#?}", course?);
    }

    todo!("implement RPC server")
}
