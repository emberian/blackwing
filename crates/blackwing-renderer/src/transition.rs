//! Screen transition effects.

use blackwing_tilemap::Direction;
use smol_str::SmolStr;

/// Screen scroll transition (Zelda-style).
#[derive(Debug, Clone)]
pub struct ScreenTransition {
    /// Room we're leaving.
    pub from_room: SmolStr,
    /// Room we're entering.
    pub to_room: SmolStr,
    /// Direction of transition.
    pub direction: Direction,
    /// Total transition duration in milliseconds.
    pub duration_ms: u32,
    /// Elapsed time in milliseconds.
    pub elapsed_ms: f32,
}

impl ScreenTransition {
    /// Create a new screen transition.
    pub fn new(
        from_room: SmolStr,
        to_room: SmolStr,
        direction: Direction,
        duration_ms: u32,
    ) -> Self {
        Self {
            from_room,
            to_room,
            direction,
            duration_ms,
            elapsed_ms: 0.0,
        }
    }

    /// Get the transition progress (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        (self.elapsed_ms / self.duration_ms as f32).clamp(0.0, 1.0)
    }

    /// Update the transition. Returns true if finished.
    pub fn update(&mut self, delta_ms: f32) -> bool {
        self.elapsed_ms += delta_ms;
        self.elapsed_ms >= self.duration_ms as f32
    }

    /// Get the offsets for rendering both rooms during transition.
    ///
    /// Returns ((old_x, old_y), (new_x, new_y)) offsets.
    pub fn offsets(&self, room_width: f32, room_height: f32) -> ((f32, f32), (f32, f32)) {
        let t = self.progress();

        match self.direction {
            Direction::North => {
                // Old room scrolls down, new room comes from above
                let old_offset = (0.0, t * room_height);
                let new_offset = (0.0, (t - 1.0) * room_height);
                (old_offset, new_offset)
            }
            Direction::South => {
                // Old room scrolls up, new room comes from below
                let old_offset = (0.0, -t * room_height);
                let new_offset = (0.0, (1.0 - t) * room_height);
                (old_offset, new_offset)
            }
            Direction::West => {
                // Old room scrolls right, new room comes from left
                let old_offset = (t * room_width, 0.0);
                let new_offset = ((t - 1.0) * room_width, 0.0);
                (old_offset, new_offset)
            }
            Direction::East => {
                // Old room scrolls left, new room comes from right
                let old_offset = (-t * room_width, 0.0);
                let new_offset = ((1.0 - t) * room_width, 0.0);
                (old_offset, new_offset)
            }
        }
    }

    /// Get the player position offset during transition.
    ///
    /// The player should move with the old room, appearing to stay in place
    /// relative to the screen center during the scroll.
    pub fn player_offset(&self, room_width: f32, room_height: f32) -> (f32, f32) {
        let t = self.progress();

        match self.direction {
            Direction::North => (0.0, t * room_height),
            Direction::South => (0.0, -t * room_height),
            Direction::West => (t * room_width, 0.0),
            Direction::East => (-t * room_width, 0.0),
        }
    }

    /// Check if the transition is finished.
    pub fn is_finished(&self) -> bool {
        self.elapsed_ms >= self.duration_ms as f32
    }
}

/// Fade transition effect.
#[derive(Debug, Clone)]
pub struct FadeTransition {
    /// Fade direction (true = fading out, false = fading in).
    pub fading_out: bool,
    /// Total duration in milliseconds.
    pub duration_ms: u32,
    /// Elapsed time in milliseconds.
    pub elapsed_ms: f32,
}

impl FadeTransition {
    /// Create a fade out transition.
    pub fn fade_out(duration_ms: u32) -> Self {
        Self {
            fading_out: true,
            duration_ms,
            elapsed_ms: 0.0,
        }
    }

    /// Create a fade in transition.
    pub fn fade_in(duration_ms: u32) -> Self {
        Self {
            fading_out: false,
            duration_ms,
            elapsed_ms: 0.0,
        }
    }

    /// Get the current alpha (0.0 = fully transparent, 1.0 = fully black).
    pub fn alpha(&self) -> f32 {
        let t = (self.elapsed_ms / self.duration_ms as f32).clamp(0.0, 1.0);
        if self.fading_out {
            t
        } else {
            1.0 - t
        }
    }

    /// Update the transition. Returns true if finished.
    pub fn update(&mut self, delta_ms: f32) -> bool {
        self.elapsed_ms += delta_ms;
        self.elapsed_ms >= self.duration_ms as f32
    }

    /// Check if the transition is finished.
    pub fn is_finished(&self) -> bool {
        self.elapsed_ms >= self.duration_ms as f32
    }
}
