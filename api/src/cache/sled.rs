use super::{CacheEntry, Key, Cache, Store};
use poem::{error::InternalServerError, Result};
use std::marker::PhantomData;
use uuid::Uuid;

#[derive(Debug)]
pub struct SledStore<R> {
    tree: sled::Tree,
    _resource: PhantomData<R>,
}

impl<R> Clone for SledStore<R> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            _resource: PhantomData,
        }
    }
}

impl<R: Cache> Store for SledStore<R> {
    type Resource = R;

    fn get(
        &self,
        view: Uuid,
        key: &<Self::Resource as Cache>::Key,
    ) -> Result<Option<CacheEntry<Self::Resource>>> {
        match self.tree.get(
            [
                view.key_serialize()?.as_slice(),
                key.key_serialize()?.as_slice(),
            ]
            .concat(),
        ) {
            Ok(Some(bytes)) => Ok(Some(
                bincode::deserialize(&bytes).map_err(InternalServerError)?,
            )),
            Ok(None) => Ok(None),
            Err(err) => Err(InternalServerError(err)),
        }
    }

    fn insert(
        &self,
        view: Uuid,
        entry: &CacheEntry<Self::Resource>,
    ) -> Result<Option<CacheEntry<Self::Resource>>> {
        match self.tree.insert(
            &[
                view.key_serialize()?.as_slice(),
                entry.resource.key().key_serialize()?.as_slice(),
            ]
            .concat(),
            bincode::serialize(&entry).map_err(InternalServerError)?,
        ) {
            Ok(Some(bytes)) => Ok(Some(
                bincode::deserialize(&bytes).map_err(InternalServerError)?,
            )),
            Ok(None) => Ok(None),
            Err(err) => Err(InternalServerError(err)),
        }
    }
}
