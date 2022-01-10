use pigment::cache::*;
use sled::{IVec, Tree};
use std::ops::{Deref, DerefMut, RangeBounds};

pub struct SledStore {
    tree: Tree,
}

impl From<Tree> for SledStore {
    fn from(tree: Tree) -> Self {
        Self { tree }
    }
}

impl Deref for SledStore {
    type Target = Tree;
    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}
impl DerefMut for SledStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

impl Store for SledStore {
    type ByteVec = IVec;
    type ByteVecBorrowed<'s> = IVec;

    fn get<K: AsRef<[u8]>>(&self, key: &K) -> Result<Option<Self::ByteVecBorrowed<'_>>> {
        self.tree.get(key).map_err(Error::store)
    }

    fn insert<K: AsRef<[u8]>, V: Into<Self::ByteVec>>(
        &self,
        key: &K,
        value: V,
    ) -> Result<Option<Self::ByteVec>> {
        self.tree.insert(key, value).map_err(Error::store)
    }

    fn remove<K: AsRef<[u8]>>(&self, key: &K) -> Result<Option<Self::ByteVec>> {
        self.tree.remove(key).map_err(Error::store)
    }

    type ScanRangeIter<'s> = SledIter;
    fn scan_range<'s, K, R>(&'s self, range: R) -> Self::ScanRangeIter<'_>
    where
        K: AsRef<[u8]> + 's,
        R: RangeBounds<K> + 's,
    {
        SledIter::new(self.tree.range(range))
    }

    type ScanPrefixIter<'s> = SledIter;
    fn scan_prefix<P: AsRef<[u8]>>(&self, prefix: &P) -> Self::ScanPrefixIter<'_> {
        SledIter::new(self.tree.scan_prefix(prefix))
    }
}

pub struct SledIter(sled::Iter);
impl SledIter {
    fn new(iter: sled::Iter) -> Self {
        SledIter(iter)
    }
}
impl Iterator for SledIter {
    type Item = Result<(IVec, IVec), Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|r| r.map_err(Error::store))
    }
}
