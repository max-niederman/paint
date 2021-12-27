use super::prelude::*;
use resource::assignment::*;

impl Cache for Assignment {
    type Key = canvas::Id;

    #[inline]
    fn key(&self) -> Self::Key {
        self.id
    }
}
