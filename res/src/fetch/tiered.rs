use std::marker::PhantomData;

use super::*;
use fallible_stream::{FallibleStreamExt, TryFlatMapSelect};

pub struct TieredFetcher<'f, F>(pub &'f F);

impl<'f, F, T, D> Fetch<T> for TieredFetcher<'f, F>
where
    F: Fetch<T, Dependency = D>,
    F: Fetch<D>,
    F: Unpin + Clone,
    D: Unpin,
    T: Unpin,
{
    type Dependency = <F as Fetch<D>>::Dependency;

    type FetchStream = TryFlatMapSelect<
        <F as Fetch<D>>::FetchStream,
        Pin<Box<<F as Fetch<T>>::FetchStream>>,
        TierMap<F, T, D>,
    >;
    fn fetch(&self, dependency: &Self::Dependency) -> Self::FetchStream {
        <F as Fetch<D>>::fetch(self.0, dependency).try_flat_map_select(TierMap::new(self.0.clone()))
    }
}

// it would be _really_ nice if `<F as Fetch<T>>::fetch` was a type which
// implemented `Fn`, but alas, it is not, so we have to do this.
// maybe I should write an RFC

pub struct TierMap<F, T, D> {
    fetcher: F,
    _dependant: PhantomData<T>,
    _dependency: PhantomData<D>,
}

impl<F, T, D> TierMap<F, T, D> {
    const fn new(fetcher: F) -> Self {
        Self {
            fetcher,
            _dependant: PhantomData,
            _dependency: PhantomData,
        }
    }
}

impl<F, T, D> FnOnce<(D,)> for TierMap<F, T, D>
where
    F: Fetch<T, Dependency = D>,
{
    type Output = Pin<Box<<F as Fetch<T>>::FetchStream>>;

    extern "rust-call" fn call_once(self, args: (D,)) -> Self::Output {
        Box::pin(self.fetcher.fetch(&args.0))
    }
}

impl<F, T, D> FnMut<(D,)> for TierMap<F, T, D>
where
    F: Fetch<T, Dependency = D>,
{
    extern "rust-call" fn call_mut(&mut self, args: (D,)) -> Self::Output {
        Box::pin(self.fetcher.fetch(&args.0))
    }
}

impl<F, T, D> Fn<(D,)> for TierMap<F, T, D>
where
    F: Fetch<T, Dependency = D>,
{
    extern "rust-call" fn call(&self, args: (D,)) -> Self::Output {
        Box::pin(self.fetcher.fetch(&args.0))
    }
}
