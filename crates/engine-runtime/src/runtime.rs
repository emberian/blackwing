use engine_core::{Command, Event, GameConfig, GameSchema, GameState};

use crate::{CommandHandler, ContentRegistry, Rng, RuntimeError};

pub struct Runtime<'a> {
    schema: &'a GameSchema,
    registry: &'a dyn ContentRegistry,
    config: GameConfig,
    state: GameState,
    event_log: Vec<Event>,
    rng: Rng,
}

impl<'a> Runtime<'a> {
    pub fn new(schema: &'a GameSchema, registry: &'a dyn ContentRegistry, seed: u64) -> Self {
        Self::with_config(schema, registry, GameConfig::default(), seed)
    }

    pub fn with_config(
        schema: &'a GameSchema,
        registry: &'a dyn ContentRegistry,
        config: GameConfig,
        seed: u64,
    ) -> Self {
        let state = GameState::from_schema(schema, seed);
        Self {
            schema,
            registry,
            config,
            state,
            event_log: Vec::new(),
            rng: Rng::new(seed),
        }
    }

    pub fn schema(&self) -> &GameSchema {
        self.schema
    }

    pub fn config(&self) -> &GameConfig {
        &self.config
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn event_log(&self) -> &[Event] {
        &self.event_log
    }

    pub fn rng(&self) -> &Rng {
        &self.rng
    }

    pub fn rng_mut(&mut self) -> &mut Rng {
        &mut self.rng
    }

    pub fn dispatch(&mut self, command: Command) -> Result<Vec<Event>, RuntimeError> {
        let handler = CommandHandler::new(self.schema, self.registry, &self.config);
        let events = handler.handle(&self.state, &command, &mut self.rng)?;

        let mut game_over = None;
        for event in &events {
            if let Some(reason) = self.state.apply_event(event, self.schema) {
                game_over = Some(reason);
            }
        }

        self.event_log.extend(events.clone());

        if let Some(reason) = game_over {
            return Err(RuntimeError::GameOver {
                reason: format!("{:?}", reason),
            });
        }

        Ok(events)
    }

    pub fn replay(&mut self, events: &[Event]) {
        for event in events {
            self.state.apply_event(event, self.schema);
            self.event_log.push(event.clone());
        }
    }

    pub fn fork(&self) -> Runtime<'a> {
        Runtime {
            schema: self.schema,
            registry: self.registry,
            config: self.config.clone(),
            state: self.state.clone(),
            event_log: self.event_log.clone(),
            rng: Rng::new(self.rng.state()),
        }
    }
}
