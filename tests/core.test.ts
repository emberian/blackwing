import { describe, it, expect, beforeEach } from 'vitest';
import { createInitialState } from '../src/core/init.js';
import { simulateOffline, compressEvents } from '../src/core/simulate.js';
import { dispatch } from '../src/core/actions.js';
import { buildCardDefMap } from '../src/content/cards/index.js';
import { DEFAULT_CONFIG, createId } from '../src/core/types.js';
import type { GameState, CardDef } from '../src/core/types.js';

describe('Game Core', () => {
  let state: GameState;
  let cardDefs: Map<string, CardDef>;

  beforeEach(() => {
    state = createInitialState();
    cardDefs = buildCardDefMap();
  });

  describe('Initial State', () => {
    it('creates valid initial state', () => {
      expect(state.schemaVersion).toBe(1);
      expect(state.resources.credits).toBe(100);
      expect(state.ship.name).toBe('Holdfast');
      expect(state.chronicle.length).toBe(1);
    });

    it('starts at Haven Prime', () => {
      expect(state.world.currentLocation).toBe(createId.port('port_haven_prime'));
    });

    it('has correct initial resources', () => {
      expect(state.resources).toEqual({
        credits: 100,
        fuel: 50,
        supplies: 30,
        hull: 100,
        morale: 75,
      });
    });
  });

  describe('Simulation', () => {
    it('handles zero tick simulation', () => {
      const result = simulateOffline(state, cardDefs, state.time.lastSimulatedAt);
      expect(result.ticksSimulated).toBe(0);
      expect(result.events).toHaveLength(0);
    });

    it('simulates time passage', () => {
      const futureTime = state.time.lastSimulatedAt + DEFAULT_CONFIG.msPerTick * 5;
      const result = simulateOffline(state, cardDefs, futureTime);
      
      expect(result.ticksSimulated).toBe(5);
      expect(result.state.time.ticks).toBe(5);
    });

    it('consumes supplies over time', () => {
      const futureTime = state.time.lastSimulatedAt + DEFAULT_CONFIG.msPerTick * 10;
      const result = simulateOffline(state, cardDefs, futureTime);
      
      expect(result.state.resources.supplies).toBeLessThan(state.resources.supplies);
    });

    it('caps catch-up ticks', () => {
      const veryFuture = state.time.lastSimulatedAt + DEFAULT_CONFIG.msPerTick * 10000;
      const result = simulateOffline(state, cardDefs, veryFuture);
      
      expect(result.ticksSimulated).toBe(DEFAULT_CONFIG.maxCatchUpTicks);
    });

    it('compresses resource events', () => {
      const events = [
        { type: 'resource_change' as const, resource: 'credits' as const, delta: 10, reason: 'a' },
        { type: 'resource_change' as const, resource: 'credits' as const, delta: 5, reason: 'b' },
        { type: 'resource_change' as const, resource: 'fuel' as const, delta: -2, reason: 'c' },
        { type: 'arrival' as const, portId: createId.port('test') },
      ];
      
      const compressed = compressEvents(events);
      const creditEvents = compressed.filter(e => e.type === 'resource_change' && e.resource === 'credits');
      const fuelEvents = compressed.filter(e => e.type === 'resource_change' && e.resource === 'fuel');
      
      expect(creditEvents).toHaveLength(1);
      expect(creditEvents[0]!.type === 'resource_change' && creditEvents[0]!.delta).toBe(15);
      expect(fuelEvents).toHaveLength(1);
    });
  });

  describe('Actions', () => {
    describe('Trade', () => {
      it('allows buying available cargo', () => {
        const result = dispatch(
          state,
          { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
          cardDefs
        );
        
        expect(result.success).toBe(true);
        expect(result.state.resources.credits).toBeLessThan(state.resources.credits);
        expect(Object.keys(result.state.cards.instances).length).toBe(1);
      });

      it('rejects buying unavailable cargo', () => {
        const result = dispatch(
          state,
          { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_ancient_artifacts'), quantity: 1 } },
          cardDefs
        );
        
        expect(result.success).toBe(false);
        expect(result.message).toContain('Not available');
      });

      it('rejects purchase without funds', () => {
        state.resources.credits = 0;
        const result = dispatch(
          state,
          { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
          cardDefs
        );
        
        expect(result.success).toBe(false);
        expect(result.message).toContain('Insufficient credits');
      });
    });

    describe('Travel', () => {
      it('allows travel to known destination', () => {
        const result = dispatch(
          state,
          { type: 'TRAVEL', payload: { destination: createId.port('port_frontier_station') } },
          cardDefs
        );
        
        expect(result.success).toBe(true);
        expect(result.state.time.inTransit).toBe(true);
        expect(result.state.resources.fuel).toBeLessThan(state.resources.fuel);
      });

      it('rejects travel without fuel', () => {
        state.resources.fuel = 0;
        const result = dispatch(
          state,
          { type: 'TRAVEL', payload: { destination: createId.port('port_frontier_station') } },
          cardDefs
        );
        
        expect(result.success).toBe(false);
        expect(result.message).toContain('Insufficient fuel');
      });

      it('rejects travel while in transit', () => {
        state.time.inTransit = true;
        const result = dispatch(
          state,
          { type: 'TRAVEL', payload: { destination: createId.port('port_frontier_station') } },
          cardDefs
        );
        
        expect(result.success).toBe(false);
        expect(result.message).toContain('Already in transit');
      });
    });

    describe('Crew', () => {
      it('allows hiring available crew', () => {
        const result = dispatch(
          state,
          { type: 'CREW_HIRE', payload: { cardDefId: createId.cardDef('crew_navigator') } },
          cardDefs
        );
        
        expect(result.success).toBe(true);
        expect(result.state.cards.activeCrew.length).toBe(1);
        expect(result.state.cards.deck.length).toBe(1);
      });
    });

    describe('Repair & Resupply', () => {
      it('allows repair when has credits', () => {
        state.resources.hull = 50;
        const result = dispatch(
          state,
          { type: 'REPAIR', payload: { amount: 10 } },
          cardDefs
        );
        
        expect(result.success).toBe(true);
        expect(result.state.resources.hull).toBe(60);
        expect(result.state.resources.credits).toBeLessThan(state.resources.credits);
      });

      it('allows refueling', () => {
        state.resources.fuel = 10;
        const result = dispatch(
          state,
          { type: 'REFUEL', payload: { amount: 20 } },
          cardDefs
        );
        
        expect(result.success).toBe(true);
        expect(result.state.resources.fuel).toBe(30);
      });
    });
  });
});

describe('Card Definitions', () => {
  it('loads all card definitions', () => {
    const cardDefs = buildCardDefMap();
    expect(cardDefs.size).toBeGreaterThan(20);
  });

  it('has valid cargo cards', () => {
    const cardDefs = buildCardDefMap();
    const rawOre = cardDefs.get(createId.cardDef('cargo_raw_ore'));
    
    expect(rawOre).toBeDefined();
    expect(rawOre?.type).toBe('cargo');
    expect(rawOre?.baseValue).toBeGreaterThan(0);
  });

  it('has valid crew cards', () => {
    const cardDefs = buildCardDefMap();
    const navigator = cardDefs.get(createId.cardDef('crew_navigator'));
    
    expect(navigator).toBeDefined();
    expect(navigator?.type).toBe('crew');
    expect(navigator?.lifespan).toBeGreaterThan(0);
  });

  it('has valid module cards', () => {
    const cardDefs = buildCardDefMap();
    const sensor = cardDefs.get(createId.cardDef('module_sensor_array'));
    
    expect(sensor).toBeDefined();
    expect(sensor?.type).toBe('module');
    expect(sensor?.installRequirements?.slotType).toBe('sensor');
  });
});
