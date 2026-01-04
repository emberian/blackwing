//! Main game loop integrating renderer, runtime, and tilemap.
//!
//! This module provides the core game loop for real-time Zelda-style gameplay,
//! coordinating input handling, physics, rendering, and screen transitions.

use crate::input::Input;
use crate::renderer::Renderer;
use crate::transition::ScreenTransition;
use blackwing_core::ecs::EntityKey;
use blackwing_core::{Effect, EntityId, Value, WorldState};
use blackwing_tilemap::{can_place, resolve_movement, Direction, Room, TileSet, WorldMap};
use macroquad::prelude::*;
use smol_str::SmolStr;

/// Configuration for the game loop.
#[derive(Debug, Clone)]
pub struct GameLoopConfig {
    /// Player movement speed in pixels per second.
    pub player_speed: f32,
    /// Player hitbox width.
    pub hitbox_width: f32,
    /// Player hitbox height.
    pub hitbox_height: f32,
    /// Tile size in pixels.
    pub tile_size: u16,
    /// Screen transition duration in milliseconds.
    pub transition_duration_ms: u32,
}

impl Default for GameLoopConfig {
    fn default() -> Self {
        Self {
            player_speed: 120.0, // pixels per second
            hitbox_width: 12.0,
            hitbox_height: 12.0,
            tile_size: 16,
            transition_duration_ms: 300,
        }
    }
}

/// State for the game loop.
pub struct GameLoopState {
    /// Current world map.
    pub world: WorldMap,
    /// Available tilesets.
    pub tilesets: Vec<TileSet>,
    /// Current room ID.
    pub current_room_id: SmolStr,
    /// Player entity ID.
    pub player_id: EntityId,
    /// Player entity key (cached for quick access).
    pub player_key: Option<EntityKey>,
    /// Whether a screen transition is in progress.
    pub transitioning: bool,
    /// Target room after transition completes.
    pub transition_target: Option<(SmolStr, f32, f32)>,
}

impl GameLoopState {
    /// Create a new game loop state.
    pub fn new(world: WorldMap, tilesets: Vec<TileSet>, player_id: EntityId) -> Self {
        let current_room_id = world.start_room.clone();
        Self {
            world,
            tilesets,
            current_room_id,
            player_id,
            player_key: None,
            transitioning: false,
            transition_target: None,
        }
    }

    /// Get the current room.
    pub fn current_room(&self) -> Option<&Room> {
        self.world.room(&self.current_room_id)
    }

    /// Get the tileset for the current room.
    pub fn current_tileset(&self) -> Option<&TileSet> {
        let room = self.current_room()?;
        self.tilesets.iter().find(|t| t.id == room.tileset_id)
    }

    /// Cache the player entity key from world state.
    pub fn cache_player_key(&mut self, state: &WorldState) {
        self.player_key = state.entities.key_of(&self.player_id);
    }
}

/// The main game loop coordinator.
pub struct GameLoop {
    /// Renderer instance.
    pub renderer: Renderer,
    /// Input handler.
    pub input: Input,
    /// Game loop configuration.
    pub config: GameLoopConfig,
    /// Accumulated time for fixed timestep (if needed).
    accumulated_time: f32,
}

impl GameLoop {
    /// Create a new game loop with the given renderer.
    pub fn new(renderer: Renderer, config: GameLoopConfig) -> Self {
        Self {
            renderer,
            input: Input::new(),
            config,
            accumulated_time: 0.0,
        }
    }

    /// Create with default configuration.
    pub fn with_renderer(renderer: Renderer) -> Self {
        Self::new(renderer, GameLoopConfig::default())
    }

    /// Run one frame of the game loop.
    ///
    /// Returns a list of effects to apply to the world state.
    pub fn update(
        &mut self,
        game_state: &mut GameLoopState,
        world_state: &WorldState,
    ) -> Vec<Effect> {
        let delta_s = get_frame_time();
        let delta_ms = (delta_s * 1000.0) as u32;

        // Poll input
        self.input.poll();

        // Update renderer (animations, transitions)
        self.renderer.update(delta_ms as f32);

        // If transitioning, don't process gameplay
        if self.renderer.is_transitioning() {
            // Check if transition finished
            if !self.renderer.is_transitioning() {
                if let Some((room_id, spawn_x, spawn_y)) = game_state.transition_target.take() {
                    game_state.current_room_id = room_id;
                    game_state.transitioning = false;
                    // Return effect to set player position
                    return vec![Effect::set_position(
                        game_state.player_id.clone(),
                        spawn_x,
                        spawn_y,
                    )];
                }
            }
            return Vec::new();
        }

        let mut effects = Vec::new();

        // Get player position from world state
        let (player_x, player_y) = self.get_player_position(game_state, world_state);

        // Process movement input
        let input_state = self.input.state();
        if input_state.moving {
            let (move_x, move_y) = input_state.movement();
            // Get speed from entity component, fall back to config default
            let base_speed = self.get_entity_speed(game_state, world_state);
            let speed = base_speed * delta_s;
            let dx = move_x * speed;
            let dy = move_y * speed;

            // Resolve collision
            if let (Some(room), Some(tileset)) =
                (game_state.current_room(), game_state.current_tileset())
            {
                // Get hitbox from entity component, fall back to config default
                let (hitbox_w, hitbox_h) = self.get_entity_hitbox(game_state, world_state);
                let (resolved_dx, resolved_dy) = resolve_movement(
                    room,
                    tileset,
                    player_x,
                    player_y,
                    hitbox_w,
                    hitbox_h,
                    dx,
                    dy,
                );

                if resolved_dx != 0.0 || resolved_dy != 0.0 {
                    effects.push(Effect::move_by(
                        game_state.player_id.clone(),
                        resolved_dx,
                        resolved_dy,
                    ));

                    // Update facing direction
                    let facing = if resolved_dx.abs() > resolved_dy.abs() {
                        if resolved_dx > 0.0 { "east" } else { "west" }
                    } else {
                        if resolved_dy > 0.0 { "south" } else { "north" }
                    };
                    effects.push(Effect::set_component(
                        game_state.player_id.clone(),
                        "facing",
                        Value::String(facing.into()),
                    ));
                }
            }
        }

        // Check for screen edge transitions
        let new_pos = self.get_player_position_after_effects(
            player_x,
            player_y,
            &effects,
            &game_state.player_id,
        );
        if let Some(transition_effect) =
            self.check_screen_transition(game_state, new_pos.0, new_pos.1)
        {
            effects.push(transition_effect);
        }

        // Update hitbox lifetimes (despawn expired hitboxes)
        effects.extend(Self::update_hitbox_lifetimes(world_state, delta_ms as i64));

        // Update invincibility timers
        effects.extend(Self::update_invincibility_timers(world_state, delta_ms as i64));

        // Update projectile movement
        if let (Some(room), Some(tileset)) =
            (game_state.current_room(), game_state.current_tileset())
        {
            effects.extend(Self::update_projectiles(world_state, room, tileset, delta_s));
        }

        effects
    }

    /// Update hitbox lifetimes and return despawn effects for expired hitboxes.
    fn update_hitbox_lifetimes(world_state: &WorldState, delta_ms: i64) -> Vec<Effect> {
        let mut effects = Vec::new();

        for (_key, entity) in world_state.entities.iter() {
            // Check if entity has "hitbox" tag
            if !entity.tags.iter().any(|t| t.as_str() == "hitbox") {
                continue;
            }

            // Get current lifetime
            let lifetime = world_state
                .entities
                .get_component(_key, "lifetime")
                .and_then(|v| v.as_int())
                .unwrap_or(0);

            let new_lifetime = lifetime - delta_ms;

            if new_lifetime <= 0 {
                // Hitbox expired, despawn it
                effects.push(Effect::despawn(entity.id.clone()));
            } else {
                // Update lifetime
                effects.push(Effect::set_component(
                    entity.id.clone(),
                    "lifetime",
                    Value::Int(new_lifetime),
                ));
            }
        }

        effects
    }

    /// Update invincibility timers and clear invincibility when timer expires.
    fn update_invincibility_timers(world_state: &WorldState, delta_ms: i64) -> Vec<Effect> {
        let mut effects = Vec::new();

        for (key, entity) in world_state.entities.iter() {
            // Check if entity is invincible
            let invincible = world_state
                .entities
                .get_component(key, "invincible")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !invincible {
                continue;
            }

            // Get invincibility timer
            let timer = world_state
                .entities
                .get_component(key, "invincible_timer")
                .and_then(|v| v.as_int())
                .unwrap_or(0);

            let new_timer = timer - delta_ms;

            if new_timer <= 0 {
                // Invincibility expired
                effects.push(Effect::set_component(
                    entity.id.clone(),
                    "invincible",
                    Value::Bool(false),
                ));
                effects.push(Effect::set_component(
                    entity.id.clone(),
                    "invincible_timer",
                    Value::Int(0),
                ));
            } else {
                // Update timer
                effects.push(Effect::set_component(
                    entity.id.clone(),
                    "invincible_timer",
                    Value::Int(new_timer),
                ));
            }
        }

        effects
    }

    /// Update projectile movement - move projectiles in their direction, despawn on wall hit.
    fn update_projectiles(
        world_state: &WorldState,
        room: &Room,
        tileset: &TileSet,
        delta_s: f32,
    ) -> Vec<Effect> {
        let mut effects = Vec::new();

        for (key, entity) in world_state.entities.iter() {
            // Check if entity has "projectile" tag
            if !entity.tags.iter().any(|t| t.as_str() == "projectile") {
                continue;
            }

            // Get projectile properties
            let x = world_state
                .entities
                .get_component(key, "position_x")
                .and_then(|v| v.as_float())
                .unwrap_or(0.0) as f32;
            let y = world_state
                .entities
                .get_component(key, "position_y")
                .and_then(|v| v.as_float())
                .unwrap_or(0.0) as f32;

            let direction = world_state
                .entities
                .get_component(key, "direction")
                .and_then(|v| v.as_str())
                .unwrap_or("south")
                .to_string();

            let speed = world_state
                .entities
                .get_component(key, "speed")
                .and_then(|v| v.as_float())
                .unwrap_or(100.0) as f32;

            // Get hitbox size for collision (default to small)
            let hitbox_w = world_state
                .entities
                .get_component(key, "hitbox_width")
                .and_then(|v| v.as_float())
                .unwrap_or(8.0) as f32;
            let hitbox_h = world_state
                .entities
                .get_component(key, "hitbox_height")
                .and_then(|v| v.as_float())
                .unwrap_or(8.0) as f32;

            // Calculate velocity from direction
            let (dx, dy) = Self::direction_to_velocity(&direction, speed, delta_s);

            // Check if projectile can move to new position
            let (resolved_dx, resolved_dy) =
                resolve_movement(room, tileset, x, y, hitbox_w, hitbox_h, dx, dy);

            // If movement was blocked (resolved != intended), despawn projectile
            if (resolved_dx - dx).abs() > 0.001 || (resolved_dy - dy).abs() > 0.001 {
                // Hit a wall, despawn
                effects.push(Effect::despawn(entity.id.clone()));
            } else if dx != 0.0 || dy != 0.0 {
                // Move the projectile
                effects.push(Effect::move_by(entity.id.clone(), dx, dy));
            }
        }

        effects
    }

    /// Convert direction string to velocity components.
    fn direction_to_velocity(direction: &str, speed: f32, delta_s: f32) -> (f32, f32) {
        let velocity = speed * delta_s;
        match direction {
            "north" => (0.0, -velocity),
            "south" => (0.0, velocity),
            "east" => (velocity, 0.0),
            "west" => (-velocity, 0.0),
            "northeast" => {
                let diag = velocity * 0.7071; // 1/sqrt(2)
                (diag, -diag)
            }
            "northwest" => {
                let diag = velocity * 0.7071;
                (-diag, -diag)
            }
            "southeast" => {
                let diag = velocity * 0.7071;
                (diag, diag)
            }
            "southwest" => {
                let diag = velocity * 0.7071;
                (-diag, diag)
            }
            _ => (0.0, 0.0),
        }
    }

    /// Get player position from world state.
    fn get_player_position(
        &self,
        game_state: &GameLoopState,
        world_state: &WorldState,
    ) -> (f32, f32) {
        let key = game_state
            .player_key
            .or_else(|| world_state.entities.key_of(&game_state.player_id));

        if let Some(key) = key {
            let x = world_state
                .entities
                .get_component(key, "position_x")
                .and_then(|v| v.as_float())
                .unwrap_or(0.0) as f32;
            let y = world_state
                .entities
                .get_component(key, "position_y")
                .and_then(|v| v.as_float())
                .unwrap_or(0.0) as f32;
            (x, y)
        } else {
            (0.0, 0.0)
        }
    }

    /// Get entity speed from component, falling back to config default.
    fn get_entity_speed(&self, game_state: &GameLoopState, world_state: &WorldState) -> f32 {
        let key = game_state
            .player_key
            .or_else(|| world_state.entities.key_of(&game_state.player_id));

        key.and_then(|k| {
            world_state
                .entities
                .get_component(k, "speed")
                .and_then(|v| v.as_float())
                .map(|f| f as f32)
        })
        .unwrap_or(self.config.player_speed)
    }

    /// Get entity hitbox dimensions from components, falling back to config defaults.
    fn get_entity_hitbox(&self, game_state: &GameLoopState, world_state: &WorldState) -> (f32, f32) {
        let key = game_state
            .player_key
            .or_else(|| world_state.entities.key_of(&game_state.player_id));

        let Some(key) = key else {
            return (self.config.hitbox_width, self.config.hitbox_height);
        };

        let width = world_state
            .entities
            .get_component(key, "hitbox_width")
            .and_then(|v| v.as_float())
            .map(|f| f as f32)
            .unwrap_or(self.config.hitbox_width);

        let height = world_state
            .entities
            .get_component(key, "hitbox_height")
            .and_then(|v| v.as_float())
            .map(|f| f as f32)
            .unwrap_or(self.config.hitbox_height);

        (width, height)
    }

    /// Calculate player position after applying pending effects.
    fn get_player_position_after_effects(
        &self,
        mut x: f32,
        mut y: f32,
        effects: &[Effect],
        player_id: &EntityId,
    ) -> (f32, f32) {
        for effect in effects {
            match effect {
                Effect::SetPosition { entity, x: ex, y: ey } if entity == player_id => {
                    x = *ex;
                    y = *ey;
                }
                Effect::MoveBy { entity, dx, dy } if entity == player_id => {
                    x += dx;
                    y += dy;
                }
                _ => {}
            }
        }
        (x, y)
    }

    /// Check if player has moved to screen edge and trigger transition.
    fn check_screen_transition(
        &mut self,
        game_state: &mut GameLoopState,
        player_x: f32,
        player_y: f32,
    ) -> Option<Effect> {
        let room = game_state.current_room()?;
        let tile_size = self.config.tile_size;
        let room_width = room.width as f32 * tile_size as f32;
        let room_height = room.height as f32 * tile_size as f32;

        // Check screen edges
        let direction = if player_y < 0.0 {
            Some(Direction::North)
        } else if player_y + self.config.hitbox_height > room_height {
            Some(Direction::South)
        } else if player_x < 0.0 {
            Some(Direction::West)
        } else if player_x + self.config.hitbox_width > room_width {
            Some(Direction::East)
        } else {
            None
        };

        let direction = direction?;

        // Check if there's an adjacent room
        let next_room = game_state.world.adjacent_room(&game_state.current_room_id, direction)?;
        let next_room_id = next_room.id.clone();

        // Calculate spawn position in new room
        let (spawn_x, spawn_y) = match direction {
            Direction::North => (player_x, room_height - self.config.hitbox_height - 1.0),
            Direction::South => (player_x, 1.0),
            Direction::West => (room_width - self.config.hitbox_width - 1.0, player_y),
            Direction::East => (1.0, player_y),
        };

        // Start screen transition
        let transition = ScreenTransition::new(
            game_state.current_room_id.clone(),
            next_room_id.clone(),
            direction,
            self.config.transition_duration_ms,
        );
        self.renderer.start_screen_transition(transition);

        game_state.transitioning = true;
        game_state.transition_target = Some((next_room_id.clone(), spawn_x, spawn_y));

        Some(Effect::change_room(next_room_id, spawn_x, spawn_y))
    }

    /// Render the current frame.
    pub fn render(&self, game_state: &GameLoopState, world_state: &WorldState) {
        self.renderer.begin_frame();

        // Draw room(s)
        if let Some(transition) = self.renderer.current_transition() {
            // During transition, draw both rooms
            if let (Some(old_room), Some(new_room)) = (
                game_state.world.room(&transition.from_room),
                game_state.world.room(&transition.to_room),
            ) {
                self.renderer
                    .draw_transition(old_room, new_room, self.config.tile_size);
            }
        } else {
            // Normal rendering - just current room
            if let Some(room) = game_state.current_room() {
                self.renderer.draw_room(room, 0.0, 0.0);
            }
        }

        // Draw entities (including player)
        self.render_entities(game_state, world_state);

        self.renderer.end_frame();
    }

    /// Render all entities in the current room.
    fn render_entities(&self, game_state: &GameLoopState, world_state: &WorldState) {
        // For now, just render entities with position and sprite components
        for (key, entity) in world_state.entities.iter() {
            let Some(x) = world_state
                .entities
                .get_component(key, "position_x")
                .and_then(|v| v.as_float())
            else {
                continue;
            };
            let Some(y) = world_state
                .entities
                .get_component(key, "position_y")
                .and_then(|v| v.as_float())
            else {
                continue;
            };

            // Get sprite info
            let sprite_sheet = world_state
                .entities
                .get_component(key, "sprite_sheet")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            let sprite_index = world_state
                .entities
                .get_component(key, "sprite_index")
                .and_then(|v| v.as_int())
                .unwrap_or(0) as u16;

            // Draw sprite
            self.renderer
                .draw_sprite(sprite_sheet, sprite_index, x as f32, y as f32);
        }
    }

    /// Run the game loop until window is closed.
    ///
    /// This is a convenience method for simple games. For more control,
    /// use `update` and `render` separately.
    pub async fn run<F>(
        &mut self,
        game_state: &mut GameLoopState,
        world_state: &mut WorldState,
        mut apply_effects: F,
    ) where
        F: FnMut(&mut WorldState, Vec<Effect>),
    {
        // Cache player key
        game_state.cache_player_key(world_state);

        loop {
            // Update
            let effects = self.update(game_state, world_state);
            if !effects.is_empty() {
                apply_effects(world_state, effects);
            }

            // Render
            self.render(game_state, world_state);

            // Check for exit
            if is_key_pressed(KeyCode::Escape) {
                break;
            }

            next_frame().await;
        }
    }
}

/// Helper to check if a position is walkable in the current room.
pub fn is_position_walkable(
    room: &Room,
    tileset: &TileSet,
    x: f32,
    y: f32,
    hitbox_w: f32,
    hitbox_h: f32,
) -> bool {
    can_place(room, tileset, x, y, hitbox_w, hitbox_h)
}
