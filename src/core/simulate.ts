import {
  type GameState,
  type CardDef,
  type GameConfig,
  DEFAULT_CONFIG,
  type PortId,
} from './types.js';

export function calculateJourneyEventCount(
  state: GameState,
  _destination: PortId,
  config: GameConfig = DEFAULT_CONFIG
): number {
  const { min, max } = config.journeyEventCount;
  const rngValue = seededRandom(state.rngState);
  return min + Math.floor(rngValue * (max - min + 1));
}

function seededRandom(seed: number): number {
  const state = (seed * 1103515245 + 12345) & 0x7fffffff;
  return state / 0x7fffffff;
}

export function calculateFuelCost(
  state: GameState,
  cardDefs: Map<string, CardDef>,
  config: GameConfig = DEFAULT_CONFIG
): number {
  let fuelEfficiency = 1;
  
  for (const instanceId of state.cards.deck) {
    const instance = state.cards.instances[instanceId];
    if (!instance) continue;
    
    const def = cardDefs.get(instance.cardDefId);
    if (!def?.effects.modifiers?.fuelEfficiency) continue;
    
    fuelEfficiency += def.effects.modifiers.fuelEfficiency;
  }
  
  for (const slot of Object.values(state.ship.modules)) {
    if (!slot) continue;
    const instance = state.cards.instances[slot];
    if (!instance) continue;
    
    const def = cardDefs.get(instance.cardDefId);
    if (!def?.effects.modifiers?.fuelEfficiency) continue;
    
    fuelEfficiency += def.effects.modifiers.fuelEfficiency;
  }
  
  return Math.max(1, Math.ceil(config.baseFuelPerJump / fuelEfficiency));
}

export function processJourneyWear(
  state: GameState,
  config: GameConfig = DEFAULT_CONFIG
): GameState {
  const crewCount = state.cards.activeCrew.length;
  const supplyCost = config.journeySupplyCost + crewCount;
  
  const newSupplies = Math.max(0, state.resources.supplies - supplyCost);
  const starving = newSupplies === 0 && state.resources.supplies > 0;
  
  let newIntegrity = state.resources.integrity;
  if (starving) {
    newIntegrity = Math.max(0, newIntegrity - 15);
  }
  
  const newHull = Math.max(0, state.resources.hull - config.journeyHullWear);
  
  return {
    ...state,
    resources: {
      ...state.resources,
      supplies: newSupplies,
      hull: newHull,
      integrity: newIntegrity,
    },
  };
}

export function processCargoDecay(
  state: GameState,
  cardDefs: Map<string, CardDef>,
  rng: () => number
): { state: GameState; decayedCards: string[] } {
  const instances = { ...state.cards.instances };
  const collection = [...state.cards.collection];
  const deck = [...state.cards.deck];
  const decayedCards: string[] = [];

  for (const instanceId of [...deck, ...collection]) {
    const instance = instances[instanceId];
    if (!instance) continue;

    const def = cardDefs.get(instance.cardDefId);
    if (!def?.journeyBehavior?.decayChance) continue;

    if (rng() < def.journeyBehavior.decayChance) {
      const newCondition = instance.condition - 10;
      
      if (newCondition <= 0) {
        decayedCards.push(def.name);
        delete instances[instanceId];
        const deckIdx = deck.indexOf(instanceId);
        if (deckIdx >= 0) deck.splice(deckIdx, 1);
        const collIdx = collection.indexOf(instanceId);
        if (collIdx >= 0) collection.splice(collIdx, 1);
      } else {
        instances[instanceId] = { ...instance, condition: newCondition };
      }
    }
  }

  return {
    state: {
      ...state,
      cards: { ...state.cards, instances, deck, collection },
    },
    decayedCards,
  };
}

export function tickContractTimers(
  state: GameState,
  cardDefs: Map<string, CardDef>
): { state: GameState; expiredContracts: string[] } {
  let instances = { ...state.cards.instances };
  const activeContracts = [...state.cards.activeContracts];
  const expiredContracts: string[] = [];
  let resources = { ...state.resources };
  let stats = { ...state.stats };

  for (const contractId of [...activeContracts]) {
    const instance = instances[contractId];
    if (!instance || instance.cyclesRemaining === undefined) continue;

    const newCycles = instance.cyclesRemaining - 1;
    
    if (newCycles <= 0) {
      const def = cardDefs.get(instance.cardDefId);
      const penalty = def?.contractTerms?.penalty ?? {};
      
      expiredContracts.push(def?.name ?? 'Unknown contract');
      delete instances[contractId];
      const idx = activeContracts.indexOf(contractId);
      if (idx >= 0) activeContracts.splice(idx, 1);
      
      resources = {
        ...resources,
        credits: Math.max(0, resources.credits - (penalty.credits ?? 0)),
        integrity: Math.max(0, resources.integrity - (penalty.integrity ?? 10)),
      };
      stats = {
        ...stats,
        contractsFailed: stats.contractsFailed + 1,
      };
    } else {
      instances[contractId] = { ...instance, cyclesRemaining: newCycles };
    }
  }

  return {
    state: {
      ...state,
      resources,
      stats,
      cards: { ...state.cards, instances, activeContracts },
    },
    expiredContracts,
  };
}

export function advanceCycle(state: GameState): GameState {
  return {
    ...state,
    time: {
      ...state.time,
      cycle: state.time.cycle + 1,
    },
  };
}

export function incrementJumps(state: GameState): GameState {
  return {
    ...state,
    time: {
      ...state.time,
      jumpsCompleted: state.time.jumpsCompleted + 1,
    },
  };
}
