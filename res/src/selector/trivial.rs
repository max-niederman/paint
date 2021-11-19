//! Trivial Selectors

use canvas_lms::resource::*;

use super::Selector;

struct All;
impl<R: Resource> Selector<R> for All {
    fn matches(&self, resource: &R) -> bool {
        true
    }
}

struct None;
impl<R: Resource> Selector<R> for None {
    fn matches(&self, resource: &R) -> bool {
        false
    }
}

struct Id(canvas_lms::Id);
macro_rules! id_selector {
    ($res:ty) => {
        impl Selector<$res> for Id {
            fn matches(&self, resource: &$res) -> bool {
                resource.id == self.0
            }
        }
    };
}
id_selector!(Assignment);
id_selector!(Course);
id_selector!(Enrollment);
id_selector!(GradingPeriod);
id_selector!(User);

struct Not<A>(A);
impl<R: Resource, A: Selector<R>> Selector<R> for Not<A> {
    fn matches(&self, resource: &R) -> bool {
        !self.0.matches(resource)
    }
}

struct Or<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for Or<A, B> {
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) || self.1.matches(resource)
    }
}

struct And<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for And<A, B> {
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) && self.1.matches(resource)
    }
}

struct ExclusiveOr<A, B>(A, B);
impl<R: Resource, A: Selector<R>, B: Selector<R>> Selector<R> for ExclusiveOr<A, B> {
    fn matches(&self, resource: &R) -> bool {
        self.0.matches(resource) ^ self.1.matches(resource)
    }
}
