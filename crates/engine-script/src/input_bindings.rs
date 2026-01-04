//! Rhai bindings for input state.
//!
//! These bindings provide script access to player input for real-time games.
//!
//! # Example
//!
//! ```rhai
//! // Check if player is pressing action button
//! if is_action_pressed() {
//!     start_attack(current_entity());
//! }
//!
//! // Get movement input
//! let dx = move_x() * speed * delta_ms / 1000.0;
//! let dy = move_y() * speed * delta_ms / 1000.0;
//! move_with_collision(current_entity(), dx, dy);
//!
//! // Get movement as direction
//! let dir = move_direction();
//! if dir != "" {
//!     set_facing(current_entity(), dir);
//! }
//! ```

use rhai::{Engine, ImmutableString};
use std::sync::Arc;

/// Snapshot of input state for read-only access in scripts.
#[derive(Clone, Default)]
pub struct InputSnapshot {
    /// Horizontal movement axis (-1.0 to 1.0).
    pub move_x: f32,
    /// Vertical movement axis (-1.0 to 1.0).
    pub move_y: f32,
    /// Action button was just pressed this frame.
    pub action_pressed: bool,
    /// Action button is being held down.
    pub action_held: bool,
    /// Secondary button was just pressed this frame.
    pub secondary_pressed: bool,
    /// Secondary button is being held down.
    pub secondary_held: bool,
    /// Pause button was just pressed this frame.
    pub pause_pressed: bool,
    /// Start button was just pressed this frame.
    pub start_pressed: bool,
}

impl InputSnapshot {
    /// Create a new empty input snapshot (no input).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an input snapshot with the given values.
    ///
    /// This is used by the game loop to convert its input state into a snapshot
    /// that can be passed to scripts.
    pub fn from_input(
        move_x: f32,
        move_y: f32,
        action_pressed: bool,
        action_held: bool,
        secondary_pressed: bool,
        secondary_held: bool,
        pause_pressed: bool,
        start_pressed: bool,
    ) -> Self {
        Self {
            move_x,
            move_y,
            action_pressed,
            action_held,
            secondary_pressed,
            secondary_held,
            pause_pressed,
            start_pressed,
        }
    }

    /// Check if any movement input is active.
    pub fn is_moving(&self) -> bool {
        self.move_x != 0.0 || self.move_y != 0.0
    }

    /// Get the primary cardinal direction of movement, or empty string if not moving.
    pub fn direction(&self) -> &'static str {
        // Prioritize the axis with greater magnitude
        if self.move_x.abs() > self.move_y.abs() {
            if self.move_x > 0.5 {
                "east"
            } else if self.move_x < -0.5 {
                "west"
            } else {
                ""
            }
        } else {
            if self.move_y > 0.5 {
                "south"
            } else if self.move_y < -0.5 {
                "north"
            } else {
                ""
            }
        }
    }

    /// Get normalized movement vector.
    pub fn normalized_movement(&self) -> (f32, f32) {
        let len = (self.move_x * self.move_x + self.move_y * self.move_y).sqrt();
        if len > 0.0 {
            (self.move_x / len, self.move_y / len)
        } else {
            (0.0, 0.0)
        }
    }
}

/// Register input bindings with a Rhai engine.
///
/// These bindings provide read-only access to input state for scripts.
pub fn register_input_bindings(engine: &mut Engine, snapshot: Arc<InputSnapshot>) {
    // Movement axis - horizontal
    let snap = snapshot.clone();
    engine.register_fn("move_x", move || -> f64 { snap.move_x as f64 });

    // Movement axis - vertical
    let snap = snapshot.clone();
    engine.register_fn("move_y", move || -> f64 { snap.move_y as f64 });

    // Check if any movement input
    let snap = snapshot.clone();
    engine.register_fn("is_moving", move || -> bool { snap.is_moving() });

    // Get movement direction as string
    let snap = snapshot.clone();
    engine.register_fn("move_direction", move || -> ImmutableString {
        snap.direction().into()
    });

    // Action button (just pressed)
    let snap = snapshot.clone();
    engine.register_fn("is_action_pressed", move || -> bool { snap.action_pressed });

    // Action button (held down)
    let snap = snapshot.clone();
    engine.register_fn("is_action_held", move || -> bool { snap.action_held });

    // Secondary button (just pressed)
    let snap = snapshot.clone();
    engine.register_fn("is_secondary_pressed", move || -> bool {
        snap.secondary_pressed
    });

    // Secondary button (held down)
    let snap = snapshot.clone();
    engine.register_fn("is_secondary_held", move || -> bool { snap.secondary_held });

    // Pause button (just pressed)
    let snap = snapshot.clone();
    engine.register_fn("is_pause_pressed", move || -> bool { snap.pause_pressed });

    // Start button (just pressed)
    let snap = snapshot.clone();
    engine.register_fn("is_start_pressed", move || -> bool { snap.start_pressed });

    // Normalized movement (returns array [x, y])
    let snap = snapshot.clone();
    engine.register_fn("normalized_movement", move || -> rhai::Array {
        let (x, y) = snap.normalized_movement();
        vec![rhai::Dynamic::from(x as f64), rhai::Dynamic::from(y as f64)]
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_snapshot_defaults() {
        let snap = InputSnapshot::new();
        assert!(!snap.is_moving());
        assert_eq!(snap.direction(), "");
        assert!(!snap.action_pressed);
        assert!(!snap.action_held);
    }

    #[test]
    fn input_snapshot_directions() {
        let mut snap = InputSnapshot::new();

        // Moving right
        snap.move_x = 1.0;
        snap.move_y = 0.0;
        assert!(snap.is_moving());
        assert_eq!(snap.direction(), "east");

        // Moving down
        snap.move_x = 0.0;
        snap.move_y = 1.0;
        assert_eq!(snap.direction(), "south");

        // Diagonal - should pick stronger axis
        snap.move_x = 0.7;
        snap.move_y = 0.3;
        assert_eq!(snap.direction(), "east");

        snap.move_x = 0.3;
        snap.move_y = -0.7;
        assert_eq!(snap.direction(), "north");
    }

    #[test]
    fn normalized_movement() {
        let mut snap = InputSnapshot::new();
        snap.move_x = 1.0;
        snap.move_y = 1.0;

        let (nx, ny) = snap.normalized_movement();
        let len = (nx * nx + ny * ny).sqrt();
        assert!((len - 1.0).abs() < 0.001);
    }

    #[test]
    fn rhai_bindings() {
        let mut snap = InputSnapshot::new();
        snap.move_x = 1.0;
        snap.action_pressed = true;

        let snapshot = Arc::new(snap);
        let mut engine = Engine::new();
        register_input_bindings(&mut engine, snapshot);

        let move_x: f64 = engine.eval("move_x()").unwrap();
        assert_eq!(move_x, 1.0);

        let action: bool = engine.eval("is_action_pressed()").unwrap();
        assert!(action);

        let moving: bool = engine.eval("is_moving()").unwrap();
        assert!(moving);

        let dir: String = engine.eval("move_direction()").unwrap();
        assert_eq!(dir, "east");
    }
}
