//! Here we define the finite set of Canvas resources used throughout Paint.
//! We have the [`canvas::Resource`] trait, but that is unwieldy for distributed systems like Paint.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A kind of resource.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResourceKind {
    Assignment,
    Course,
}

impl ResourceKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ResourceKind::Assignment => "assignment",
            ResourceKind::Course => "course",
        }
    }
}

impl FromStr for ResourceKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "assignment" => Ok(Self::Assignment),
            "course" => Ok(Self::Course),
            _ => Err("no such resource kind"),
        }
    }
}
