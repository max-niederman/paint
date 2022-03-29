use crate::{TryFlatMap, TryFlatMapSelect};
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

    #[inline]
    fn try_flat_map_select<U, F>(self, map: F) -> TryFlatMapSelect<Self, U, F>
    where
        U: TryStream<Error = Self::Error> + Unpin,
        F: FnMut(Self::Ok) -> U + Unpin,
    {
        TryFlatMapSelect::new(self, map)
    }
}
impl<S: TryStream> FallibleStreamExt for S {}
