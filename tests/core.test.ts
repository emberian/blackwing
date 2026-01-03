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
        fuel: 35,
        supplies: 20,
        hull: 100,
        morale: 75,
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
      expect(fuelCost).toBe(10);
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
      const cargoId = 'test-cargo' as CardInstanceId;
      const cargoInstance: CardInstance = {
        instanceId: cargoId,
        cardDefId: createId.cardDef('cargo_cryo_seeds'),
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
      
      let hasDecayed = false;
      for (let i = 0; i < 100; i++) {
        const rng = () => Math.random();
        const result = processCargoDecay(state, cardDefs, rng);
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

describe('Advanced Actions', () => {
  let state: GameState;
  let cardDefs: Map<string, CardDef>;

  beforeEach(() => {
    state = createInitialState();
    cardDefs = buildCardDefMap();
  });

  describe('Module Install/Uninstall', () => {
    it('installs module in empty slot', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('module_sensor_array'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      
      const installResult = dispatch(
        buyResult.state,
        { type: 'MODULE_INSTALL', payload: { instanceId, slot: 'sensor' } },
        cardDefs
      );
      
      expect(installResult.success).toBe(true);
      expect(installResult.state.ship.modules.sensor).toBe(instanceId);
      expect(installResult.state.cards.collection).not.toContain(instanceId);
    });

    it('rejects installing non-module card', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      
      const installResult = dispatch(
        buyResult.state,
        { type: 'MODULE_INSTALL', payload: { instanceId, slot: 'sensor' } },
        cardDefs
      );
      
      expect(installResult.success).toBe(false);
      expect(installResult.message).toContain('Not a module');
    });

    it('rejects installing in occupied slot', () => {
      state = { ...state, resources: { ...state.resources, credits: 500 } };
      
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('module_sensor_array'), quantity: 2 } },
        cardDefs
      );
      
      expect(buyResult.success).toBe(true);
      
      const [id1, id2] = buyResult.state.cards.collection;
      
      const installResult1 = dispatch(
        buyResult.state,
        { type: 'MODULE_INSTALL', payload: { instanceId: id1!, slot: 'sensor' } },
        cardDefs
      );
      
      expect(installResult1.success).toBe(true);
      expect(installResult1.state.cards.collection).toContain(id2);
      
      const installResult2 = dispatch(
        installResult1.state,
        { type: 'MODULE_INSTALL', payload: { instanceId: id2!, slot: 'sensor' } },
        cardDefs
      );
      
      expect(installResult2.success).toBe(false);
      expect(installResult2.message).toContain('Slot occupied');
    });

    it('uninstalls module from slot', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('module_sensor_array'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      
      const installResult = dispatch(
        buyResult.state,
        { type: 'MODULE_INSTALL', payload: { instanceId, slot: 'sensor' } },
        cardDefs
      );
      
      const uninstallResult = dispatch(
        installResult.state,
        { type: 'MODULE_UNINSTALL', payload: { slot: 'sensor' } },
        cardDefs
      );
      
      expect(uninstallResult.success).toBe(true);
      expect(uninstallResult.state.ship.modules.sensor).toBeNull();
      expect(uninstallResult.state.cards.collection).toContain(instanceId);
    });

    it('rejects uninstalling from empty slot', () => {
      const result = dispatch(
        state,
        { type: 'MODULE_UNINSTALL', payload: { slot: 'sensor' } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Slot empty');
    });

    it('rejects installing nonexistent module', () => {
      const result = dispatch(
        state,
        { type: 'MODULE_INSTALL', payload: { instanceId: 'nonexistent' as CardInstanceId, slot: 'sensor' } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Module not found');
    });
  });

  describe('Card Equip/Unequip', () => {
    it('equips card from collection to deck', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      
      const equipResult = dispatch(
        buyResult.state,
        { type: 'CARD_EQUIP', payload: { instanceId } },
        cardDefs
      );
      
      expect(equipResult.success).toBe(true);
      expect(equipResult.state.cards.deck).toContain(instanceId);
      expect(equipResult.state.cards.collection).not.toContain(instanceId);
    });

    it('rejects equipping already equipped card', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      
      const equipResult = dispatch(
        buyResult.state,
        { type: 'CARD_EQUIP', payload: { instanceId } },
        cardDefs
      );
      
      const doubleEquipResult = dispatch(
        equipResult.state,
        { type: 'CARD_EQUIP', payload: { instanceId } },
        cardDefs
      );
      
      expect(doubleEquipResult.success).toBe(false);
      expect(doubleEquipResult.message).toContain('Already equipped');
    });

    it('unequips card from deck to collection', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      
      const equipResult = dispatch(
        buyResult.state,
        { type: 'CARD_EQUIP', payload: { instanceId } },
        cardDefs
      );
      
      const unequipResult = dispatch(
        equipResult.state,
        { type: 'CARD_UNEQUIP', payload: { instanceId } },
        cardDefs
      );
      
      expect(unequipResult.success).toBe(true);
      expect(unequipResult.state.cards.collection).toContain(instanceId);
      expect(unequipResult.state.cards.deck).not.toContain(instanceId);
    });

    it('adds crew to activeCrew when equipping', () => {
      const hireResult = dispatch(
        state,
        { type: 'CREW_HIRE', payload: { cardDefId: createId.cardDef('crew_navigator') } },
        cardDefs
      );
      
      expect(hireResult.state.cards.activeCrew.length).toBe(1);
    });

    it('removes crew from activeCrew when unequipping', () => {
      const hireResult = dispatch(
        state,
        { type: 'CREW_HIRE', payload: { cardDefId: createId.cardDef('crew_navigator') } },
        cardDefs
      );
      
      const crewId = hireResult.state.cards.activeCrew[0]!;
      
      const unequipResult = dispatch(
        hireResult.state,
        { type: 'CARD_UNEQUIP', payload: { instanceId: crewId } },
        cardDefs
      );
      
      expect(unequipResult.state.cards.activeCrew).not.toContain(crewId);
    });
  });

  describe('Trade Sell', () => {
    it('sells cargo and adds credits', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      const creditsAfterBuy = buyResult.state.resources.credits;
      
      const sellResult = dispatch(
        buyResult.state,
        { type: 'TRADE_SELL', payload: { instanceId } },
        cardDefs
      );
      
      expect(sellResult.success).toBe(true);
      expect(sellResult.state.resources.credits).toBeGreaterThan(creditsAfterBuy);
      expect(sellResult.state.cards.instances[instanceId]).toBeUndefined();
    });

    it('removes sold card from collection', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      
      const sellResult = dispatch(
        buyResult.state,
        { type: 'TRADE_SELL', payload: { instanceId } },
        cardDefs
      );
      
      expect(sellResult.state.cards.collection).not.toContain(instanceId);
    });

    it('rejects selling nonexistent card', () => {
      const result = dispatch(
        state,
        { type: 'TRADE_SELL', payload: { instanceId: 'nonexistent' as CardInstanceId } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Card not found');
    });

    it('updates totalCreditsEarned stat on sell', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      const initialEarned = buyResult.state.stats.totalCreditsEarned;
      
      const sellResult = dispatch(
        buyResult.state,
        { type: 'TRADE_SELL', payload: { instanceId } },
        cardDefs
      );
      
      expect(sellResult.state.stats.totalCreditsEarned).toBeGreaterThan(initialEarned);
    });
  });

  describe('Contract Complete', () => {
    it('requires being at contract destination', () => {
      const acceptResult = dispatch(
        state,
        { type: 'CONTRACT_ACCEPT', payload: { cardDefId: createId.cardDef('contract_standard_delivery') } },
        cardDefs
      );
      
      const contractId = acceptResult.state.cards.activeContracts[0]!;
      
      const completeResult = dispatch(
        acceptResult.state,
        { type: 'CONTRACT_COMPLETE', payload: { instanceId: contractId } },
        cardDefs
      );
      
      expect(completeResult.success).toBe(false);
      expect(completeResult.message).toContain('Must be at contract destination');
    });

    it('rejects completing nonexistent contract', () => {
      const result = dispatch(
        state,
        { type: 'CONTRACT_COMPLETE', payload: { instanceId: 'nonexistent' as CardInstanceId } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Contract not found');
    });
  });

  describe('Contract Abandon', () => {
    it('abandons contract and applies penalty', () => {
      const acceptResult = dispatch(
        state,
        { type: 'CONTRACT_ACCEPT', payload: { cardDefId: createId.cardDef('contract_standard_delivery') } },
        cardDefs
      );
      
      const contractId = acceptResult.state.cards.activeContracts[0]!;
      const initialMorale = acceptResult.state.resources.morale;
      
      const abandonResult = dispatch(
        acceptResult.state,
        { type: 'CONTRACT_ABANDON', payload: { instanceId: contractId } },
        cardDefs
      );
      
      expect(abandonResult.success).toBe(true);
      expect(abandonResult.state.cards.activeContracts).not.toContain(contractId);
      expect(abandonResult.state.resources.morale).toBeLessThan(initialMorale);
      expect(abandonResult.state.stats.contractsFailed).toBe(1);
    });

    it('rejects abandoning nonexistent contract', () => {
      const result = dispatch(
        state,
        { type: 'CONTRACT_ABANDON', payload: { instanceId: 'nonexistent' as CardInstanceId } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Contract not found');
    });
  });

  describe('Crew Dismiss', () => {
    it('dismisses crew and reduces morale', () => {
      const hireResult = dispatch(
        state,
        { type: 'CREW_HIRE', payload: { cardDefId: createId.cardDef('crew_navigator') } },
        cardDefs
      );
      
      const crewId = hireResult.state.cards.activeCrew[0]!;
      const initialMorale = hireResult.state.resources.morale;
      
      const dismissResult = dispatch(
        hireResult.state,
        { type: 'CREW_DISMISS', payload: { instanceId: crewId } },
        cardDefs
      );
      
      expect(dismissResult.success).toBe(true);
      expect(dismissResult.state.cards.activeCrew).not.toContain(crewId);
      expect(dismissResult.state.cards.deck).not.toContain(crewId);
      expect(dismissResult.state.cards.instances[crewId]).toBeUndefined();
      expect(dismissResult.state.resources.morale).toBeLessThan(initialMorale);
    });

    it('rejects dismissing nonexistent crew', () => {
      const result = dispatch(
        state,
        { type: 'CREW_DISMISS', payload: { instanceId: 'nonexistent' as CardInstanceId } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Crew member not found');
    });
  });

  describe('Crew Hire', () => {
    it('rejects hiring without sufficient credits', () => {
      state = { ...state, resources: { ...state.resources, credits: 0 } };
      
      const result = dispatch(
        state,
        { type: 'CREW_HIRE', payload: { cardDefId: createId.cardDef('crew_navigator') } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Insufficient credits');
    });

    it('rejects hiring non-crew card', () => {
      const result = dispatch(
        state,
        { type: 'CREW_HIRE', payload: { cardDefId: createId.cardDef('cargo_raw_ore') } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Invalid crew type');
    });
  });

  describe('Resupply', () => {
    it('adds supplies for credits', () => {
      const initialSupplies = state.resources.supplies;
      const initialCredits = state.resources.credits;
      
      const result = dispatch(
        state,
        { type: 'RESUPPLY', payload: { amount: 10 } },
        cardDefs
      );
      
      expect(result.success).toBe(true);
      expect(result.state.resources.supplies).toBe(initialSupplies + 10);
      expect(result.state.resources.credits).toBeLessThan(initialCredits);
    });

    it('rejects resupply without sufficient credits', () => {
      state = { ...state, resources: { ...state.resources, credits: 0 } };
      
      const result = dispatch(
        state,
        { type: 'RESUPPLY', payload: { amount: 10 } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Insufficient credits');
    });
  });

  describe('Repair', () => {
    it('clamps hull to max hull', () => {
      state = { ...state, resources: { ...state.resources, hull: 95, credits: 1000 } };
      
      const result = dispatch(
        state,
        { type: 'REPAIR', payload: { amount: 50 } },
        cardDefs
      );
      
      expect(result.success).toBe(true);
      expect(result.state.resources.hull).toBe(state.ship.maxHull);
    });

    it('rejects repair without sufficient credits', () => {
      state = { ...state, resources: { ...state.resources, hull: 50, credits: 0 } };
      
      const result = dispatch(
        state,
        { type: 'REPAIR', payload: { amount: 10 } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Insufficient credits');
    });
  });

  describe('Refuel', () => {
    it('rejects refuel without sufficient credits', () => {
      state = { ...state, resources: { ...state.resources, credits: 0 } };
      
      const result = dispatch(
        state,
        { type: 'REFUEL', payload: { amount: 10 } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Insufficient credits');
    });
  });
});

describe('Resource Boundary Conditions', () => {
  let state: GameState;
  let cardDefs: Map<string, CardDef>;

  beforeEach(() => {
    state = createInitialState();
    cardDefs = buildCardDefMap();
  });

  describe('Journey Wear Resource Clamping', () => {
    it('clamps supplies to minimum of 0', () => {
      state = { ...state, resources: { ...state.resources, supplies: 1 } };
      
      const result = processJourneyWear(state, { ...DEFAULT_CONFIG, journeySupplyCost: 100 });
      
      expect(result.resources.supplies).toBe(0);
    });

    it('clamps hull to minimum of 0', () => {
      state = { ...state, resources: { ...state.resources, hull: 1 } };
      
      const result = processJourneyWear(state, { ...DEFAULT_CONFIG, journeyHullWear: 100 });
      
      expect(result.resources.hull).toBe(0);
    });

    it('clamps morale to minimum of 0 when starving', () => {
      state = { 
        ...state, 
        resources: { ...state.resources, supplies: 1, morale: 10 } 
      };
      
      const result = processJourneyWear(state, { ...DEFAULT_CONFIG, journeySupplyCost: 100 });
      
      expect(result.resources.morale).toBe(0);
    });
  });

  describe('Fuel Cost Calculation Edge Cases', () => {
    it('returns minimum fuel cost of 1', () => {
      const moduleId = 'super-efficient' as CardInstanceId;
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
          deck: [moduleId],
        },
        ship: {
          ...state.ship,
          modules: { ...state.ship.modules, propulsion: moduleId },
        },
      };
      
      const fuelCost = calculateFuelCost(state, cardDefs, DEFAULT_CONFIG);
      expect(fuelCost).toBeGreaterThanOrEqual(1);
    });
  });

  describe('Trade Buy Edge Cases', () => {
    it('buys multiple items correctly', () => {
      const quantity = 3;
      
      const result = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity } },
        cardDefs
      );
      
      expect(result.success).toBe(true);
      expect(result.state.cards.collection.length).toBe(quantity);
      expect(result.state.stats.cardsAcquired).toBe(quantity);
    });

    it('rejects unknown card type', () => {
      const result = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('nonexistent_card'), quantity: 1 } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Unknown card type');
    });
  });

  describe('Card Upgrade', () => {
    it('rejects upgrading nonexistent card', () => {
      const result = dispatch(
        state,
        { type: 'CARD_UPGRADE', payload: { instanceId: 'nonexistent' as CardInstanceId } },
        cardDefs
      );
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Card not found');
    });

    it('rejects upgrading non-upgradeable card', () => {
      const buyResult = dispatch(
        state,
        { type: 'TRADE_BUY', payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } },
        cardDefs
      );
      
      const instanceId = buyResult.state.cards.collection[0]!;
      
      const upgradeResult = dispatch(
        buyResult.state,
        { type: 'CARD_UPGRADE', payload: { instanceId } },
        cardDefs
      );
      
      expect(upgradeResult.success).toBe(false);
      expect(upgradeResult.message).toContain('Card cannot be upgraded');
    });
  });

  describe('Contract Penalty Clamping', () => {
    it('clamps credits to 0 on contract abandon with insufficient credits', () => {
      const acceptResult = dispatch(
        state,
        { type: 'CONTRACT_ACCEPT', payload: { cardDefId: createId.cardDef('contract_standard_delivery') } },
        cardDefs
      );
      
      const contractId = acceptResult.state.cards.activeContracts[0]!;
      const stateWithNoCredits = {
        ...acceptResult.state,
        resources: { ...acceptResult.state.resources, credits: 0 },
      };
      
      const abandonResult = dispatch(
        stateWithNoCredits,
        { type: 'CONTRACT_ABANDON', payload: { instanceId: contractId } },
        cardDefs
      );
      
      expect(abandonResult.state.resources.credits).toBe(0);
    });

    it('clamps morale to 0 on contract abandon', () => {
      const acceptResult = dispatch(
        state,
        { type: 'CONTRACT_ACCEPT', payload: { cardDefId: createId.cardDef('contract_standard_delivery') } },
        cardDefs
      );
      
      const contractId = acceptResult.state.cards.activeContracts[0]!;
      const stateWithLowMorale = {
        ...acceptResult.state,
        resources: { ...acceptResult.state.resources, morale: 1 },
      };
      
      const abandonResult = dispatch(
        stateWithLowMorale,
        { type: 'CONTRACT_ABANDON', payload: { instanceId: contractId } },
        cardDefs
      );
      
      expect(abandonResult.state.resources.morale).toBe(0);
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
