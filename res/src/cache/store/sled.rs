use std::{pin::Pin, task::Poll};

use super::super::Error;
use super::Store;
use futures::{Future, Stream};
use sled::{IVec, Tree};

impl Store for Tree {
    type ByteVec = IVec;

    // using [`future::Ready`] like this is particularly bad,
    // because consumers creating many futures don't expect
    // those calls to block for significant periods of time.
    type GetFut = SledFuture<Option<Self::ByteVec>>;
    fn get<K: AsRef<[u8]>>(&self, key: &K) -> Self::GetFut {
        SledFuture::new(|| self.get(key))
    }

    type InsertFut = SledFuture<Option<Self::ByteVec>>;
    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(&self, key: &K, value: V) -> Self::InsertFut {
        SledFuture::new(|| self.insert(key, value))
    }

    type RemoveFut = SledFuture<Option<Self::ByteVec>>;
    fn remove<K: AsRef<[u8]>>(&self, key: &K) -> Self::RemoveFut {
        SledFuture::new(|| self.remove(key))
    }

    type ScanRangeStream = SledStream;
    fn scan_range<K: AsRef<[u8]>, R: std::ops::RangeBounds<K>>(
        &self,
        range: R,
    ) -> Self::ScanRangeStream {
        SledStream::new(self.range(range))
    }

    type RemoveRangeFut = SledFuture<()>;
    fn remove_range<K: AsRef<[u8]>, R: std::ops::RangeBounds<K>>(
        &self,
        range: R,
    ) -> Self::RemoveRangeFut {
        SledFuture::new(|| {
            self.range(range)
                .try_for_each(|item| self.remove(&item?.0).map(|_| ()))
        })
    }

    type ScanPrefixStream = SledStream;
    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::ScanPrefixStream {
        SledStream::new(self.scan_prefix(prefix))
    }

    type RemovePrefixFut = SledFuture<()>;
    fn remove_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::RemovePrefixFut {
        SledFuture::new(|| {
            self.scan_prefix(prefix)
                .try_for_each(|item| self.remove(&item?.0).map(|_| ()))
        })
    }
}

// TODO: possible to rewrite these to prevent starving other tasks while sled is blocking?
//       my first thought was to use [`tokio::task::spawn_blocking`], but that requires
//       the passed closure to be `'static + Send`, which doesn't really make sense to add
//       to [`Store`].

pub struct SledFuture<T>(Option<Result<T, Error>>);
impl<T> SledFuture<T> {
    fn new<F: FnOnce() -> sled::Result<T>>(f: F) -> Self {
        SledFuture(Some(f().map_err(Error::Sled)))
    }
}
impl<T> Future for SledFuture<T>
where
    T: Unpin,
{
    type Output = Result<T, Error>;
    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context) -> std::task::Poll<Self::Output> {
        Poll::Ready(
            self.get_mut()
                .0
                .take()
                .expect("SledFuture polled after completion"),
        )
    }
}

pub struct SledStream(sled::Iter);
impl SledStream {
    fn new(iter: sled::Iter) -> Self {
        SledStream(iter)
    }
}
impl Stream for SledStream {
    type Item = Result<(IVec, IVec), Error>;
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context,
    ) -> std::task::Poll<Option<Self::Item>> {
        Poll::Ready(self.get_mut().0.next().map(|r| r.map_err(Error::Sled)))
    }
}
