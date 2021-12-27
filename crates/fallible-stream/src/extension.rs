use crate::TryFlatMap;
use futures::prelude::*;

pub trait FallibleStreamExt: TryStream + Sized {
    #[inline]
    fn try_flat_map<U, F>(self, map: F) -> TryFlatMap<Self, U, F>
    where
        U: TryStream<Error = Self::Error>,
        F: FnMut(Self::Ok) -> U + Unpin,
    {
        TryFlatMap::new(self, map)
    }
}
impl<S: TryStream> FallibleStreamExt for S {}