import type { GameState, GameAction, CardDef, Scenelet } from './types.js';
import { dispatch } from './actions.js';
import { simulateOffline, compressEvents, type SimulationResult } from './simulate.js';
import { selectEvent, applyEffects, createSeededRng, type TriggeredEvent } from './events.js';
import { createInitialState } from './init.js';
import { DEFAULT_CONFIG } from './types.js';

const SAVE_KEY = 'cargo_hold_save';
const AUTOSAVE_INTERVAL = 30_000;

export interface GameController {
  getState(): GameState;
  dispatch(action: GameAction): void;
  save(): void;
  load(): boolean;
  reset(): void;
  subscribe(listener: StateListener): () => void;
  triggerEvent(): TriggeredEvent | null;
  resolveEventChoice(choiceIndex: number): void;
  getCurrentEvent(): TriggeredEvent | null;
}

export type StateListener = (state: GameState, events: SimulationResult['events']) => void;

export function createGameController(
  cardDefs: Map<string, CardDef>,
  scenelets: Scenelet[]
): GameController {
  let state = createInitialState();
  let currentEvent: TriggeredEvent | null = null;
  const listeners = new Set<StateListener>();
  let autosaveTimer: ReturnType<typeof setInterval> | null = null;
  let tickTimer: ReturnType<typeof setInterval> | null = null;

  function notify(events: SimulationResult['events'] = []) {
    for (const listener of listeners) {
      listener(state, events);
    }
  }

  function processOfflineTime() {
    const now = Date.now();
    const result = simulateOffline(state, cardDefs, now, DEFAULT_CONFIG);
    
    if (result.ticksSimulated > 0) {
      state = result.state;
      const compressed = compressEvents(result.events);
      notify(compressed);
    }
  }

  function startTimers() {
    if (tickTimer) clearInterval(tickTimer);
    if (autosaveTimer) clearInterval(autosaveTimer);

    tickTimer = setInterval(() => {
      processOfflineTime();
    }, DEFAULT_CONFIG.msPerTick);

    autosaveTimer = setInterval(() => {
      save();
    }, AUTOSAVE_INTERVAL);
  }

  function save() {
    try {
      const serialized = JSON.stringify(state);
      localStorage.setItem(SAVE_KEY, serialized);
    } catch (e) {
      console.error('Failed to save game:', e);
    }
  }

  function load(): boolean {
    try {
      const serialized = localStorage.getItem(SAVE_KEY);
      if (!serialized) return false;

      const loaded = JSON.parse(serialized) as GameState;
      
      if (loaded.schemaVersion !== state.schemaVersion) {
        console.warn('Save version mismatch, may need migration');
      }

      state = loaded;
      processOfflineTime();
      notify();
      return true;
    } catch (e) {
      console.error('Failed to load game:', e);
      return false;
    }
  }

  function reset() {
    localStorage.removeItem(SAVE_KEY);
    state = createInitialState();
    currentEvent = null;
    notify();
  }

  function dispatchAction(action: GameAction) {
    const result = dispatch(state, action, cardDefs, DEFAULT_CONFIG);
    
    if (result.success) {
      state = result.state;
      notify();
    }
    
    return result;
  }

  function triggerEvent(): TriggeredEvent | null {
    if (currentEvent) return currentEvent;

    const rng = createSeededRng(state.rngState);
    state = { ...state, rngState: state.rngState + 1 };

    const context = { state, cardDefs, rng };
    const event = selectEvent(scenelets, context);
    
    if (event) {
      currentEvent = event;
      notify();
    }
    
    return event;
  }

  function resolveEventChoice(choiceIndex: number) {
    if (!currentEvent) return;

    const passage = currentEvent.scenelet.passages[currentEvent.passageIndex];
    if (!passage?.choices) return;

    const choice = passage.choices[choiceIndex];
    if (!choice) return;

    state = applyEffects(state, choice.effects, cardDefs);

    if (choice.nextPassage !== undefined) {
      currentEvent = {
        ...currentEvent,
        passageIndex: choice.nextPassage,
      };
    } else {
      currentEvent = null;
    }

    notify();
  }

  startTimers();

  return {
    getState: () => state,
    dispatch: dispatchAction,
    save,
    load,
    reset,
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    triggerEvent,
    resolveEventChoice,
    getCurrentEvent: () => currentEvent,
  };
}
