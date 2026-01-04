//! Tile-based world/map system for Zelda-style gameplay.
//!
//! This crate provides data structures for:
//! - Tile definitions and tilesets
//! - Room/screen layouts (16x11 tiles like NES Zelda)
//! - World maps (grids of interconnected rooms)
//! - Collision detection and movement resolution

mod tile;
mod room;
mod world;
mod collision;

pub use tile::*;
pub use room::*;
pub use world::*;
pub use collision::*;
