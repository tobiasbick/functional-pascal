//! Shared scroll model and scroll-bar geometry.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

mod geometry;
mod model;

#[cfg(test)]
mod tests;

pub use geometry::{
    ScrollBarHit, ScrollBarOrientation, ScrollBarThumb, drag_offset, hit_zone, thumb_geometry,
    track_cells,
};
pub use model::ScrollModel;
