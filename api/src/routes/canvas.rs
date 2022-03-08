use poem::{Endpoint, Request, Response};

pub struct CanvasEndpoint<Conn> {
    http: hyper::Client<Conn>,
}

impl<Conn> CanvasEndpoint<Conn> {
    pub fn new(http: hyper::Client<Conn>) -> Self {
        Self { http }
    }
}

#[poem::async_trait]
impl<Conn> Endpoint for CanvasEndpoint<Conn>
where
    Conn: Send + Sync,
{
    type Output = Response;

    async fn call(&self, req: Request) -> poem::Result<Response> {
        todo!()
    }
}
