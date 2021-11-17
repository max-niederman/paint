#[tarpc::service]
pub trait ResourceCache {
    async fn get_by_selector(selector: ResourceSelector) -> Vec<Resource>;
    async fn update_by_selector(selector: ResourceSelector) -> UpdateResponse;
}

pub struct UpdateResponse {
    pub checked: u64,
    pub updated: u64,

    pub canvas_time: f64,
    pub canvas_cost: f64,
}
