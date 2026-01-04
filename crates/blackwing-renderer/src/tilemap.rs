//! Tilemap rendering.

use blackwing_tilemap::{Room, TileSet};
use macroquad::prelude::*;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// Renderer for tilemaps.
pub struct TilemapRenderer {
    /// Loaded tileset textures by tileset ID.
    textures: FxHashMap<SmolStr, Texture2D>,
    /// Cached tileset data.
    tilesets: FxHashMap<SmolStr, TileSet>,
    /// Current animation time in milliseconds.
    time_ms: u64,
}

impl TilemapRenderer {
    /// Create a new tilemap renderer.
    pub fn new() -> Self {
        Self {
            textures: FxHashMap::default(),
            tilesets: FxHashMap::default(),
            time_ms: 0,
        }
    }

    /// Load a tileset texture.
    pub async fn load_tileset(&mut self, tileset: &TileSet, base_path: &str) -> Result<(), String> {
        let path = format!("{}/{}", base_path, tileset.texture_path);
        let texture = load_texture(&path)
            .await
            .map_err(|e| format!("Failed to load tileset '{}': {}", path, e))?;

        texture.set_filter(FilterMode::Nearest);

        self.textures.insert(tileset.id.clone(), texture);
        self.tilesets.insert(tileset.id.clone(), tileset.clone());

        Ok(())
    }

    /// Add a tileset with an already loaded texture.
    pub fn add_tileset(&mut self, tileset: TileSet, texture: Texture2D) {
        texture.set_filter(FilterMode::Nearest);
        self.textures.insert(tileset.id.clone(), texture);
        self.tilesets.insert(tileset.id.clone(), tileset);
    }

    /// Update animation time.
    pub fn update(&mut self, delta_ms: u64) {
        self.time_ms = self.time_ms.wrapping_add(delta_ms);
    }

    /// Draw a room at the given camera offset.
    pub fn draw_room(&self, room: &Room, camera_x: f32, camera_y: f32) {
        let Some(tileset) = self.tilesets.get(&room.tileset_id) else {
            return;
        };
        let Some(texture) = self.textures.get(&room.tileset_id) else {
            return;
        };

        let tw = tileset.tile_width as f32;
        let th = tileset.tile_height as f32;

        for y in 0..room.height {
            for x in 0..room.width {
                let tile_id = room.tiles[y * room.width + x];

                // Get tile definition for animation support
                let texture_index = if let Some(tile_def) = tileset.get_tile(tile_id) {
                    tile_def.texture_index_at(self.time_ms)
                } else {
                    tile_id
                };

                // Calculate source rectangle in tileset texture
                let src_col = texture_index % tileset.tiles_per_row;
                let src_row = texture_index / tileset.tiles_per_row;
                let src_rect = Rect::new(
                    src_col as f32 * tw,
                    src_row as f32 * th,
                    tw,
                    th,
                );

                // Calculate destination position
                let dest_x = x as f32 * tw - camera_x;
                let dest_y = y as f32 * th - camera_y;

                draw_texture_ex(
                    texture,
                    dest_x,
                    dest_y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(src_rect),
                        ..Default::default()
                    },
                );
            }
        }
    }

    /// Draw a room at a specific offset (for transitions).
    pub fn draw_room_offset(&self, room: &Room, offset_x: f32, offset_y: f32) {
        self.draw_room(room, -offset_x, -offset_y);
    }

    /// Get a tileset by ID.
    pub fn tileset(&self, id: &str) -> Option<&TileSet> {
        self.tilesets.get(id)
    }

    /// Check if a tileset is loaded.
    pub fn has_tileset(&self, id: &str) -> bool {
        self.textures.contains_key(id)
    }
}

impl Default for TilemapRenderer {
    fn default() -> Self {
        Self::new()
    }
}
