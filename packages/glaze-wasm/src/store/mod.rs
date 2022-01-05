use std::pin::Pin;

use indexed_db_futures::prelude::*;
use js_sys::Uint8Array;
use web_sys::DomException;
use pigment::cache::*;
use futures::prelude::*;

#[derive(Debug)]
pub struct IdbStore<'db> {
    name: &'db str,
    transaction: IdbTransaction<'db>,
}

impl<'db> Store for IdbStore<'db> {
    type ByteVec = Vec<u8>;

    type GetFut;
    fn get<K: AsRef<[u8]>>(&self, key: &K) -> Self::GetFut {
        todo!()
    }

    type InsertFut;

    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(&self, key: &K, value: V) -> Self::InsertFut {
        todo!()
    }

    type RemoveFut;

    fn remove<K: AsRef<[u8]>>(&self, key: &K) -> Self::RemoveFut {
        todo!()
    }

    type ScanRangeStream;

    fn scan_range<K: AsRef<[u8]>, R: std::ops::RangeBounds<K>>(&self, range: R) -> Self::ScanRangeStream {
        todo!()
    }

    type RemoveRangeFut;

    fn remove_range<K: AsRef<[u8]>, R: std::ops::RangeBounds<K>>(&self, range: R) -> Self::RemoveRangeFut {
        todo!()
    }

    type ScanPrefixStream;

    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::ScanPrefixStream {
        todo!()
    }

    type RemovePrefixFut;

    fn remove_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::RemovePrefixFut {
        todo!()
    }
}