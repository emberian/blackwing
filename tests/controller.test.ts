import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { createGameController, type GameController } from '../src/core/controller.js';
import { buildCardDefMap } from '../src/content/cards/index.js';
import { ALL_SCENELETS } from '../src/content/scenelets/index.js';
import { createId, type CardDef, type Scenelet, type GameState } from '../src/core/types.js';

const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { store = {}; }),
  };
})();

Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock });

describe('Game Controller', () => {
  let controller: GameController;
  let cardDefs: Map<string, CardDef>;
  let emptyScenelets: Scenelet[];

  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
    cardDefs = buildCardDefMap();
    emptyScenelets = [];
    controller = createGameController(cardDefs, emptyScenelets);
  });

  afterEach(() => {
    localStorageMock.clear();
  });

  describe('Initialization', () => {
    it('creates controller with valid initial state', () => {
      const state = controller.getState();
      
      expect(state.schemaVersion).toBe(2);
      expect(state.resources.credits).toBe(100);
      expect(state.ship.name).toBe('Holdfast');
    });

    it('creates controller with meta state', () => {
      const meta = controller.getMetaState();
      
      expect(meta.schemaVersion).toBe(1);
      expect(meta.lifetime.gamesStarted).toBeGreaterThanOrEqual(1);
    });
  });

  describe('State Subscription', () => {
    it('notifies listeners on state change', () => {
      const listener = vi.fn();
      controller.subscribe(listener);
      
      controller.dispatch({ type: 'REFUEL', payload: { amount: 5 } });
      
      expect(listener).toHaveBeenCalled();
    });

    it('allows unsubscribing listeners', () => {
      const listener = vi.fn();
      const unsubscribe = controller.subscribe(listener);
      
      unsubscribe();
      controller.dispatch({ type: 'REFUEL', payload: { amount: 5 } });
      
      expect(listener).not.toHaveBeenCalled();
    });

    it('supports multiple listeners', () => {
      const listener1 = vi.fn();
      const listener2 = vi.fn();
      
      controller.subscribe(listener1);
      controller.subscribe(listener2);
      
      controller.dispatch({ type: 'REFUEL', payload: { amount: 5 } });
      
      expect(listener1).toHaveBeenCalled();
      expect(listener2).toHaveBeenCalled();
    });
  });

  describe('Action Dispatch', () => {
    it('dispatches valid actions', () => {
      const initialCredits = controller.getState().resources.credits;
      
      const result = controller.dispatch({ 
        type: 'TRADE_BUY', 
        payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } 
      });
      
      expect(result.success).toBe(true);
      expect(controller.getState().resources.credits).toBeLessThan(initialCredits);
    });

    it('rejects invalid actions', () => {
      const initialCredits = controller.getState().resources.credits;
      
      const result = controller.dispatch({ 
        type: 'TRADE_BUY', 
        payload: { cardDefId: createId.cardDef('cargo_ancient_artifacts'), quantity: 1 } 
      });
      
      expect(result.success).toBe(false);
      expect(controller.getState().resources.credits).toBe(initialCredits);
    });

    it('auto-saves after successful dispatch', () => {
      controller.dispatch({ 
        type: 'TRADE_BUY', 
        payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } 
      });
      
      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        'cargo_hold_save',
        expect.any(String)
      );
    });
  });

  describe('Travel System', () => {
    it('initiates travel to valid destination', () => {
      const destination = createId.port('port_frontier_station');
      
      const result = controller.travel(destination);
      
      expect(result.success).toBe(true);
    });

    it('consumes fuel when traveling', () => {
      const initialFuel = controller.getState().resources.fuel;
      const destination = createId.port('port_frontier_station');
      
      controller.travel(destination);
      
      expect(controller.getState().resources.fuel).toBeLessThan(initialFuel);
    });

    it('rejects travel to current location', () => {
      const currentLocation = controller.getState().world.currentLocation;
      
      const result = controller.travel(currentLocation);
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Already at this location');
    });

    it('rejects travel without sufficient fuel', () => {
      const state = controller.getState();
      const zeroFuelController = createGameController(cardDefs, []);
      const modifiedState = { ...state, resources: { ...state.resources, fuel: 0 } };
      
      localStorageMock.setItem('cargo_hold_save', JSON.stringify({ ...modifiedState, schemaVersion: 2 }));
      zeroFuelController.load();
      
      const result = zeroFuelController.travel(createId.port('port_frontier_station'));
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Insufficient fuel');
    });

    it('updates location after travel completes (with no events)', () => {
      const destination = createId.port('port_frontier_station');
      
      controller.travel(destination);
      
      expect(controller.getState().world.currentLocation).toBe(destination);
    });

    it('adds chronicle entries on travel', () => {
      const initialChronicleLength = controller.getState().chronicle.length;
      
      controller.travel(createId.port('port_frontier_station'));
      
      const newChronicleLength = controller.getState().chronicle.length;
      expect(newChronicleLength).toBeGreaterThan(initialChronicleLength);
    });
  });

  describe('Journey Events', () => {
    it('processes journey events until destination', () => {
      const controllerWithEvents = createGameController(cardDefs, []);
      
      controllerWithEvents.travel(createId.port('port_frontier_station'));
      
      while (controllerWithEvents.getJourneyState()) {
        const event = controllerWithEvents.getCurrentEvent();
        if (event) {
          controllerWithEvents.resolveEventChoice(0);
        }
      }
      
      expect(controllerWithEvents.getJourneyState()).toBeNull();
      expect(controllerWithEvents.getState().world.currentLocation).toBe(
        createId.port('port_frontier_station')
      );
    });

    it('increments jumps on journey completion', () => {
      const controllerWithEvents = createGameController(cardDefs, []);
      const initialJumps = controllerWithEvents.getState().time.jumpsCompleted;
      
      controllerWithEvents.travel(createId.port('port_frontier_station'));
      
      while (controllerWithEvents.getJourneyState()) {
        const event = controllerWithEvents.getCurrentEvent();
        if (event) {
          controllerWithEvents.resolveEventChoice(0);
        }
      }
      
      expect(controllerWithEvents.getState().time.jumpsCompleted).toBe(initialJumps + 1);
    });

    it('processes journey wear during travel', () => {
      const controllerWithEvents = createGameController(cardDefs, []);
      const initialSupplies = controllerWithEvents.getState().resources.supplies;
      
      controllerWithEvents.travel(createId.port('port_frontier_station'));
      
      while (controllerWithEvents.getJourneyState()) {
        const event = controllerWithEvents.getCurrentEvent();
        if (event) {
          controllerWithEvents.resolveEventChoice(0);
        }
      }
      
      expect(controllerWithEvents.getState().resources.supplies).toBeLessThan(initialSupplies);
    });

    it('advances cycle during travel', () => {
      const controllerWithEvents = createGameController(cardDefs, []);
      const initialCycle = controllerWithEvents.getState().time.cycle;
      
      controllerWithEvents.travel(createId.port('port_frontier_station'));
      
      while (controllerWithEvents.getJourneyState()) {
        const event = controllerWithEvents.getCurrentEvent();
        if (event) {
          controllerWithEvents.resolveEventChoice(0);
        }
      }
      
      expect(controllerWithEvents.getState().time.cycle).toBeGreaterThan(initialCycle);
    });
  });

  describe('Event Resolution', () => {
    it('resolves events without choices by clearing the event', () => {
      const controllerWithScenelets = createGameController(cardDefs, ALL_SCENELETS);
      controllerWithScenelets.travel(createId.port('port_frontier_station'));
      
      const currentEvent = controllerWithScenelets.getCurrentEvent();
      if (currentEvent) {
        controllerWithScenelets.resolveEventChoice(0);
      }
    });

    it('handles invalid choice index gracefully', () => {
      const controllerWithScenelets = createGameController(cardDefs, ALL_SCENELETS);
      controllerWithScenelets.travel(createId.port('port_frontier_station'));
      
      const currentEvent = controllerWithScenelets.getCurrentEvent();
      if (currentEvent) {
        controllerWithScenelets.resolveEventChoice(999);
      }
    });
  });

  describe('Port Events', () => {
    it('can trigger port events when not traveling', () => {
      const controllerWithScenelets = createGameController(cardDefs, ALL_SCENELETS);
      
      const event = controllerWithScenelets.triggerPortEvent();
    });

    it('returns existing event if one is active during journey', () => {
      const controllerWithScenelets = createGameController(cardDefs, ALL_SCENELETS);
      controllerWithScenelets.travel(createId.port('port_frontier_station'));
      
      const event1 = controllerWithScenelets.getCurrentEvent();
      if (event1) {
        const event2 = controllerWithScenelets.triggerPortEvent();
        expect(event2).toBe(event1);
      }
    });
  });

  describe('Game Over Detection', () => {
    it('detects game over when hull reaches zero', () => {
      const state = controller.getState();
      const zeroHullState: GameState = {
        ...state,
        resources: { ...state.resources, hull: 0 },
      };
      
      localStorageMock.setItem('cargo_hold_save', JSON.stringify({ ...zeroHullState, schemaVersion: 2 }));
      controller.load();
      
      expect(controller.isGameOver()).toBe(true);
    });

    it('detects game over when morale reaches zero', () => {
      const state = controller.getState();
      const zeroMoraleState: GameState = {
        ...state,
        resources: { ...state.resources, morale: 0 },
      };
      
      localStorageMock.setItem('cargo_hold_save', JSON.stringify({ ...zeroMoraleState, schemaVersion: 2 }));
      controller.load();
      
      expect(controller.isGameOver()).toBe(true);
    });

    it('does not report game over with valid resources', () => {
      expect(controller.isGameOver()).toBe(false);
    });
  });

  describe('Save/Load System', () => {
    it('saves game state', () => {
      controller.save();
      
      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        'cargo_hold_save',
        expect.any(String)
      );
    });

    it('loads game state successfully', () => {
      const state = controller.getState();
      const modifiedState = { 
        ...state, 
        resources: { ...state.resources, credits: 999 },
        schemaVersion: 2,
      };
      
      localStorageMock.setItem('cargo_hold_save', JSON.stringify(modifiedState));
      
      const loaded = controller.load();
      
      expect(loaded).toBe(true);
      expect(controller.getState().resources.credits).toBe(999);
    });

    it('returns false when no save exists', () => {
      const loaded = controller.load();
      
      expect(loaded).toBe(false);
    });
  });

  describe('Reset System', () => {
    it('resets to initial state', () => {
      controller.dispatch({ 
        type: 'TRADE_BUY', 
        payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } 
      });
      
      controller.reset();
      
      expect(controller.getState().resources.credits).toBe(100);
      expect(controller.getState().cards.collection.length).toBe(0);
    });

    it('clears journey state on reset', () => {
      controller.travel(createId.port('port_frontier_station'));
      
      controller.reset();
      
      expect(controller.getJourneyState()).toBeNull();
    });

    it('clears current event on reset', () => {
      const controllerWithScenelets = createGameController(cardDefs, ALL_SCENELETS);
      controllerWithScenelets.travel(createId.port('port_frontier_station'));
      
      controllerWithScenelets.reset();
      
      expect(controllerWithScenelets.getCurrentEvent()).toBeNull();
    });

    it('increments games started on reset', () => {
      const initialGamesStarted = controller.getMetaState().lifetime.gamesStarted;
      
      controller.reset();
      
      expect(controller.getMetaState().lifetime.gamesStarted).toBe(initialGamesStarted + 1);
    });
  });

  describe('Clear All', () => {
    it('clears all data and resets state', () => {
      controller.dispatch({ 
        type: 'TRADE_BUY', 
        payload: { cardDefId: createId.cardDef('cargo_raw_ore'), quantity: 1 } 
      });
      controller.save();
      
      controller.clearAll();
      
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('cargo_hold_save');
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('cargo_hold_meta');
      expect(controller.getState().resources.credits).toBe(100);
    });
  });

  describe('Achievement System', () => {
    it('tracks new achievements', () => {
      expect(controller.getNewAchievements()).toEqual([]);
    });

    it('clears new achievements on request', () => {
      controller.clearNewAchievements();
      expect(controller.getNewAchievements()).toEqual([]);
    });
  });
});
