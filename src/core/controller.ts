import type { GameState, GameAction, CardDef, Scenelet, PortId, ChronicleEntry, AchievementId } from './types.js';
import { dispatch } from './actions.js';
import { 
  calculateJourneyEventCount, 
  calculateFuelCost,
  processJourneyWear, 
  processCargoDecay, 
  tickContractTimers,
  advanceCycle,
  incrementJumps,
} from './simulate.js';
import { selectEvent, applyEffects, createSeededRng, type TriggeredEvent, type EventContextType } from './events.js';
import { createInitialState } from './init.js';
import { DEFAULT_CONFIG, createId } from './types.js';
import { checkAchievements, getAchievement } from '../content/achievements/index.js';

const SAVE_KEY = 'cargo_hold_save';

export interface JourneyState {
  destination: PortId;
  eventsRemaining: number;
  totalEvents: number;
}

export interface GameController {
  getState(): GameState;
  dispatch(action: GameAction): { success: boolean; message?: string | undefined };
  travel(destination: PortId): { success: boolean; message?: string };
  save(): void;
  load(): boolean;
  reset(): void;
  subscribe(listener: StateListener): () => void;
  triggerPortEvent(): TriggeredEvent | null;
  resolveEventChoice(choiceIndex: number): void;
  getCurrentEvent(): TriggeredEvent | null;
  getJourneyState(): JourneyState | null;
  isGameOver(): boolean;
  getNewAchievements(): AchievementId[];
  clearNewAchievements(): void;
}

export type StateListener = (state: GameState) => void;

export function createGameController(
  cardDefs: Map<string, CardDef>,
  scenelets: Scenelet[]
): GameController {
  let state = createInitialState();
  let currentEvent: TriggeredEvent | null = null;
  let journeyState: JourneyState | null = null;
  let pendingAchievements: AchievementId[] = [];
  const listeners = new Set<StateListener>();

  function processAchievements() {
    const newlyUnlocked = checkAchievements(state);
    if (newlyUnlocked.length > 0) {
      pendingAchievements.push(...newlyUnlocked);
      
      const unlockedAt = { ...state.achievements.unlockedAt };
      for (const id of newlyUnlocked) {
        unlockedAt[id] = state.time.cycle;
      }
      
      state = {
        ...state,
        achievements: {
          unlocked: [...state.achievements.unlocked, ...newlyUnlocked],
          unlockedAt,
        },
      };
      
      for (const id of newlyUnlocked) {
        const achievement = getAchievement(id);
        if (achievement) {
          const chronicle: ChronicleEntry = {
            id: createId.chronicleEntry(`achievement-${id}-${state.time.cycle}`),
            type: 'milestone',
            timestamp: { cycle: state.time.cycle },
            title: `Achievement: ${achievement.name}`,
            text: achievement.description,
            tags: ['achievement'],
          };
          state = {
            ...state,
            chronicle: [...state.chronicle, chronicle],
          };
        }
      }
    }
  }

  function notify() {
    for (const listener of listeners) {
      listener(state);
    }
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
        console.warn('Save version mismatch, starting fresh');
        return false;
      }

      state = loaded;
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
    journeyState = null;
    notify();
  }

  function dispatchAction(action: GameAction) {
    const result = dispatch(state, action, cardDefs, DEFAULT_CONFIG);
    
    if (result.success) {
      state = result.state;
      processAchievements();
      save();
      notify();
    }
    
    return { success: result.success, message: result.message };
  }

  function travel(destination: PortId): { success: boolean; message?: string } {
    if (journeyState) {
      return { success: false, message: 'Already traveling' };
    }

    if (state.world.currentLocation === destination) {
      return { success: false, message: 'Already at this location' };
    }

    const fuelCost = calculateFuelCost(state, cardDefs, DEFAULT_CONFIG);
    if (state.resources.fuel < fuelCost) {
      return { success: false, message: 'Insufficient fuel' };
    }

    state = {
      ...state,
      resources: {
        ...state.resources,
        fuel: state.resources.fuel - fuelCost,
      },
    };

    const eventCount = calculateJourneyEventCount(state, destination, DEFAULT_CONFIG);
    
    journeyState = {
      destination,
      eventsRemaining: eventCount,
      totalEvents: eventCount,
    };

    const port = state.world.ports[destination];
    const chronicle: ChronicleEntry = {
      id: createId.chronicleEntry(`departure-${destination}-${state.time.cycle}`),
      type: 'departure',
      timestamp: { cycle: state.time.cycle },
      title: `Departed for ${port?.name ?? 'Unknown'}`,
      text: `The hold is sealed. The jump drive spools. ${port?.name ?? 'Our destination'} awaits.`,
      tags: ['travel', 'departure'],
      refs: { portId: destination },
    };
    
    state = {
      ...state,
      chronicle: [...state.chronicle, chronicle],
    };

    triggerNextJourneyEvent();
    
    return { success: true, message: `Jumping to ${port?.name ?? 'unknown'}` };
  }

  function triggerNextJourneyEvent() {
    if (!journeyState) return;

    if (journeyState.eventsRemaining <= 0) {
      completeJourney();
      return;
    }

    state = processJourneyWear(state, DEFAULT_CONFIG);
    
    const decayResult = processCargoDecay(state, cardDefs);
    state = decayResult.state;
    
    const contractResult = tickContractTimers(state, cardDefs);
    state = contractResult.state;

    state = advanceCycle(state);

    const rng = createSeededRng(state.rngState);
    state = { ...state, rngState: state.rngState + 1 };

    const context = { state, cardDefs, rng, contextType: 'journey' as EventContextType };
    const journeyScenelets = scenelets.filter(s => 
      !s.requirements.context || s.requirements.context === 'journey' || s.requirements.context === 'any'
    );
    
    const event = selectEvent(journeyScenelets, context);
    
    journeyState = {
      ...journeyState,
      eventsRemaining: journeyState.eventsRemaining - 1,
    };

    if (event) {
      currentEvent = event;
    } else {
      if (journeyState.eventsRemaining > 0) {
        triggerNextJourneyEvent();
      } else {
        completeJourney();
      }
    }

    notify();
  }

  function completeJourney() {
    if (!journeyState) return;

    const destination = journeyState.destination;
    const port = state.world.ports[destination];

    state = incrementJumps(state);

    const chronicle: ChronicleEntry = {
      id: createId.chronicleEntry(`arrival-${destination}-${state.time.cycle}`),
      type: 'arrival',
      timestamp: { cycle: state.time.cycle },
      title: `Arrived at ${port?.name ?? 'Unknown Port'}`,
      text: `The jump ends. ${port?.name ?? 'Our destination'} emerges from the static.`,
      tags: ['travel', 'arrival'],
      refs: { portId: destination },
    };

    const wasFirstVisit = !state.world.ports[destination]?.lastVisited;
    
    state = {
      ...state,
      world: {
        ...state.world,
        currentLocation: destination,
        ports: {
          ...state.world.ports,
          [destination]: port ? {
            ...port,
            lastVisited: { cycle: state.time.cycle },
          } : state.world.ports[destination],
        },
        knownPorts: state.world.knownPorts.includes(destination) 
          ? state.world.knownPorts 
          : [...state.world.knownPorts, destination],
      },
      chronicle: [...state.chronicle, chronicle],
      stats: {
        ...state.stats,
        totalDistanceTraveled: state.stats.totalDistanceTraveled + 1,
        portsVisited: wasFirstVisit ? state.stats.portsVisited + 1 : state.stats.portsVisited,
      },
    };

    journeyState = null;
    processAchievements();
    save();
    notify();
  }

  function triggerPortEvent(): TriggeredEvent | null {
    if (currentEvent) return currentEvent;
    if (journeyState) return null;

    const rng = createSeededRng(state.rngState);
    state = { ...state, rngState: state.rngState + 1 };

    const context = { state, cardDefs, rng, contextType: 'port' as EventContextType };
    const portScenelets = scenelets.filter(s => 
      !s.requirements.context || s.requirements.context === 'port' || s.requirements.context === 'any'
    );
    
    const event = selectEvent(portScenelets, context);
    
    if (event) {
      currentEvent = event;
      notify();
    }
    
    return event;
  }

  function resolveEventChoice(choiceIndex: number) {
    if (!currentEvent) return;

    const passage = currentEvent.scenelet.passages[currentEvent.passageIndex];
    
    if (!passage?.choices) {
      currentEvent = null;
      if (journeyState && journeyState.eventsRemaining > 0) {
        triggerNextJourneyEvent();
      } else if (journeyState) {
        completeJourney();
      }
      notify();
      return;
    }

    if (choiceIndex < 0 || choiceIndex >= passage.choices.length) {
      currentEvent = null;
      if (journeyState && journeyState.eventsRemaining > 0) {
        triggerNextJourneyEvent();
      } else if (journeyState) {
        completeJourney();
      }
      notify();
      return;
    }

    const choice = passage.choices[choiceIndex];
    if (!choice) {
      currentEvent = null;
      if (journeyState && journeyState.eventsRemaining > 0) {
        triggerNextJourneyEvent();
      } else if (journeyState) {
        completeJourney();
      }
      notify();
      return;
    }
    
    state = applyEffects(state, choice.effects, cardDefs);

    if (choice.nextPassage !== undefined) {
      currentEvent = {
        ...currentEvent,
        passageIndex: choice.nextPassage,
      };
    } else {
      currentEvent = null;
      if (journeyState && journeyState.eventsRemaining > 0) {
        triggerNextJourneyEvent();
      } else if (journeyState) {
        completeJourney();
      }
    }

    processAchievements();
    save();
    notify();
  }

  function isGameOver(): boolean {
    return state.resources.hull <= 0 || state.resources.morale <= 0;
  }

  return {
    getState: () => state,
    dispatch: dispatchAction,
    travel,
    save,
    load,
    reset,
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    triggerPortEvent,
    resolveEventChoice,
    getCurrentEvent: () => currentEvent,
    getJourneyState: () => journeyState,
    isGameOver,
    getNewAchievements: () => pendingAchievements,
    clearNewAchievements: () => { pendingAchievements = []; },
  };
}
