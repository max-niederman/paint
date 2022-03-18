use crate::{view::DbView, Error, Result};
use serde::{Deserialize, Serialize};

pub mod assignment;
pub mod course;

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
struct DbResource<R> {
    view: bson::Uuid,
    inserted_at: bson::DateTime,
    resource: R,
}

// TODO: can we refactor this into a struct implementing `FromRequest` perhaps?
async fn get_view(
    views: &mongodb::Collection<DbView>,
    view_id: bson::Uuid,
) -> Result<Option<DbView>> {
    views
        .find_one(bson::doc! { "_id": view_id }, None)
        .await
        .map_err(|err| Error::database_while("fetching view information", err))
}

type HttpClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

macro_rules! composite_api {
    ($( $api:ty ),* $(,)?) => {
        // NOTE: we can remove the unit once poem-rs/poem#232 is merged
        type Api = ( $($api),*, );

        pub fn make_api(database: &mongodb::Database, db_client: &mongodb::Client, http: &HttpClient) -> Api {
            ( $( <$api>::new(database, db_client, http.clone()) ),*, )
        }
    };
}

composite_api!(course::Api, assignment::Api);
