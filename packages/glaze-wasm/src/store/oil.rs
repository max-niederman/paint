//! Responsible for communication with Oil.

use super::{GlazeStore, Stores};
use chrono::{DateTime, Utc};
use ebauche_rpc::{ResourceKind, UpdateResponse};
use futures::prelude::*;
use miette::{bail, IntoDiagnostic, Result, WrapErr};
use pigment::{
    cache::{self, increment_key, Key, Store},
    view::{View, Viewer},
};
use std::rc::Rc;
use ws_stream_wasm::*;

#[tracing::instrument("updating store")]
pub fn update_store<'a>(
    store: &'a GlazeStore,
    resource_kind: ResourceKind,
    since: DateTime<Utc>,
    view: &'a View,
) -> impl Future<Output = Result<()>> + 'a {
    async move {
        #[derive(serde::Serialize)]
        struct QueryVariables<'a> {
            canvas: &'a str,
            user_id: canvas::Id,
            resource_kind: &'static str,
            since: DateTime<Utc>,
        }

        let (_meta, socket): (_, WsStream) = WsMeta::connect(
        &format!(
            "{oil}/ebauche/update?{query}",
            oil = "ws://localhost:4210", // FIXME: make oil address configurable
            query = serde_urlencoded::to_string(
                QueryVariables {
                    canvas: view.truth.base_url.as_str(),
                    user_id: {
                        let Viewer::User(id) = view.viewer;
                        id
                    },
                    resource_kind: resource_kind.as_str(),
                    since,
                }
            ).into_diagnostic()?,
        ),
        None,
    )
    .await
    .into_diagnostic()
    .wrap_err("failed to create WebSocket client")?;

        let mut responses = socket.map(|msg| match msg {
            WsMessage::Binary(bytes) => bincode::deserialize::<UpdateResponse>(&bytes)
                .into_diagnostic()
                .wrap_err("failed to deserialize update message"),
            WsMessage::Text(contents) => {
                bail!("unexpected text message with contents {:#?}", contents)
            }
        });

        let view_serialized = view.serialize()?;
        // the start of the gap between the preceding resource and the current one
        let mut gap_start = view_serialized.clone();

        while let Some(resp) = responses.next().await {
            let resp = resp?;

            let key_bytes = [view_serialized.as_slice(), &resp.key].concat();

            if key_bytes >= gap_start {
                // remove all keys inbetween the last key and this key
                store
                    .resources
                    .range::<Vec<u8>, _>(&gap_start..&key_bytes)
                    .for_each(|entry| {
                        entry.remove();
                    })
            } else {
                return Err(cache::Error::UnexpectedStreamYield {
                    expected: "key lexicographically greater than the last",
                    actual: "key lexicographically less than the last",
                }
                .into());
            }

            if let Some(resource_bytes) = resp.resource {
                store.resources.insert(key_bytes.clone(), resource_bytes);
            }

            // move the key forward by one to get the start of the gap
            // this assumes that the keys will not increase in length
            gap_start = key_bytes;
            increment_key(&mut gap_start);
        }

        Ok(())
    }
}

#[tracing::instrument("updating all stores", skip(stores))]
pub fn update_stores<'a>(
    stores: &'a Rc<Stores>,
    since: DateTime<Utc>,
    view: &'a View,
) -> impl Future<Output = Result<()>> + 'a {
    future::try_join_all([
        update_store(&stores.courses, ResourceKind::Course, since, &view),
        update_store(&stores.assignments, ResourceKind::Assignment, since, &view),
        update_store(&stores.submissions, ResourceKind::Submission, since, &view),
    ])
    .map_ok(|_| ())
}
