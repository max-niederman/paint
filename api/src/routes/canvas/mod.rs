use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::view::DbView;

pub mod course;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct DbResource<R> {
    view: Uuid,
    inserted_at: DateTime<Utc>,
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

type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

macro_rules! composite_api {
    ($( $api:ty ),* $(,)?) => {
        type Api = ( $($api),*, );

        pub fn make_api(database: &mongodb::Database, http: &HttpClient) -> Api {
            ( $( <$api>::new(database, http.clone()) ),*, )
        }
    };
}

composite_api!(course::Api);
