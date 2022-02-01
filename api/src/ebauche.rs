//! Implements a simple authorized proxy to the Ebauche cache server.
//! This is necessary because funging other users' Canvas base URLs and IDs is trivial.

use async_bincode::AsyncBincodeStream;
use chrono::{DateTime, TimeZone, Utc};
use futures::prelude::*;
use miette::{miette, IntoDiagnostic, WrapErr};
use poem::{
    handler,
    web::{
        websocket::{self, WebSocket},
        Data, Query,
    },
    EndpointExt, IntoEndpoint, IntoResponse, Route,
};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct Api {
    pub address: SocketAddr,
}

impl IntoEndpoint for Api {
    type Endpoint = Route;
    fn into_endpoint(self) -> Self::Endpoint {
        let arced = Arc::new(self);
        Route::new()
            .at("/update", update.data(arced.clone()))
            .at("/fetch", fetch.data(arced))
    }
}

#[derive(Deserialize)]
struct UpdateQuery {
    canvas: String,
    user_id: canvas::Id,
    resource_kind: ebauche_rpc::ResourceKind,
    since: DateTime<Utc>,
}

#[handler]
async fn update(
    ebauche: Data<&Arc<Api>>,
    ws: WebSocket,
    query: Query<UpdateQuery>,
) -> impl IntoResponse {
    use pigment::view::*;

    let ebauche = ebauche.clone();
    ws.on_upgrade(move |mut socket| async move {
        let req = ebauche_rpc::Request::Update {
            since: query.since,
            view: View {
                truth: Canvas {
                    base_url: query.canvas.clone(),
                },
                viewer: Viewer::User(query.user_id),
            },
            resource_kind: query.resource_kind,
        };
        let mut transport = AsyncBincodeStream::from(
            TcpStream::connect(ebauche.address)
                .await
                .into_diagnostic()
                .wrap_err("failed connecting to Ebauche")?,
        );
        let mut responses = req.send(&mut transport).await?;
        while let Some(resp) = responses.next().await {
            // TODO: currently, we are deserializing the entire message only to immediately reserialize it
            //        this is fucking stupid, but it'll do for now

            let resp: ebauche_rpc::Response = resp
                .into_diagnostic()
                .wrap_err("failed recieving message from Ebauche?")?
                .map_err(|msg| miette!(msg).wrap_err("Ebauche response was error"))?;

            if let ebauche_rpc::Response::Update(resp) = resp {
                socket
                    .send(websocket::Message::Binary(
                        bincode::serialize(&resp)
                            .into_diagnostic()
                            .wrap_err("failed to serialize update response")?,
                    ))
                    .await
                    .into_diagnostic()
                    .wrap_err("failed to send message over WebSocket")?;
            }
        }

        tracing::info!("exhausted response stream, finishing...");

        Ok::<(), miette::ErrReport>(())
    })
}

#[derive(Deserialize)]
struct FetchQuery {
    canvas: String,
    user_id: canvas::Id,
}

#[handler]
async fn fetch(
    ebauche: Data<&Arc<Api>>,
    ws: WebSocket,
    query: Query<FetchQuery>,
) -> impl IntoResponse {
    use pigment::view::*;

    let ebauche = ebauche.clone();
    ws.on_upgrade(move |mut socket| async move {
        let req = ebauche_rpc::Request::Fetch {
            canvas_token: todo!("fetch canvas token from database"),
            view: View {
                truth: Canvas {
                    base_url: query.canvas.clone(),
                },
                viewer: Viewer::User(query.user_id),
            },
        };
        let mut transport = AsyncBincodeStream::from(
            TcpStream::connect(ebauche.address)
                .await
                .into_diagnostic()
                .wrap_err("failed connecting to Ebauche")?,
        );
        let mut responses = req.send(&mut transport).await?;
        while let Some(resp) = responses.next().await {
            // TODO: currently, we are deserializing the entire message only to immediately reserialize it
            //        this is fucking stupid, but it'll do for now

            let resp: ebauche_rpc::Response = resp
                .into_diagnostic()
                .wrap_err("failed recieving message from Ebauche?")?
                .map_err(|msg| miette!(msg).wrap_err("Ebauche response was error"))?;

            if let ebauche_rpc::Response::Fetch(resp) = resp {
                socket
                    .send(websocket::Message::Binary(
                        bincode::serialize(&resp)
                            .into_diagnostic()
                            .wrap_err("failed to serialize update response")?,
                    ))
                    .await
                    .into_diagnostic()
                    .wrap_err("failed to send message over WebSocket")?;
            }
        }

        Ok::<(), miette::ErrReport>(())
    })
}
