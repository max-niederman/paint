use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::{btree_map, BTreeMap},
    ops::RangeBounds,
};

use js_sys::Iter;
use pigment::cache::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GlazeStore {
    resources: RefCell<BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl Store for GlazeStore {
    type ByteVec = Vec<u8>;
    type ByteVecBorrowed<'s> = &'s [u8];

    fn get<K: AsRef<[u8]>>(&self, key: &K) -> Result<Option<Self::ByteVecBorrowed<'_>>> {
        Ok(self.resources.borrow().get(key.as_ref()).map(Vec::as_slice))
    }

    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(
        &self,
        key: &K,
        value: V,
    ) -> Result<Option<Self::ByteVec>> {
        Ok(self
            .resources
            .borrow_mut()
            .insert(key.as_ref().to_vec(), value.into()))
    }

    fn remove<K: AsRef<[u8]>>(&self, key: &K) -> Result<Option<Self::ByteVec>> {
        Ok(self.resources.borrow_mut().remove(key.as_ref()))
    }

    type ScanRangeIter<'s> = GlazeIter<'s>;
    fn scan_range<K: AsRef<[u8]>, R: std::ops::RangeBounds<K>>(
        &self,
        range: R,
    ) -> Self::ScanRangeIter<'_> {
        self.resources.borrow().range::<[u8], _>((
            range.start_bound().map(AsRef::as_ref),
            range.end_bound().map(AsRef::as_ref),
        )).into()
    }
    fn remove_range<K: AsRef<[u8]>, R: std::ops::RangeBounds<K>>(&self, range: R) -> Result<()> {
        let range = (
            range.start_bound().map(AsRef::as_ref),
            range.end_bound().map(AsRef::as_ref),
        );
        self.resources
            .borrow_mut()
            .drain_filter(|k, _| RangeBounds::<[u8]>::contains(&range, k.as_slice()));
        Ok(())
    }

    type ScanPrefixIter<'s> = GlazeIter<'s>;
    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::ScanPrefixIter<'_> {
        todo!()
    }
    fn remove_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Result<()> {
        todo!()
    }
}

pub(crate) struct GlazeIter<'s>(btree_map::Range<'s, Vec<u8>, Vec<u8>>);
impl<'s> From<btree_map::Range<'s, Vec<u8>, Vec<u8>>> for GlazeIter<'s> {
    fn from(iter: btree_map::Range<'s, Vec<u8>, Vec<u8>>) -> Self {
        Self(iter)
    }
}
impl<'s> Iterator for GlazeIter<'s> {
    type Item = Result<(&'s [u8], &'s [u8])>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| Ok((k.as_slice(), v.as_slice())))
    }
}