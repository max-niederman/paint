//! Trivial Selectors

use canvas_lms::resource::*;

use super::Selector;

pub struct All;
impl<R: Resource> Selector<R> for All {
    fn matches(&self, _resource: &R) -> bool {
        true
    }
}

pub struct None;
impl<R: Resource> Selector<R> for None {
    fn matches(&self, _resource: &R) -> bool {
        false
    }
}

pub struct Id(canvas_lms::Id);
pub struct Ids(Vec<canvas_lms::Id>);
macro_rules! id_selector {
    ($res:ty) => {
        impl Selector<$res> for Id {
            fn matches(&self, resource: &$res) -> bool {
                resource.id == self.0
            }
        }
        impl Selector<$res> for Ids {
            fn matches(&self, resource: &$res) -> bool {
                self.0.iter().any(|id| id == &resource.id)
            }
        }
    };
}
id_selector!(Assignment);
id_selector!(Course);
id_selector!(Enrollment);
id_selector!(GradingPeriod);
id_selector!(User);

pub struct Not<A>(A);
impl<R: Resource, A: Selector<R>> Selector<R> for Not<A> {
    fn matches(&self, resource: &R) -> bool {
        !self.0.matches(resource)
    }
}

pub struct Or<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for Or<A, B> {
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) || self.1.matches(resource)
    }
}

pub struct And<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for And<A, B> {
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) && self.1.matches(resource)
    }
}

pub struct ExclusiveOr<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for ExclusiveOr<A, B> {
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) ^ self.1.matches(resource)
    }
}
