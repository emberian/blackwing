//! Macroquad-based renderer for Blackwing engine tilemap games.
//!
//! Provides:
//! - Pixel-perfect 2D rendering with low-res render target
//! - Tilemap rendering with animation support
//! - Entity sprite rendering
//! - Screen transition effects (Zelda-style scrolling)
//! - Input handling abstraction
//! - Game loop integration

mod config;
mod game_loop;
mod input;
mod renderer;
mod script_runner;
mod sprites;
mod tilemap;
mod transition;

pub use config::RenderConfig;
pub use game_loop::{GameLoop, GameLoopConfig, GameLoopState};
pub use input::{Input, InputState};
pub use renderer::Renderer;
pub use script_runner::ScriptRunner;
pub use sprites::{SpriteAnimation, SpriteSheet};
pub use tilemap::TilemapRenderer;
pub use transition::{FadeTransition, ScreenTransition};

// Re-export macroquad for convenience
pub use macroquad;
