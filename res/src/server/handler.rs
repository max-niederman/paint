use std::pin::Pin;

use crate::fetch::Fetch;
use canvas::resource::*;
use futures::{stream, Stream};
use miette::{Diagnostic, IntoDiagnostic, WrapErr};
use pigment::{
    cache,
    rpc::{self, *},
};

#[derive(Clone, Debug)]
pub struct Handler {
    pub db: sled::Db,
}

impl<'h> rpc::Handler<'h> for Handler {
    type Err = Box<dyn Diagnostic + Send + Sync + 'static>;
    type ResponseStream = Pin<Box<dyn Stream<Item = Result<Response, Self::Err>> + Send + 'h>>;

    fn handle(&'h self, request: Request) -> Self::ResponseStream {
        Box::pin(match request {
            Request::Update { view, canvas_token } => {
                log::debug!("updating {} with token {}", view, canvas_token);

                let canvas_client = canvas::Client::builder()
                    .with_auth(canvas::Auth::Bearer(canvas_token))
                    .with_base_url(view.truth.base_url.clone())
                    .build();

                stream::once(async move {
                    cache::replace_view(
                        &self
                            .db
                            .open_tree("courses")
                            .into_diagnostic()
                            .wrap_err("failed to open sled tree")?,
                        &view,
                        &mut Course::fetch_all(canvas_client),
                    )
                    .await??;

                    Ok(Response::UpdateFinished)
                })
            }
            _ => todo!(),
        })
    }
}
