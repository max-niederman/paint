//! Views into Canvas instances.
//!
//! This module contains the [`View`] type, which is the basic property by which resources are organized by Pigment.
//! A [`View`] consists of the `truth`, or the Canvas instance being viewed, and the `viewer`, which represents the
//! way the view differs from the truth. The viewer can currently only be a [`Viewer::User`], but in the future, there
//! could be other types of viewers.

use serde::{Deserialize, Serialize};

// TODO: refactor item/member visibility

/// A view into a Canvas instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct View {
    pub truth: Canvas,
    pub viewer: Viewer,
}

/// A Canvas instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Canvas {
    pub base_url: String,
}

/// A viewer into a Canvas instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Viewer {
    User(canvas::Id),
}
