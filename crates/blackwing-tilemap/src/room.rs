//! Room/screen structure for tile-based worlds.

use crate::{Direction, TileId};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Default room dimensions (NES Zelda: 16x11).
pub const DEFAULT_ROOM_WIDTH: usize = 16;
pub const DEFAULT_ROOM_HEIGHT: usize = 11;

/// A spawn point for an entity within a room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPoint {
    /// X position in tiles.
    pub x: u16,
    /// Y position in tiles.
    pub y: u16,
    /// Entity template ID to spawn (e.g., "enemy:octorok").
    pub template_id: SmolStr,
    /// Optional condition (Rhai expression) for spawning.
    #[serde(default)]
    pub condition: Option<SmolStr>,
    /// Tags to add to spawned entity.
    #[serde(default)]
    pub tags: Vec<SmolStr>,
}

impl SpawnPoint {
    /// Create a spawn point at the given tile position.
    pub fn new(x: u16, y: u16, template_id: impl Into<SmolStr>) -> Self {
        Self {
            x,
            y,
            template_id: template_id.into(),
            condition: None,
            tags: Vec::new(),
        }
    }

    /// Add a spawn condition.
    pub fn with_condition(mut self, condition: impl Into<SmolStr>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Get pixel position (assuming 16x16 tiles).
    pub fn pixel_position(&self, tile_size: u16) -> (f32, f32) {
        (
            self.x as f32 * tile_size as f32 + tile_size as f32 / 2.0,
            self.y as f32 * tile_size as f32 + tile_size as f32 / 2.0,
        )
    }
}

/// An exit from a room leading to another room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomExit {
    /// Tile positions that trigger this exit.
    pub tiles: Vec<(u16, u16)>,
    /// Direction player moves to trigger exit.
    pub direction: Direction,
    /// Target room ID.
    pub target_room: SmolStr,
    /// Target X position in pixels.
    pub target_x: f32,
    /// Target Y position in pixels.
    pub target_y: f32,
}

impl RoomExit {
    /// Create an exit at a single tile position.
    pub fn new(
        x: u16,
        y: u16,
        direction: Direction,
        target_room: impl Into<SmolStr>,
        target_x: f32,
        target_y: f32,
    ) -> Self {
        Self {
            tiles: vec![(x, y)],
            direction,
            target_room: target_room.into(),
            target_x,
            target_y,
        }
    }

    /// Create an exit spanning multiple tiles (e.g., a wide doorway).
    pub fn spanning(
        tiles: Vec<(u16, u16)>,
        direction: Direction,
        target_room: impl Into<SmolStr>,
        target_x: f32,
        target_y: f32,
    ) -> Self {
        Self {
            tiles,
            direction,
            target_room: target_room.into(),
            target_x,
            target_y,
        }
    }

    /// Check if a tile position is part of this exit.
    pub fn contains_tile(&self, x: u16, y: u16) -> bool {
        self.tiles.contains(&(x, y))
    }
}

/// A room/screen in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Unique room identifier.
    pub id: SmolStr,
    /// Human-readable name.
    pub name: String,

    /// Tile data (row-major, height x width).
    pub tiles: Vec<TileId>,
    /// Room width in tiles.
    pub width: usize,
    /// Room height in tiles.
    pub height: usize,

    /// Which tileset this room uses.
    pub tileset_id: SmolStr,

    /// Entity spawn points in this room.
    #[serde(default)]
    pub spawn_points: Vec<SpawnPoint>,

    /// Exit/entrance points.
    #[serde(default)]
    pub exits: Vec<RoomExit>,

    /// Script to run when entering this room.
    #[serde(default)]
    pub on_enter: Option<SmolStr>,

    /// Script to run when leaving this room.
    #[serde(default)]
    pub on_exit: Option<SmolStr>,

    /// Room-specific properties (key-value metadata).
    #[serde(default)]
    pub properties: IndexMap<SmolStr, SmolStr>,
}

impl Room {
    /// Create a new empty room with default dimensions.
    pub fn new(id: impl Into<SmolStr>, tileset_id: impl Into<SmolStr>) -> Self {
        let width = DEFAULT_ROOM_WIDTH;
        let height = DEFAULT_ROOM_HEIGHT;
        Self {
            id: id.into(),
            name: String::new(),
            tiles: vec![0; width * height],
            width,
            height,
            tileset_id: tileset_id.into(),
            spawn_points: Vec::new(),
            exits: Vec::new(),
            on_enter: None,
            on_exit: None,
            properties: IndexMap::new(),
        }
    }

    /// Create a room with custom dimensions.
    pub fn with_size(
        id: impl Into<SmolStr>,
        tileset_id: impl Into<SmolStr>,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            tiles: vec![0; width * height],
            width,
            height,
            tileset_id: tileset_id.into(),
            spawn_points: Vec::new(),
            exits: Vec::new(),
            on_enter: None,
            on_exit: None,
            properties: IndexMap::new(),
        }
    }

    /// Get the tile at a position (in tiles).
    pub fn tile_at(&self, x: usize, y: usize) -> Option<TileId> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.tiles[y * self.width + x])
    }

    /// Set the tile at a position.
    pub fn set_tile(&mut self, x: usize, y: usize, tile: TileId) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    /// Get the tile at a pixel position.
    pub fn tile_at_pixel(&self, px: f32, py: f32, tile_size: u16) -> Option<TileId> {
        let x = (px / tile_size as f32) as usize;
        let y = (py / tile_size as f32) as usize;
        self.tile_at(x, y)
    }

    /// Get room dimensions in pixels.
    pub fn pixel_size(&self, tile_size: u16) -> (f32, f32) {
        (
            self.width as f32 * tile_size as f32,
            self.height as f32 * tile_size as f32,
        )
    }

    /// Fill a rectangular area with a tile.
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, tile: TileId) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_tile(x + dx, y + dy, tile);
            }
        }
    }

    /// Fill the border of the room with a tile.
    pub fn fill_border(&mut self, tile: TileId) {
        // Top and bottom
        for x in 0..self.width {
            self.set_tile(x, 0, tile);
            self.set_tile(x, self.height - 1, tile);
        }
        // Left and right
        for y in 1..self.height - 1 {
            self.set_tile(0, y, tile);
            self.set_tile(self.width - 1, y, tile);
        }
    }

    /// Check if a pixel position is within room bounds.
    pub fn contains_pixel(&self, px: f32, py: f32, tile_size: u16) -> bool {
        let (w, h) = self.pixel_size(tile_size);
        px >= 0.0 && px < w && py >= 0.0 && py < h
    }

    /// Find an exit at the given tile position and direction.
    pub fn exit_at(&self, x: u16, y: u16, direction: Direction) -> Option<&RoomExit> {
        self.exits
            .iter()
            .find(|e| e.direction == direction && e.contains_tile(x, y))
    }

    /// Get a property value.
    pub fn property(&self, key: &str) -> Option<&SmolStr> {
        self.properties.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_tile_access() {
        let mut room = Room::new("test", "tileset");

        // Default is 0
        assert_eq!(room.tile_at(0, 0), Some(0));

        // Set and get
        room.set_tile(5, 3, 42);
        assert_eq!(room.tile_at(5, 3), Some(42));

        // Out of bounds
        assert_eq!(room.tile_at(100, 100), None);
    }

    #[test]
    fn room_fill_border() {
        let mut room = Room::with_size("test", "tileset", 4, 4);
        room.fill_border(1);

        // Corners
        assert_eq!(room.tile_at(0, 0), Some(1));
        assert_eq!(room.tile_at(3, 0), Some(1));
        assert_eq!(room.tile_at(0, 3), Some(1));
        assert_eq!(room.tile_at(3, 3), Some(1));

        // Center should be unchanged
        assert_eq!(room.tile_at(1, 1), Some(0));
        assert_eq!(room.tile_at(2, 2), Some(0));
    }

    #[test]
    fn room_pixel_bounds() {
        let room = Room::with_size("test", "tileset", 16, 11);
        let tile_size = 16;

        assert!(room.contains_pixel(0.0, 0.0, tile_size));
        assert!(room.contains_pixel(255.0, 175.0, tile_size));
        assert!(!room.contains_pixel(256.0, 0.0, tile_size));
        assert!(!room.contains_pixel(0.0, 176.0, tile_size));
        assert!(!room.contains_pixel(-1.0, 0.0, tile_size));
    }
}
