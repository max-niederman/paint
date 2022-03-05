mod prelude {
    pub use crate::{
        impl_collection_fetch,
        res::{CacheLocation, Collection, Fetch, Resource},
        view::View,
    };
    pub use canvas_lms::client::{self, hyper, pagination};
    pub use fallible_stream::YieldError;
    pub use futures::prelude::*;
    pub use std::pin::Pin;
}

pub mod course;

#[macro_export]
macro_rules! impl_collection_fetch {
    ($ty:ty, single, $path_gen:expr) => {
        impl<'f, Conn> Fetch<'f, hyper::Client<Conn>> for $ty
        where
            Conn: hyper::client::connect::Connect + Clone + Unpin + Send + Sync + 'static,
        {
            type Err = canvas_lms::client::Error;

            type FetchAllStream =
                stream::Once<Pin<Box<dyn Future<Output = Result<Self::Resource, Self::Err>> + Send >>>;
            fn fetch_all(
                &'f self,
                view: &'f View,
                http: hyper::Client<Conn>,
            ) -> Self::FetchAllStream {
                let path_gen: fn(&'f Self, &'f View) -> String = $path_gen;
                let mut path = path_gen(self, view);

                // append resource query parameters
                path.push('?');
                path.push_str(Self::Resource::query_string());

                let client = view.client(http);
                stream::once(
                    async move {
                        client
                            .request(hyper::Method::GET, path)
                            .send()
                            .and_then(client::Response::deserialize)
                            .await
                    }
                    .boxed(),
                )
            }
        }
    };
    ($ty:ty, paginated, $path_gen:expr) => {
        impl<'f, Conn> Fetch<'f, hyper::Client<Conn>> for $ty
        where
            Conn: hyper::client::connect::Connect + Clone + Unpin + Send + Sync + 'static,
        {
            type Err = canvas_lms::client::Error;

            type FetchAllStream = YieldError<pagination::Items<'f, Conn, Self::Resource>>;
            fn fetch_all(
                &'f self,
                view: &'f View,
                http: hyper::Client<Conn>,
            ) -> Self::FetchAllStream {
                let path_gen: fn(&'f Self, &'f View) -> String = $path_gen;
                let path = path_gen(self, view);

                YieldError::Ok(
                    view.client(http)
                        .request(hyper::Method::GET, path)
                        .paginate_owned(100)?
                        .items(),
                )
            }
        }
    };
}
