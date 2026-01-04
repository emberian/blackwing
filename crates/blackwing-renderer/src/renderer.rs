//! Main renderer coordinating all rendering operations.

use crate::config::RenderConfig;
use crate::sprites::SpriteSheet;
use crate::tilemap::TilemapRenderer;
use crate::transition::{FadeTransition, ScreenTransition};
use blackwing_tilemap::{Room, TileSet, WorldMap};
use macroquad::prelude::*;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// Main renderer for the game.
pub struct Renderer {
    /// Render configuration.
    pub config: RenderConfig,
    /// Low-resolution render target.
    render_target: RenderTarget,
    /// Tilemap renderer.
    pub tilemap: TilemapRenderer,
    /// Loaded sprite sheets by ID.
    sprites: FxHashMap<SmolStr, SpriteSheet>,
    /// Current screen transition (if any).
    screen_transition: Option<ScreenTransition>,
    /// Current fade transition (if any).
    fade_transition: Option<FadeTransition>,
}

impl Renderer {
    /// Create a new renderer with the given configuration.
    pub fn new(config: RenderConfig) -> Self {
        let render_target = render_target(config.target_width, config.target_height);
        render_target.texture.set_filter(FilterMode::Nearest);

        Self {
            config,
            render_target,
            tilemap: TilemapRenderer::new(),
            sprites: FxHashMap::default(),
            screen_transition: None,
            fade_transition: None,
        }
    }

    /// Create a renderer with default (Zelda-style) configuration.
    pub fn zelda() -> Self {
        Self::new(RenderConfig::zelda())
    }

    /// Load a tileset.
    pub async fn load_tileset(&mut self, tileset: &TileSet, base_path: &str) -> Result<(), String> {
        self.tilemap.load_tileset(tileset, base_path).await
    }

    /// Load all tilesets from a world map.
    pub async fn load_world_tilesets(
        &mut self,
        world: &WorldMap,
        tilesets: &[TileSet],
        base_path: &str,
    ) -> Result<(), String> {
        // Find all unique tileset IDs used in the world
        let mut used_tilesets = std::collections::HashSet::new();
        for room in world.rooms.values() {
            used_tilesets.insert(room.tileset_id.as_str());
        }

        // Load each required tileset
        for tileset in tilesets {
            if used_tilesets.contains(tileset.id.as_str()) {
                self.load_tileset(tileset, base_path).await?;
            }
        }

        Ok(())
    }

    /// Load a sprite sheet.
    pub async fn load_sprite_sheet(
        &mut self,
        id: impl Into<SmolStr>,
        path: &str,
        sprite_width: u16,
        sprite_height: u16,
    ) -> Result<(), String> {
        let id = id.into();
        let sheet = SpriteSheet::load(id.clone(), path, sprite_width, sprite_height).await?;
        self.sprites.insert(id, sheet);
        Ok(())
    }

    /// Get a sprite sheet by ID.
    pub fn sprite_sheet(&self, id: &str) -> Option<&SpriteSheet> {
        self.sprites.get(id)
    }

    /// Start a screen transition.
    pub fn start_screen_transition(&mut self, transition: ScreenTransition) {
        self.screen_transition = Some(transition);
    }

    /// Start a fade transition.
    pub fn start_fade(&mut self, fade: FadeTransition) {
        self.fade_transition = Some(fade);
    }

    /// Check if a screen transition is in progress.
    pub fn is_transitioning(&self) -> bool {
        self.screen_transition.is_some()
    }

    /// Check if a fade is in progress.
    pub fn is_fading(&self) -> bool {
        self.fade_transition.is_some()
    }

    /// Get the current transition (if any).
    pub fn current_transition(&self) -> Option<&ScreenTransition> {
        self.screen_transition.as_ref()
    }

    /// Update transitions.
    pub fn update(&mut self, delta_ms: f32) {
        // Update tilemap animations
        self.tilemap.update(delta_ms as u64);

        // Update screen transition
        if let Some(ref mut transition) = self.screen_transition {
            if transition.update(delta_ms) {
                self.screen_transition = None;
            }
        }

        // Update fade transition
        if let Some(ref mut fade) = self.fade_transition {
            if fade.update(delta_ms) {
                self.fade_transition = None;
            }
        }
    }

    /// Begin rendering to the low-res target.
    pub fn begin_frame(&self) {
        // Set up camera for low-res rendering
        set_camera(&Camera2D {
            render_target: Some(self.render_target.clone()),
            zoom: vec2(
                2.0 / self.config.target_width as f32,
                2.0 / self.config.target_height as f32,
            ),
            target: vec2(
                self.config.target_width as f32 / 2.0,
                self.config.target_height as f32 / 2.0,
            ),
            ..Default::default()
        });

        // Clear with background color
        clear_background(Color::from_rgba(
            (self.config.clear_color[0] * 255.0) as u8,
            (self.config.clear_color[1] * 255.0) as u8,
            (self.config.clear_color[2] * 255.0) as u8,
            (self.config.clear_color[3] * 255.0) as u8,
        ));
    }

    /// Finish rendering and display the result.
    pub fn end_frame(&self) {
        // Reset to default camera
        set_default_camera();
        clear_background(BLACK);

        // Calculate scaling
        let scale = if self.config.pixel_perfect {
            // Integer scaling only
            let scale_x = (screen_width() / self.config.target_width as f32).floor() as u32;
            let scale_y = (screen_height() / self.config.target_height as f32).floor() as u32;
            scale_x.min(scale_y).max(1) as f32
        } else {
            // Fill the screen
            let scale_x = screen_width() / self.config.target_width as f32;
            let scale_y = screen_height() / self.config.target_height as f32;
            scale_x.min(scale_y)
        };

        let display_width = self.config.target_width as f32 * scale;
        let display_height = self.config.target_height as f32 * scale;

        // Center on screen
        let x = (screen_width() - display_width) / 2.0;
        let y = (screen_height() - display_height) / 2.0;

        // Draw the render target scaled up
        draw_texture_ex(
            &self.render_target.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(display_width, display_height)),
                ..Default::default()
            },
        );

        // Draw fade overlay if active
        if let Some(ref fade) = self.fade_transition {
            let alpha = fade.alpha();
            if alpha > 0.0 {
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, alpha),
                );
            }
        }
    }

    /// Draw a room with optional camera offset.
    pub fn draw_room(&self, room: &Room, camera_x: f32, camera_y: f32) {
        self.tilemap.draw_room(room, camera_x, camera_y);
    }

    /// Draw rooms during a screen transition.
    pub fn draw_transition(&self, old_room: &Room, new_room: &Room, tile_size: u16) {
        let Some(ref transition) = self.screen_transition else {
            // No transition, just draw current room
            self.tilemap.draw_room(new_room, 0.0, 0.0);
            return;
        };

        let room_width = old_room.width as f32 * tile_size as f32;
        let room_height = old_room.height as f32 * tile_size as f32;

        let ((old_x, old_y), (new_x, new_y)) = transition.offsets(room_width, room_height);

        // Draw both rooms at their transition offsets
        self.tilemap.draw_room_offset(old_room, old_x, old_y);
        self.tilemap.draw_room_offset(new_room, new_x, new_y);
    }

    /// Draw a sprite at the given position.
    pub fn draw_sprite(&self, sheet_id: &str, index: u16, x: f32, y: f32) {
        if let Some(sheet) = self.sprites.get(sheet_id) {
            sheet.draw(index, x, y);
        }
    }

    /// Draw a sprite with flip options.
    pub fn draw_sprite_ex(&self, sheet_id: &str, index: u16, x: f32, y: f32, flip_x: bool, flip_y: bool) {
        if let Some(sheet) = self.sprites.get(sheet_id) {
            sheet.draw_ex(index, x, y, flip_x, flip_y);
        }
    }
}
