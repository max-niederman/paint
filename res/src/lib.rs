pub mod resource;

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[tarpc::service]
pub trait ResourceCache {
    async fn get(selector: ResourceSelector) -> Box<dyn tokio_stream::Stream<Item = Resource>>;
}