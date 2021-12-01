use super::*;

use canvas::resource;
use serde::{Deserialize, Serialize};

/// A discriminated resource selector which may be serialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DSelector {
    // trivial
    All(All),
    None(None),
    Id(Id),
    Ids(Ids),
    Not(Box<Not<DSelector>>),
    Or(Box<Or<DSelector, DSelector>>),
    And(Box<And<DSelector, DSelector>>),
    ExclusiveOr(Box<ExclusiveOr<DSelector, DSelector>>),
}

macro_rules! impl_selector {
    (
        resource = $resource:ty;
        special = [ $( $var:ident, )* ];
    ) => {
        impl Selector<$resource> for DSelector {
            fn matches(&self, resource: &$resource) -> bool {
                #![allow(unreachable_patterns)]
                match self {
                    // these variants are always implemented
                    DSelector::All(sel) => sel.matches(resource),
                    DSelector::None(sel) => sel.matches(resource),
                    DSelector::Not(sel) => sel.matches(resource),
                    DSelector::Or(sel) => sel.matches(resource),
                    DSelector::And(sel) => sel.matches(resource),
                    DSelector::ExclusiveOr(sel) => sel.matches(resource),

                    // extra variants which are implemented on only some resources
                    $( DSelector::$var(sel) => sel.matches(resource), )*

                    // variants which cannot be implemented return false
                    _ => false,
                }
            }
        }
    }
}

impl_selector! {
    resource = resource::Assignment;
    special = [
        Id,
        Ids,
    ];
}

impl_selector! {
    resource = resource::Course;
    special = [
        Id,
        Ids,
    ];
}

impl_selector! {
    resource = resource::Submission;
    special = [];
}

impl_selector! {
    resource = resource::User;
    special = [
        Id,
        Ids,
    ];
}
