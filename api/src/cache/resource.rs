use super::*;
use canvas::resource::*;

impl Cache for Course {
    type Key = canvas::Id;
    fn key(&self) -> Self::Key {
        self.id
    }
}