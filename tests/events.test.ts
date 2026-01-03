import { describe, it, expect, beforeEach } from 'vitest';
import {
  selectEvent,
  applyEffects,
  createSeededRng,
  type EventContext,
} from '../src/core/events.js';
import { createInitialState } from '../src/core/init.js';
import { buildCardDefMap } from '../src/content/cards/index.js';
import { createId, type GameState, type Scenelet, type SceneletId, type CardDef, type CardInstanceId, type CardInstance } from '../src/core/types.js';

describe('Events Module', () => {
  let state: GameState;
  let cardDefs: Map<string, CardDef>;
  
  beforeEach(() => {
    state = createInitialState();
    cardDefs = buildCardDefMap();
  });

  describe('Seeded RNG', () => {
    it('produces deterministic results from same seed', () => {
      const rng1 = createSeededRng(12345);
      const rng2 = createSeededRng(12345);
      
      const results1 = [rng1(), rng1(), rng1()];
      const results2 = [rng2(), rng2(), rng2()];
      
      expect(results1).toEqual(results2);
    });

    it('produces different results from different seeds', () => {
      const rng1 = createSeededRng(12345);
      const rng2 = createSeededRng(54321);
      
      expect(rng1()).not.toBe(rng2());
    });

    it('produces values between 0 and 1', () => {
      const rng = createSeededRng(98765);
      
      for (let i = 0; i < 100; i++) {
        const value = rng();
        expect(value).toBeGreaterThanOrEqual(0);
        expect(value).toBeLessThanOrEqual(1);
      }
    });

    it('produces varying values (not constant)', () => {
      const rng = createSeededRng(11111);
      const values = new Set<number>();
      
      for (let i = 0; i < 100; i++) {
        values.add(rng());
      }
      
      expect(values.size).toBeGreaterThan(50);
    });
  });

  describe('Event Selection', () => {
    const createTestScenelet = (id: string, weight: number, requirements: Scenelet['requirements'] = {}): Scenelet => ({
      id: id as SceneletId,
      title: `Test Event ${id}`,
      tags: ['test'],
      requirements,
      weight,
      cooldown: 0,
      passages: [{ text: 'Test passage' }],
    });

    it('selects event from eligible pool', () => {
      const scenelets = [
        createTestScenelet('event_1', 100),
        createTestScenelet('event_2', 100),
      ];
      
      const context: EventContext = {
        state,
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const event = selectEvent(scenelets, context);
      
      expect(event).not.toBeNull();
      expect(['event_1', 'event_2']).toContain(event?.scenelet.id);
    });

    it('returns null when no events are eligible', () => {
      const scenelets = [
        createTestScenelet('event_1', 100, { requiredFlags: ['nonexistent_flag'] }),
      ];
      
      const context: EventContext = {
        state,
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const event = selectEvent(scenelets, context);
      
      expect(event).toBeNull();
    });

    it('respects context requirements (journey vs port)', () => {
      const scenelets = [
        createTestScenelet('port_only', 100, { context: 'port' }),
        createTestScenelet('journey_only', 100, { context: 'journey' }),
      ];
      
      const journeyContext: EventContext = {
        state,
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const event = selectEvent(scenelets, journeyContext);
      
      expect(event?.scenelet.id).toBe('journey_only');
    });

    it('respects minimum resource requirements', () => {
      const scenelets = [
        createTestScenelet('needs_credits', 100, { minResources: { credits: 1000 } }),
        createTestScenelet('no_requirements', 100),
      ];
      
      const context: EventContext = {
        state: { ...state, resources: { ...state.resources, credits: 50 } },
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const event = selectEvent(scenelets, context);
      
      expect(event?.scenelet.id).toBe('no_requirements');
    });

    it('respects maximum resource requirements', () => {
      const scenelets = [
        createTestScenelet('low_morale_only', 100, { maxResources: { morale: 30 } }),
        createTestScenelet('no_requirements', 100),
      ];
      
      const context: EventContext = {
        state: { ...state, resources: { ...state.resources, morale: 75 } },
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const event = selectEvent(scenelets, context);
      
      expect(event?.scenelet.id).toBe('no_requirements');
    });

    it('respects required flags', () => {
      const scenelets = [
        createTestScenelet('needs_flag', 100, { requiredFlags: ['visited_sanctuary'] }),
        createTestScenelet('no_flag_needed', 100),
      ];
      
      const context: EventContext = {
        state: { ...state, flags: {} },
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const event = selectEvent(scenelets, context);
      
      expect(event?.scenelet.id).toBe('no_flag_needed');
    });

    it('allows events when required flags are present', () => {
      const scenelets = [
        createTestScenelet('needs_flag', 100, { requiredFlags: ['visited_sanctuary'] }),
      ];
      
      const context: EventContext = {
        state: { ...state, flags: { visited_sanctuary: true } },
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const event = selectEvent(scenelets, context);
      
      expect(event?.scenelet.id).toBe('needs_flag');
    });

    it('respects excluded flags', () => {
      const scenelets = [
        createTestScenelet('not_if_flag', 100, { excludedFlags: ['completed_quest'] }),
      ];
      
      const context: EventContext = {
        state: { ...state, flags: { completed_quest: true } },
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const event = selectEvent(scenelets, context);
      
      expect(event).toBeNull();
    });

    it('respects event weights in selection probability', () => {
      const scenelets = [
        createTestScenelet('rare', 1),
        createTestScenelet('common', 99),
      ];
      
      let rareCount = 0;
      let commonCount = 0;
      
      for (let i = 0; i < 1000; i++) {
        const context: EventContext = {
          state,
          cardDefs,
          rng: createSeededRng(i),
          contextType: 'journey',
        };
        
        const event = selectEvent(scenelets, context);
        if (event?.scenelet.id === 'rare') rareCount++;
        if (event?.scenelet.id === 'common') commonCount++;
      }
      
      expect(commonCount).toBeGreaterThan(rareCount * 10);
    });

    it('selects event with context any for both journey and port', () => {
      const scenelets = [
        createTestScenelet('any_context', 100, { context: 'any' }),
      ];
      
      const journeyContext: EventContext = {
        state,
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'journey',
      };
      
      const portContext: EventContext = {
        state,
        cardDefs,
        rng: createSeededRng(12345),
        contextType: 'port',
      };
      
      expect(selectEvent(scenelets, journeyContext)?.scenelet.id).toBe('any_context');
      expect(selectEvent(scenelets, portContext)?.scenelet.id).toBe('any_context');
    });
  });

  describe('Apply Effects', () => {
    it('applies positive resource changes', () => {
      const effects = { resources: { credits: 50, fuel: 10 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.credits).toBe(state.resources.credits + 50);
      expect(newState.resources.fuel).toBe(state.resources.fuel + 10);
    });

    it('applies negative resource changes', () => {
      const effects = { resources: { credits: -30, supplies: -5 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.credits).toBe(state.resources.credits - 30);
      expect(newState.resources.supplies).toBe(state.resources.supplies - 5);
    });

    it('clamps resources to minimum of 0', () => {
      const effects = { resources: { credits: -9999 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.credits).toBe(0);
    });

    it('clamps hull to max hull value', () => {
      const effects = { resources: { hull: 500 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.hull).toBe(state.ship.maxHull);
    });

    it('clamps morale to 100', () => {
      const effects = { resources: { morale: 500 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.morale).toBe(100);
    });

    it('sets flags', () => {
      const effects = { setFlags: { quest_started: true, visits: 5 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.flags.quest_started).toBe(true);
      expect(newState.flags.visits).toBe(5);
    });

    it('adds cards to collection', () => {
      const effects = { addCards: [createId.cardDef('cargo_raw_ore')] };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.cards.collection.length).toBe(1);
      const addedCard = newState.cards.instances[newState.cards.collection[0]!];
      expect(addedCard?.cardDefId).toBe(createId.cardDef('cargo_raw_ore'));
    });

    it('removes cards from collection and deck', () => {
      const cargoId = 'test-cargo' as CardInstanceId;
      const instance: CardInstance = {
        instanceId: cargoId,
        cardDefId: createId.cardDef('cargo_raw_ore'),
        level: 1,
        condition: 100,
        mods: [],
        acquiredAt: { cycle: 0 },
      };
      
      const stateWithCargo: GameState = {
        ...state,
        cards: {
          ...state.cards,
          instances: { [cargoId]: instance },
          collection: [cargoId],
          deck: [cargoId],
        },
      };
      
      const effects = { removeCards: [cargoId] };
      
      const newState = applyEffects(stateWithCargo, effects, cardDefs);
      
      expect(newState.cards.collection).not.toContain(cargoId);
      expect(newState.cards.deck).not.toContain(cargoId);
      expect(newState.cards.instances[cargoId]).toBeUndefined();
    });

    it('applies hull damage', () => {
      const effects = { damage: { hull: 25 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.hull).toBe(state.resources.hull - 25);
    });

    it('applies morale damage', () => {
      const effects = { damage: { morale: 15 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.morale).toBe(state.resources.morale - 15);
    });

    it('clamps damage to minimum 0', () => {
      const effects = { damage: { hull: 9999, morale: 9999 } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.hull).toBe(0);
      expect(newState.resources.morale).toBe(0);
    });

    it('adds chronicle entries', () => {
      const effects = { addChronicle: { title: 'Strange Discovery', text: 'You found something odd.' } };
      
      const newState = applyEffects(state, effects, cardDefs);
      
      const lastEntry = newState.chronicle[newState.chronicle.length - 1];
      expect(lastEntry?.title).toBe('Strange Discovery');
      expect(lastEntry?.text).toBe('You found something odd.');
      expect(lastEntry?.type).toBe('encounter');
    });

    it('handles multiple effects simultaneously', () => {
      const effects = {
        resources: { credits: 100, fuel: -5 },
        setFlags: { event_happened: true },
        damage: { morale: 10 },
        addChronicle: { title: 'Event', text: 'Something happened.' },
      };
      
      const initialCredits = state.resources.credits;
      const initialFuel = state.resources.fuel;
      const initialMorale = state.resources.morale;
      const initialChronicleLength = state.chronicle.length;
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources.credits).toBe(initialCredits + 100);
      expect(newState.resources.fuel).toBe(initialFuel - 5);
      expect(newState.resources.morale).toBe(initialMorale - 10);
      expect(newState.flags.event_happened).toBe(true);
      expect(newState.chronicle.length).toBe(initialChronicleLength + 1);
    });

    it('handles empty effects object', () => {
      const effects = {};
      
      const newState = applyEffects(state, effects, cardDefs);
      
      expect(newState.resources).toEqual(state.resources);
      expect(newState.flags).toEqual(state.flags);
    });
  });
});
