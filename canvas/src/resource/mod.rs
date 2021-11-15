use crate::Id;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Resource: DeserializeOwned + Serialize {
    fn id(&self) -> Id;
}

macro_rules! resource_modules {
    ($($mod:ident :: $ty:ident),*,) => {
        $(
            pub mod $mod;
            pub use $mod::$ty;
        )*
    };
}

macro_rules! gen_resource_enum {
    ($($ty:ident),*,) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum Res {
            $( $ty($ty) ),*
        }

        impl Resource for Res {
            fn id(&self) -> u64 {
                match self {
                    $( Self::$ty(r) => r.id(), )*
                }
            }
        }
    };
}

resource_modules! {
    announcement::Announcement,
    assignment::Assignment,
}

gen_resource_enum! {
    Announcement,
    Assignment,
}

pub mod shared;
