//! Trivial Selectors

use canvas_lms::Resource;

use super::Selector;

struct All;
impl<R: Resource> Selector<R> for All {
    fn matches(&self, resource: &R) -> bool {
        true
    }
}

struct None;
impl<R: Resource> Selector<R> for All {
    fn matches(&self, resource: &R) -> bool {
        false
    }
}

struct Id(canvas_lms::Id);
impl<R: Resource> Resource for Id {
    fn matches(&self, resource: &R) -> bool {
        resource.id() == self.0
    }
}

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
