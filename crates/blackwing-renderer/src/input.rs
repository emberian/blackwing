//! Input handling abstraction.

use blackwing_tilemap::Direction;
use macroquad::prelude::*;

/// Current input state.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// Movement direction (normalized).
    pub move_x: f32,
    pub move_y: f32,
    /// Action button (sword/attack).
    pub action: bool,
    /// Secondary action (item use).
    pub secondary: bool,
    /// Pause/menu button.
    pub pause: bool,
    /// Start button.
    pub start: bool,
    /// Any direction pressed.
    pub moving: bool,
}

impl InputState {
    /// Get the movement as a direction (if moving in a cardinal direction).
    pub fn direction(&self) -> Option<Direction> {
        // Prioritize vertical over horizontal for 4-way movement
        if self.move_y < -0.5 {
            Some(Direction::North)
        } else if self.move_y > 0.5 {
            Some(Direction::South)
        } else if self.move_x < -0.5 {
            Some(Direction::West)
        } else if self.move_x > 0.5 {
            Some(Direction::East)
        } else {
            None
        }
    }

    /// Get movement as a normalized vector.
    pub fn movement(&self) -> (f32, f32) {
        let len = (self.move_x * self.move_x + self.move_y * self.move_y).sqrt();
        if len > 0.0 {
            (self.move_x / len, self.move_y / len)
        } else {
            (0.0, 0.0)
        }
    }
}

/// Input handler that reads from keyboard/gamepad.
#[derive(Debug, Clone, Default)]
pub struct Input {
    /// Previous frame's input state (for edge detection).
    previous: InputState,
    /// Current frame's input state.
    current: InputState,
}

impl Input {
    /// Create a new input handler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Poll input for the current frame.
    pub fn poll(&mut self) {
        self.previous = self.current.clone();

        // Reset
        self.current = InputState::default();

        // Keyboard input
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            self.current.move_y -= 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            self.current.move_y += 1.0;
        }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.current.move_x -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.current.move_x += 1.0;
        }

        // Action buttons
        self.current.action = is_key_down(KeyCode::Z) || is_key_down(KeyCode::Space);
        self.current.secondary = is_key_down(KeyCode::X) || is_key_down(KeyCode::LeftShift);
        self.current.pause = is_key_down(KeyCode::Escape);
        self.current.start = is_key_down(KeyCode::Enter);

        // Set moving flag
        self.current.moving = self.current.move_x != 0.0 || self.current.move_y != 0.0;

        // TODO: Gamepad support
        // if let Some(gamepad) = gamepads().next() { ... }
    }

    /// Get the current input state.
    pub fn state(&self) -> &InputState {
        &self.current
    }

    /// Check if action button was just pressed this frame.
    pub fn action_pressed(&self) -> bool {
        self.current.action && !self.previous.action
    }

    /// Check if secondary button was just pressed this frame.
    pub fn secondary_pressed(&self) -> bool {
        self.current.secondary && !self.previous.secondary
    }

    /// Check if pause was just pressed this frame.
    pub fn pause_pressed(&self) -> bool {
        self.current.pause && !self.previous.pause
    }

    /// Check if start was just pressed this frame.
    pub fn start_pressed(&self) -> bool {
        self.current.start && !self.previous.start
    }
}
