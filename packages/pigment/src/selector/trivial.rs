//! Trivial Selectors

use super::Selector;

use canvas::resource::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct All;
impl<R: Resource> Selector<R> for All {
    #[inline]
    fn matches(&self, _resource: &R) -> bool {
        true
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct None;
impl<R: Resource> Selector<R> for None {
    #[inline]
    fn matches(&self, _resource: &R) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Id(canvas::Id);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ids(Vec<canvas::Id>);

macro_rules! id_selector {
    ($res:ty) => {
        impl Selector<$res> for Id {
            #[inline]
            fn matches(&self, resource: &$res) -> bool {
                resource.id == self.0
            }
        }
        impl Selector<$res> for Ids {
            #[inline]
            fn matches(&self, resource: &$res) -> bool {
                self.0.iter().any(|id| id == &resource.id)
            }
        }
    };
}

id_selector!(Assignment);
id_selector!(Course);
id_selector!(GradingPeriod);
id_selector!(User);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Not<A>(A);
impl<R: Resource, A: Selector<R>> Selector<R> for Not<A> {
    #[inline]
    fn matches(&self, resource: &R) -> bool {
        !self.0.matches(resource)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Or<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for Or<A, B> {
    #[inline]
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) || self.1.matches(resource)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct And<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for And<A, B> {
    #[inline]
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) && self.1.matches(resource)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ExclusiveOr<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for ExclusiveOr<A, B> {
    #[inline]
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) ^ self.1.matches(resource)
    }
}
