//! Implements a simple authorized proxy to the Ebauche cache server.
//! This is necessary because funging other users' Canvas base URLs and IDs is trivial.

use std::{net::SocketAddr, sync::Arc};

use async_bincode::AsyncBincodeStream;
use chrono::{DateTime, Utc};
use ebauche_rpc::message::*;
use futures::{SinkExt, StreamExt};
use miette::{miette, IntoDiagnostic, WrapErr};
use poem::{
    handler,
    web::{
        websocket::{self, WebSocket},
        Data, Query,
    },
    EndpointExt, IntoEndpoint, IntoResponse, Route,
};
use tokio::net::TcpStream;

/// Represents the information needed to connect to Ebauche.
#[derive(Debug, Clone)]
pub struct Ebauche {
    pub address: SocketAddr,
}

impl IntoEndpoint for Ebauche {
    type Endpoint = Route;
    fn into_endpoint(self) -> Self::Endpoint {
        let arced = Arc::new(self);
        Route::new()
            .at("/update", update.data(arced.clone()))
            .at("/fetch", fetch.data(arced))
    }
}

#[handler]
async fn update(
    ebauche: Data<&Arc<Ebauche>>,
    ws: WebSocket,
    canvas: Query<String>,
    user_id: Query<String>,
    since: Query<DateTime<Utc>>,
) -> impl IntoResponse {
    use pigment::view::*;

    let ebauche = ebauche.clone();
    ws.on_upgrade(move |mut socket| async move {
        let req = Request::Update {
            since: *since,
            view: View {
                truth: Canvas {
                    base_url: canvas.to_string(),
                },
                viewer: Viewer::User(user_id.parse().into_diagnostic()?),
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

            let resp: Response = resp
                .into_diagnostic()
                .wrap_err("failed recieving message from Ebauche?")?
                .map_err(|msg| miette!(msg).wrap_err("Ebauche response was error"))?;

            if let Response::Update(resp) = resp {
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

#[handler]
async fn fetch(
    ebauche: Data<&Arc<Ebauche>>,
    ws: WebSocket,
    canvas: Query<String>,
    user_id: Query<String>,
) -> impl IntoResponse {
    use pigment::view::*;

    let ebauche = ebauche.clone();
    ws.on_upgrade(move |mut socket| async move {
        let req = Request::Fetch {
            canvas_token: todo!("fetch canvas token from database"),
            view: View {
                truth: Canvas {
                    base_url: canvas.to_string(),
                },
                viewer: Viewer::User(user_id.parse().into_diagnostic()?),
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

            let resp: Response = resp
                .into_diagnostic()
                .wrap_err("failed recieving message from Ebauche?")?
                .map_err(|msg| miette!(msg).wrap_err("Ebauche response was error"))?;

            if let Response::Fetch(resp) = resp {
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
