//! Sprite sheet handling.

use macroquad::prelude::*;
use smol_str::SmolStr;

/// A sprite sheet containing multiple sprites/frames.
#[derive(Debug)]
pub struct SpriteSheet {
    /// The loaded texture.
    pub texture: Texture2D,
    /// Sprite width in pixels.
    pub sprite_width: u16,
    /// Sprite height in pixels.
    pub sprite_height: u16,
    /// Number of sprites per row.
    pub sprites_per_row: u16,
    /// Identifier.
    pub id: SmolStr,
}

impl SpriteSheet {
    /// Load a sprite sheet from a file.
    pub async fn load(
        id: impl Into<SmolStr>,
        path: &str,
        sprite_width: u16,
        sprite_height: u16,
    ) -> Result<Self, String> {
        let texture = load_texture(path)
            .await
            .map_err(|e| format!("Failed to load sprite sheet '{}': {}", path, e))?;

        texture.set_filter(FilterMode::Nearest);

        let sprites_per_row = (texture.width() as u16) / sprite_width;

        Ok(Self {
            texture,
            sprite_width,
            sprite_height,
            sprites_per_row,
            id: id.into(),
        })
    }

    /// Create from an already loaded texture.
    pub fn from_texture(
        id: impl Into<SmolStr>,
        texture: Texture2D,
        sprite_width: u16,
        sprite_height: u16,
    ) -> Self {
        texture.set_filter(FilterMode::Nearest);
        let sprites_per_row = (texture.width() as u16) / sprite_width;

        Self {
            texture,
            sprite_width,
            sprite_height,
            sprites_per_row,
            id: id.into(),
        }
    }

    /// Get the source rectangle for a sprite index.
    pub fn sprite_rect(&self, index: u16) -> Rect {
        let col = index % self.sprites_per_row;
        let row = index / self.sprites_per_row;

        Rect::new(
            col as f32 * self.sprite_width as f32,
            row as f32 * self.sprite_height as f32,
            self.sprite_width as f32,
            self.sprite_height as f32,
        )
    }

    /// Draw a sprite at the given position.
    pub fn draw(&self, index: u16, x: f32, y: f32) {
        self.draw_ex(index, x, y, false, false);
    }

    /// Draw a sprite with flip options.
    pub fn draw_ex(&self, index: u16, x: f32, y: f32, flip_x: bool, flip_y: bool) {
        let source = self.sprite_rect(index);

        draw_texture_ex(
            &self.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(source),
                flip_x,
                flip_y,
                ..Default::default()
            },
        );
    }

    /// Draw a sprite with color tint.
    pub fn draw_tinted(&self, index: u16, x: f32, y: f32, color: Color) {
        let source = self.sprite_rect(index);

        draw_texture_ex(
            &self.texture,
            x,
            y,
            color,
            DrawTextureParams {
                source: Some(source),
                ..Default::default()
            },
        );
    }
}

/// Animation definition for sprites.
#[derive(Debug, Clone)]
pub struct SpriteAnimation {
    /// Frame indices into the sprite sheet.
    pub frames: Vec<u16>,
    /// Duration of each frame in milliseconds.
    pub frame_duration_ms: u32,
    /// Whether the animation loops.
    pub looping: bool,
}

impl SpriteAnimation {
    /// Create a new animation.
    pub fn new(frames: Vec<u16>, frame_duration_ms: u32, looping: bool) -> Self {
        Self {
            frames,
            frame_duration_ms,
            looping,
        }
    }

    /// Get the frame index for a given time.
    pub fn frame_at(&self, time_ms: u64) -> u16 {
        if self.frames.is_empty() || self.frame_duration_ms == 0 {
            return 0;
        }

        let total_duration = self.frame_duration_ms as u64 * self.frames.len() as u64;

        let time_in_anim = if self.looping {
            time_ms % total_duration
        } else {
            time_ms.min(total_duration - 1)
        };

        let frame_index = (time_in_anim / self.frame_duration_ms as u64) as usize;
        self.frames[frame_index.min(self.frames.len() - 1)]
    }

    /// Check if the animation is finished (for non-looping animations).
    pub fn is_finished(&self, time_ms: u64) -> bool {
        if self.looping {
            return false;
        }
        let total_duration = self.frame_duration_ms as u64 * self.frames.len() as u64;
        time_ms >= total_duration
    }
}
