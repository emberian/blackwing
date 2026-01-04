//! Tile definitions and tilesets.

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Unique tile type identifier within a tileset.
pub type TileId = u16;

/// Cardinal direction for movement and exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Get the opposite direction.
    pub fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Get the unit vector for this direction.
    pub fn as_delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

/// How a tile interacts with collision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionType {
    /// Can walk through freely.
    #[default]
    Walkable,
    /// Solid wall/obstacle - blocks movement.
    Solid,
    /// Water - requires item to cross (e.g., raft, flippers).
    Water,
    /// Pit/hole - may damage or block.
    Pit,
    /// Trigger zone - activates a script when entered.
    Trigger,
    /// One-way passage - only allows movement in specified direction.
    OneWay(Direction),
}

impl CollisionType {
    /// Check if this collision type blocks movement.
    pub fn is_blocking(&self) -> bool {
        matches!(self, CollisionType::Solid | CollisionType::Water | CollisionType::Pit)
    }
}

/// Animation data for animated tiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileAnimation {
    /// Texture indices for each frame.
    pub frames: Vec<u16>,
    /// Duration of each frame in milliseconds.
    pub frame_duration_ms: u32,
}

impl TileAnimation {
    /// Get the frame index for a given time in milliseconds.
    pub fn frame_at(&self, time_ms: u64) -> u16 {
        if self.frames.is_empty() || self.frame_duration_ms == 0 {
            return 0;
        }
        let cycle_duration = self.frame_duration_ms as u64 * self.frames.len() as u64;
        let time_in_cycle = time_ms % cycle_duration;
        let frame_index = (time_in_cycle / self.frame_duration_ms as u64) as usize;
        self.frames[frame_index]
    }
}

/// A tile type definition within a tileset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileDef {
    /// Unique ID within this tileset.
    pub id: TileId,
    /// Human-readable name.
    pub name: SmolStr,
    /// How this tile interacts with collision.
    #[serde(default)]
    pub collision: CollisionType,
    /// Index into the tileset texture (row-major).
    pub texture_index: u16,
    /// Animation data if this tile is animated.
    #[serde(default)]
    pub animation: Option<TileAnimation>,
    /// Tags for scripting queries (e.g., "ground", "water", "hazard").
    #[serde(default)]
    pub tags: Vec<SmolStr>,
}

impl TileDef {
    /// Create a simple walkable tile.
    pub fn walkable(id: TileId, name: impl Into<SmolStr>, texture_index: u16) -> Self {
        Self {
            id,
            name: name.into(),
            collision: CollisionType::Walkable,
            texture_index,
            animation: None,
            tags: Vec::new(),
        }
    }

    /// Create a solid blocking tile.
    pub fn solid(id: TileId, name: impl Into<SmolStr>, texture_index: u16) -> Self {
        Self {
            id,
            name: name.into(),
            collision: CollisionType::Solid,
            texture_index,
            animation: None,
            tags: Vec::new(),
        }
    }

    /// Get the texture index, accounting for animation.
    pub fn texture_index_at(&self, time_ms: u64) -> u16 {
        match &self.animation {
            Some(anim) => anim.frame_at(time_ms),
            None => self.texture_index,
        }
    }

    /// Check if this tile has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// A tileset containing tile definitions and texture reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSet {
    /// Unique identifier for this tileset.
    pub id: SmolStr,
    /// Human-readable name.
    pub name: String,
    /// Path to tileset image (relative to assets/).
    pub texture_path: String,
    /// Tile width in pixels.
    pub tile_width: u16,
    /// Tile height in pixels.
    pub tile_height: u16,
    /// Number of tiles per row in the texture.
    pub tiles_per_row: u16,
    /// Tile definitions indexed by TileId.
    pub tiles: Vec<TileDef>,
}

impl TileSet {
    /// Look up a tile definition by ID.
    pub fn get_tile(&self, id: TileId) -> Option<&TileDef> {
        self.tiles.iter().find(|t| t.id == id)
    }

    /// Get the UV coordinates for a texture index.
    pub fn tile_uv(&self, texture_index: u16) -> (f32, f32, f32, f32) {
        let col = texture_index % self.tiles_per_row;
        let row = texture_index / self.tiles_per_row;
        let u = col as f32 * self.tile_width as f32;
        let v = row as f32 * self.tile_height as f32;
        (u, v, self.tile_width as f32, self.tile_height as f32)
    }

    /// Get collision type for a tile ID.
    pub fn collision_for(&self, id: TileId) -> CollisionType {
        self.get_tile(id)
            .map(|t| t.collision)
            .unwrap_or(CollisionType::Solid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
    }

    #[test]
    fn tile_animation_frame() {
        let anim = TileAnimation {
            frames: vec![0, 1, 2, 1],
            frame_duration_ms: 100,
        };
        assert_eq!(anim.frame_at(0), 0);
        assert_eq!(anim.frame_at(100), 1);
        assert_eq!(anim.frame_at(200), 2);
        assert_eq!(anim.frame_at(300), 1);
        assert_eq!(anim.frame_at(400), 0); // Wraps around
    }

    #[test]
    fn collision_blocking() {
        assert!(!CollisionType::Walkable.is_blocking());
        assert!(CollisionType::Solid.is_blocking());
        assert!(CollisionType::Water.is_blocking());
        assert!(!CollisionType::Trigger.is_blocking());
    }
}
