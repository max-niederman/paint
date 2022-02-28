use super::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct Courses;

impl Node for Courses {}
impl<'r> HomoNode<'r> for Courses {
    type Resource = Course;
}
impl<'r, Conn> FetchAll<'r, Client<Conn>> for Courses
where
    Conn: Connect + Unpin + Send + Sync + Clone + 'static,
{
    type Err = CanvasError;

    type FetchAllStream = YieldError<CanvasItemStream<Items<'r, Conn, Self::Resource>>>;
    fn fetch_all(&'r self, client: &'r Client<Conn>) -> Self::FetchAllStream {
        YieldError::Ok(
            client
                .request(Method::GET, "/api/v1/courses")
                .paginate(50)
                .map_err(CanvasError)?
                .items::<Self::Resource>()
                .into(),
        )
    }
}