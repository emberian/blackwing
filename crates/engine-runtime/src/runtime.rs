use engine_core::{Command, Event, GameConfig, GameSchema, GameState, LocationId, ResourceId};

use crate::{
    journey::{
        calculate_fuel_efficiency, process_cargo_decay, process_journey_wear, tick_contract_timers,
    },
    CommandHandler, ContentRegistry, JourneyState, Rng, RuntimeError,
};

pub struct Runtime<'a> {
    schema: &'a GameSchema,
    registry: &'a dyn ContentRegistry,
    config: GameConfig,
    state: GameState,
    event_log: Vec<Event>,
    rng: Rng,
    journey: Option<JourneyState>,
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
            journey: None,
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
            journey: self.journey.clone(),
        }
    }

    // === Journey Methods ===

    /// Start a journey to a destination
    pub fn start_journey(
        &mut self,
        destination: LocationId,
        event_count: u32,
    ) -> Result<Vec<Event>, RuntimeError> {
        // Calculate fuel cost with efficiency
        let efficiency = calculate_fuel_efficiency(&self.state, self.registry);
        let fuel_cost = self.config.fuel_cost_with_efficiency(efficiency);

        let fuel_id = ResourceId::new("fuel");
        let current_fuel = self.state.resource(&fuel_id);

        if current_fuel < fuel_cost {
            return Err(RuntimeError::InsufficientFuel {
                required: fuel_cost,
                available: current_fuel,
            });
        }

        // Consume fuel
        let events = vec![Event::resource_changed(
            fuel_id,
            current_fuel,
            current_fuel - fuel_cost,
            "journey fuel",
        )];

        // Set journey state
        self.journey = Some(JourneyState::new(destination, event_count));

        // Apply events
        for event in &events {
            self.state.apply_event(event, self.schema);
        }
        self.event_log.extend(events.clone());

        Ok(events)
    }

    /// Process one tick of the journey (wear, decay, contract timers)
    pub fn process_journey_tick(&mut self) -> Result<Vec<Event>, RuntimeError> {
        let journey = self
            .journey
            .as_mut()
            .ok_or(RuntimeError::NoJourneyInProgress)?;

        let mut events = Vec::new();

        // Process wear (supplies, hull)
        events.extend(process_journey_wear(&self.state, &self.config));

        // Process cargo decay
        let (decay_events, _decayed) =
            process_cargo_decay(&self.state, self.registry, &mut self.rng);
        events.extend(decay_events);

        // Tick contract timers
        let (contract_events, _expired) = tick_contract_timers(&self.state, self.registry);
        events.extend(contract_events);

        // Consume journey event
        journey.consume_event();

        // Apply all events
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

    /// Complete the journey, arriving at destination
    pub fn complete_journey(&mut self) -> Result<Vec<Event>, RuntimeError> {
        let journey = self
            .journey
            .take()
            .ok_or(RuntimeError::NoJourneyInProgress)?;

        let events = vec![Event::location_changed(
            self.state.location.clone(),
            journey.destination,
        )];

        for event in &events {
            self.state.apply_event(event, self.schema);
        }
        self.event_log.extend(events.clone());

        Ok(events)
    }

    /// Check if a journey is in progress
    pub fn is_journeying(&self) -> bool {
        self.journey.is_some()
    }

    /// Get current journey state
    pub fn journey(&self) -> Option<&JourneyState> {
        self.journey.as_ref()
    }
}
