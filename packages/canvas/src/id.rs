use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// A numerical Canvas ID.
#[cfg_attr(
    feature = "typescript-definitions",
    derive(typescript_definitions::TypeScriptify)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id(u64);

impl Id {
    /// Creates a new ID from a numerical value.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Creates an ID from its big-endian byte representation.
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(bytes))
    }

    /// Creates an ID from its little-endian byte representation.
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Self(u64::from_le_bytes(bytes))
    }

    /// Return the big-endian byte representation of this ID.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    /// Return the big-endian byte representation of this ID.
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

impl From<u64> for Id {
    #[inline]
    fn from(int: u64) -> Self {
        Self(int)
    }
}

impl From<Id> for u64 {
    #[inline]
    fn from(id: Id) -> Self {
        id.0
    }
}

impl fmt::Display for Id {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Id {
    type Err = <u64 as FromStr>::Err;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}
