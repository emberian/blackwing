//! World structure - a grid of interconnected rooms.

use crate::{Direction, Room};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A world map containing a grid of rooms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMap {
    /// Unique world identifier.
    pub id: SmolStr,
    /// Human-readable name.
    pub name: String,

    /// Grid of room IDs (row-major). None = no room at position.
    pub room_grid: Vec<Option<SmolStr>>,
    /// Grid width in rooms.
    pub grid_width: usize,
    /// Grid height in rooms.
    pub grid_height: usize,

    /// All rooms by ID.
    pub rooms: IndexMap<SmolStr, Room>,

    /// Starting room ID.
    pub start_room: SmolStr,
    /// Starting X position in pixels.
    pub start_x: f32,
    /// Starting Y position in pixels.
    pub start_y: f32,

    /// World-level properties.
    #[serde(default)]
    pub properties: IndexMap<SmolStr, SmolStr>,
}

impl WorldMap {
    /// Create a new empty world with the given grid dimensions.
    pub fn new(id: impl Into<SmolStr>, grid_width: usize, grid_height: usize) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            room_grid: vec![None; grid_width * grid_height],
            grid_width,
            grid_height,
            rooms: IndexMap::new(),
            start_room: SmolStr::default(),
            start_x: 0.0,
            start_y: 0.0,
            properties: IndexMap::new(),
        }
    }

    /// Add a room to the world at a grid position.
    pub fn add_room(&mut self, room: Room, grid_x: usize, grid_y: usize) {
        if grid_x < self.grid_width && grid_y < self.grid_height {
            let idx = grid_y * self.grid_width + grid_x;
            self.room_grid[idx] = Some(room.id.clone());
            self.rooms.insert(room.id.clone(), room);
        }
    }

    /// Add a room without placing it in the grid (for dungeons, interiors, etc.).
    pub fn add_unplaced_room(&mut self, room: Room) {
        self.rooms.insert(room.id.clone(), room);
    }

    /// Get a room by ID.
    pub fn room(&self, id: &str) -> Option<&Room> {
        self.rooms.get(id)
    }

    /// Get a mutable room by ID.
    pub fn room_mut(&mut self, id: &str) -> Option<&mut Room> {
        self.rooms.get_mut(id)
    }

    /// Get the room ID at a grid position.
    pub fn room_id_at_grid(&self, gx: usize, gy: usize) -> Option<&SmolStr> {
        if gx >= self.grid_width || gy >= self.grid_height {
            return None;
        }
        self.room_grid[gy * self.grid_width + gx].as_ref()
    }

    /// Get the room at a grid position.
    pub fn room_at_grid(&self, gx: usize, gy: usize) -> Option<&Room> {
        self.room_id_at_grid(gx, gy)
            .and_then(|id| self.rooms.get(id))
    }

    /// Get the grid position of a room by ID.
    pub fn grid_position_of(&self, room_id: &str) -> Option<(usize, usize)> {
        for (idx, slot) in self.room_grid.iter().enumerate() {
            if slot.as_ref().map(|s| s.as_str()) == Some(room_id) {
                return Some((idx % self.grid_width, idx / self.grid_width));
            }
        }
        None
    }

    /// Get the adjacent room in a direction.
    pub fn adjacent_room(&self, room_id: &str, dir: Direction) -> Option<&Room> {
        let (gx, gy) = self.grid_position_of(room_id)?;
        let (dx, dy) = dir.as_delta();
        let nx = gx as i32 + dx;
        let ny = gy as i32 + dy;

        if nx < 0 || ny < 0 {
            return None;
        }

        self.room_at_grid(nx as usize, ny as usize)
    }

    /// Get the adjacent room ID in a direction.
    pub fn adjacent_room_id(&self, room_id: &str, dir: Direction) -> Option<&SmolStr> {
        let (gx, gy) = self.grid_position_of(room_id)?;
        let (dx, dy) = dir.as_delta();
        let nx = gx as i32 + dx;
        let ny = gy as i32 + dy;

        if nx < 0 || ny < 0 {
            return None;
        }

        self.room_id_at_grid(nx as usize, ny as usize)
    }

    /// Check if movement from one room to an adjacent room is possible.
    pub fn can_move_to_adjacent(&self, room_id: &str, dir: Direction) -> bool {
        self.adjacent_room(room_id, dir).is_some()
    }

    /// Get the starting room.
    pub fn starting_room(&self) -> Option<&Room> {
        self.rooms.get(&self.start_room)
    }

    /// Set the starting position.
    pub fn set_start(&mut self, room_id: impl Into<SmolStr>, x: f32, y: f32) {
        self.start_room = room_id.into();
        self.start_x = x;
        self.start_y = y;
    }

    /// Iterate over all rooms.
    pub fn iter_rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }

    /// Iterate over rooms in the grid with their positions.
    pub fn iter_grid(&self) -> impl Iterator<Item = (usize, usize, &Room)> {
        self.room_grid
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| {
                let room_id = slot.as_ref()?;
                let room = self.rooms.get(room_id)?;
                let gx = idx % self.grid_width;
                let gy = idx / self.grid_width;
                Some((gx, gy, room))
            })
    }

    /// Get a property value.
    pub fn property(&self, key: &str) -> Option<&SmolStr> {
        self.properties.get(key)
    }

    /// Validate the world (check for broken references, etc.).
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check starting room exists
        if !self.rooms.contains_key(&self.start_room) {
            errors.push(format!("Starting room '{}' not found", self.start_room));
        }

        // Check all grid references point to existing rooms
        for (idx, slot) in self.room_grid.iter().enumerate() {
            if let Some(room_id) = slot {
                if !self.rooms.contains_key(room_id) {
                    let gx = idx % self.grid_width;
                    let gy = idx / self.grid_width;
                    errors.push(format!(
                        "Grid position ({}, {}) references non-existent room '{}'",
                        gx, gy, room_id
                    ));
                }
            }
        }

        // Check exit target rooms exist
        for room in self.rooms.values() {
            for exit in &room.exits {
                if !self.rooms.contains_key(&exit.target_room) {
                    errors.push(format!(
                        "Room '{}' has exit to non-existent room '{}'",
                        room.id, exit.target_room
                    ));
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_world() -> WorldMap {
        let mut world = WorldMap::new("test", 3, 3);

        // Create a 3x3 grid of rooms
        for gy in 0..3 {
            for gx in 0..3 {
                let id = format!("room_{}_{}", gx, gy);
                let room = Room::new(&id, "tileset");
                world.add_room(room, gx, gy);
            }
        }

        world.set_start("room_1_1", 128.0, 88.0);
        world
    }

    #[test]
    fn grid_navigation() {
        let world = make_test_world();

        // Center room
        let center = world.room("room_1_1").unwrap();
        assert_eq!(center.id.as_str(), "room_1_1");

        // Adjacent rooms
        let north = world.adjacent_room("room_1_1", Direction::North).unwrap();
        assert_eq!(north.id.as_str(), "room_1_0");

        let south = world.adjacent_room("room_1_1", Direction::South).unwrap();
        assert_eq!(south.id.as_str(), "room_1_2");

        // Edge cases - corner room can't go further
        assert!(world.adjacent_room("room_0_0", Direction::North).is_none());
        assert!(world.adjacent_room("room_0_0", Direction::West).is_none());
    }

    #[test]
    fn grid_position_lookup() {
        let world = make_test_world();

        assert_eq!(world.grid_position_of("room_0_0"), Some((0, 0)));
        assert_eq!(world.grid_position_of("room_2_1"), Some((2, 1)));
        assert_eq!(world.grid_position_of("nonexistent"), None);
    }

    #[test]
    fn validation() {
        let mut world = WorldMap::new("test", 2, 2);
        world.start_room = "nonexistent".into();

        let errors = world.validate();
        assert!(errors.iter().any(|e| e.contains("Starting room")));
    }
}
