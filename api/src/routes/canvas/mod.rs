use chrono::{DateTime, Utc};
use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::view::DbView;

pub mod course;

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
struct DbResource<R> {
    view: bson::Uuid,
    inserted_at: bson::DateTime,
    resource: R,
}

async fn get_view(
    views: &mongodb::Collection<DbView>,
    view_id: bson::Uuid,
) -> poem::Result<DbView> {
    views
        .find_one(bson::doc! { "_id": view_id }, None)
        .await
        .map_err(poem::error::InternalServerError)?
        .ok_or_else(|| poem::error::NotFoundError.into())
}

fn expiration_index(duration: Duration) -> mongodb::IndexModel {
    mongodb::IndexModel::builder()
        .keys(bson::doc! { "inserted_at": 1u32 })
        .options(
            mongodb::options::IndexOptions::builder()
                .name(Some("expiration".to_string()))
                .expire_after(Some(duration))
                .build(),
        )
        .build()
}

type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

macro_rules! composite_api {
    ($( $api:ty ),* $(,)?) => {
        // NOTE: we can remove the unit once poem-rs/poem#232 is merged
        type Api = ( $($api),*, () );

        pub fn make_api(database: &mongodb::Database, http: &HttpClient) -> Api {
            ( $( <$api>::new(database, http.clone()) ),*, () )
        }

        pub async fn init_database(database: &mongodb::Database) -> miette::Result<()> {
            $( <$api>::init_database(database).await?; )*
            Ok(())
        }
    };
}

composite_api!(course::Api);
