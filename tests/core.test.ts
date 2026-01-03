import { describe, it, expect, beforeEach } from 'vitest';
import { createInitialState } from '../src/core/init.js';
import { 
  calculateJourneyEventCount, 
  calculateFuelCost,
  processJourneyWear,
  processCargoDecay,
  tickContractTimers,
  advanceCycle,
  incrementJumps,
} from '../src/core/simulate.js';
import { dispatch } from '../src/core/actions.js';
import { buildCardDefMap } from '../src/content/cards/index.js';
import { DEFAULT_CONFIG, createId } from '../src/core/types.js';
import type { GameState, CardDef, CardInstance, CardInstanceId } from '../src/core/types.js';

describe('Game Core', () => {
  let state: GameState;
  let cardDefs: Map<string, CardDef>;

  beforeEach(() => {
    state = createInitialState();
    cardDefs = buildCardDefMap();
  });

  describe('Initial State', () => {
    it('creates valid initial state', () => {
      expect(state.schemaVersion).toBe(2);
      expect(state.resources.credits).toBe(80);
      expect(state.ship.name).toBe('Holdfast');
      expect(state.chronicle.length).toBe(1);
    });

    it('starts at Haven Prime', () => {
      expect(state.world.currentLocation).toBe(createId.port('port_haven_prime'));
    });

    it('has correct initial resources', () => {
      expect(state.resources).toEqual({
        credits: 80,
        fuel: 40,
        supplies: 25,
        hull: 100,
        morale: 70,
      });
    });

    it('has correct initial time state', () => {
      expect(state.time.cycle).toBe(0);
      expect(state.time.jumpsCompleted).toBe(0);
    });
  });

  describe('Journey Simulation', () => {
    it('calculates journey event count within config bounds', () => {
      const eventCount = calculateJourneyEventCount(state, createId.port('port_frontier_station'), DEFAULT_CONFIG);
      expect(eventCount).toBeGreaterThanOrEqual(DEFAULT_CONFIG.journeyEventCount.min);
      expect(eventCount).toBeLessThanOrEqual(DEFAULT_CONFIG.journeyEventCount.max);
    });

    it('calculates fuel cost with base config', () => {
      const fuelCost = calculateFuelCost(state, cardDefs, DEFAULT_CONFIG);
      expect(fuelCost).toBe(DEFAULT_CONFIG.baseFuelPerJump);
    });

    it('reduces fuel cost with efficient drives module', () => {
      // Add an efficient drives module to the ship
      const moduleId = 'test-module' as CardInstanceId;
      const moduleInstance: CardInstance = {
        instanceId: moduleId,
        cardDefId: createId.cardDef('module_efficient_drives'),
        level: 1,
        condition: 100,
        mods: [],
        acquiredAt: { cycle: 0 },
      };
      
      state = {
        ...state,
        cards: {
          ...state.cards,
          instances: { [moduleId]: moduleInstance },
        },
        ship: {
          ...state.ship,
          modules: {
            ...state.ship.modules,
            propulsion: moduleId,
          },
        },
      };
      
      const fuelCost = calculateFuelCost(state, cardDefs, DEFAULT_CONFIG);
      expect(fuelCost).toBeLessThan(DEFAULT_CONFIG.baseFuelPerJump);
    });

    it('processes journey wear - consumes supplies and damages hull', () => {
      const initialSupplies = state.resources.supplies;
      const initialHull = state.resources.hull;
      
      const newState = processJourneyWear(state, DEFAULT_CONFIG);
      
      expect(newState.resources.supplies).toBeLessThan(initialSupplies);
      expect(newState.resources.hull).toBeLessThan(initialHull);
    });

    it('reduces morale when supplies run out', () => {
      state = {
        ...state,
        resources: { ...state.resources, supplies: 1 },
      };
      
      const newState = processJourneyWear(state, DEFAULT_CONFIG);
      
      expect(newState.resources.supplies).toBe(0);
      expect(newState.resources.morale).toBeLessThan(state.resources.morale);
    });

    it('advances cycle', () => {
      const newState = advanceCycle(state);
      expect(newState.time.cycle).toBe(state.time.cycle + 1);
    });

    it('increments jumps', () => {
      const newState = incrementJumps(state);
      expect(newState.time.jumpsCompleted).toBe(state.time.jumpsCompleted + 1);
    });
  });

  describe('Cargo Decay', () => {
    it('can decay cargo with decayChance', () => {
      // Add a cargo with decay chance
      const cargoId = 'test-cargo' as CardInstanceId;
      const cargoInstance: CardInstance = {
        instanceId: cargoId,
        cardDefId: createId.cardDef('cargo_cryo_seeds'), // Has 15% decay chance
        level: 1,
        condition: 100,
        mods: [],
        acquiredAt: { cycle: 0 },
      };
      
      state = {
        ...state,
        cards: {
          ...state.cards,
          instances: { [cargoId]: cargoInstance },
          collection: [cargoId],
        },
      };
      
      // Run decay many times - should eventually decay
      let hasDecayed = false;
      for (let i = 0; i < 100; i++) {
        const result = processCargoDecay(state, cardDefs);
        if (result.decayedCards.length > 0 || 
            (result.state.cards.instances[cargoId]?.condition ?? 0) < 100) {
          hasDecayed = true;
          break;
        }
      }
      
      expect(hasDecayed).toBe(true);
    });
  });

  describe('Contract Timers', () => {
    it('ticks down contract cycles remaining', () => {
      // Accept a contract first
      const contractResult = dispatch(
        state,
        { type: 'CONTRACT_ACCEPT', payload: { cardDefId: createId.cardDef('contract_standard_delivery') } },
        cardDefs
      );
      
      expect(contractResult.success).toBe(true);
      const contractId = contractResult.state.cards.activeContracts[0];
      const initialCycles = contractResult.state.cards.instances[contractId!]?.cyclesRemaining;
      
      // Tick the contract
      const tickResult = tickContractTimers(contractResult.state, cardDefs);
      const newCycles = tickResult.state.cards.instances[contractId!]?.cyclesRemaining;
      
      expect(newCycles).toBe((initialCycles ?? 0) - 1);
    });

    it('expires contracts when cycles reach zero', () => {
      // Accept a contract
      const contractResult = dispatch(
        state,
        { type: 'CONTRACT_ACCEPT', payload: { cardDefId: createId.cardDef('contract_standard_delivery') } },
        cardDefs
      );
      
      const contractId = contractResult.state.cards.activeContracts[0]!;
      
      // Set cycles to 1 so it expires on next tick
      let testState = {
        ...contractResult.state,
        cards: {
          ...contractResult.state.cards,
          instances: {
            ...contractResult.state.cards.instances,
            [contractId]: {
              ...contractResult.state.cards.instances[contractId]!,
              cyclesRemaining: 1,
            },
          },
        },
      };
      
      const tickResult = tickContractTimers(testState, cardDefs);
      
      expect(tickResult.expiredContracts.length).toBe(1);
      expect(tickResult.state.cards.activeContracts).not.toContain(contractId);
      expect(tickResult.state.stats.contractsFailed).toBe(1);
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

      it('rejects travel to current location', () => {
        const result = dispatch(
          state,
          { type: 'TRAVEL', payload: { destination: createId.port('port_haven_prime') } },
          cardDefs
        );
        
        expect(result.success).toBe(false);
        expect(result.message).toContain('Already at this location');
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

    describe('Contracts', () => {
      it('allows accepting contracts', () => {
        const result = dispatch(
          state,
          { type: 'CONTRACT_ACCEPT', payload: { cardDefId: createId.cardDef('contract_standard_delivery') } },
          cardDefs
        );
        
        expect(result.success).toBe(true);
        expect(result.state.cards.activeContracts.length).toBe(1);
      });

      it('tracks cycles remaining on contracts', () => {
        const result = dispatch(
          state,
          { type: 'CONTRACT_ACCEPT', payload: { cardDefId: createId.cardDef('contract_standard_delivery') } },
          cardDefs
        );
        
        const contractId = result.state.cards.activeContracts[0];
        const contract = result.state.cards.instances[contractId!];
        
        expect(contract?.cyclesRemaining).toBeDefined();
        expect(contract?.cyclesRemaining).toBeGreaterThan(0);
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
    expect(navigator?.effects.modifiers?.journeySpeed).toBeDefined();
  });

  it('has valid module cards', () => {
    const cardDefs = buildCardDefMap();
    const sensor = cardDefs.get(createId.cardDef('module_sensor_array'));
    
    expect(sensor).toBeDefined();
    expect(sensor?.type).toBe('module');
    expect(sensor?.installRequirements?.slotType).toBe('sensor');
  });

  it('has valid contract cards with cycle limits', () => {
    const cardDefs = buildCardDefMap();
    const contract = cardDefs.get(createId.cardDef('contract_standard_delivery'));
    
    expect(contract).toBeDefined();
    expect(contract?.type).toBe('contract');
    expect(contract?.contractTerms?.cycleLimit).toBeGreaterThan(0);
  });
});
