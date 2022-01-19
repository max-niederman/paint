//! Responsible for communication with Oil.

use super::Stores;
use chrono::{DateTime, Utc};
use miette::{IntoDiagnostic, Result, WrapErr};
use pigment::view::{View, Viewer};
use std::rc::Rc;
use ws_stream_wasm::*;

#[tracing::instrument("updating stores", skip(stores))]
pub async fn update_stores(stores: Rc<Stores>, since: DateTime<Utc>, view: View) -> Result<()> {
    let (_meta, stream): (_, WsStream) = WsMeta::connect(
        &format!(
            "wss://{oil}/ebauche/update?since={since}&canvas={canvas}&user_id={user_id}",
            oil = "localhost:4210", // FIXME: make oil address configurable
            since = since,
            canvas = view.truth.base_url,
            user_id = {
                let Viewer::User(id) = view.viewer;
                id
            }
        ),
        None,
    )
    .await
    .into_diagnostic()
    .wrap_err("failed to create WebSocket client")?;

    todo!()
}
