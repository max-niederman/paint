use super::prelude::*;
use resource::course::*;

impl Cache for Course {
    type Key = canvas::Id;
    fn key(&self) -> Self::Key {
        self.id
    }
}
