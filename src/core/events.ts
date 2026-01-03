import {
  type GameState,
  type Scenelet,
  type SceneletEffects,
  type CardDef,
  type ChronicleEntry,
  createId,
  type CardInstanceId,
} from './types.js';

export interface EventContext {
  state: GameState;
  cardDefs: Map<string, CardDef>;
  rng: () => number;
}

export interface TriggeredEvent {
  scenelet: Scenelet;
  passageIndex: number;
}

export function selectEvent(
  scenelets: Scenelet[],
  context: EventContext
): TriggeredEvent | null {
  const eligible = scenelets.filter(s => meetsRequirements(s, context));
  
  if (eligible.length === 0) return null;
  
  const totalWeight = eligible.reduce((sum, s) => sum + s.weight, 0);
  let roll = context.rng() * totalWeight;
  
  for (const scenelet of eligible) {
    roll -= scenelet.weight;
    if (roll <= 0) {
      return { scenelet, passageIndex: 0 };
    }
  }
  
  return { scenelet: eligible[0]!, passageIndex: 0 };
}

function meetsRequirements(scenelet: Scenelet, context: EventContext): boolean {
  const req = scenelet.requirements;
  const state = context.state;
  
  if (req.location === 'transit' && !state.time.inTransit) return false;
  if (req.location === 'port' && state.time.inTransit) return false;
  
  if (req.minResources) {
    for (const [key, min] of Object.entries(req.minResources)) {
      const current = state.resources[key as keyof typeof state.resources];
      if (current < (min ?? 0)) return false;
    }
  }
  
  if (req.maxResources) {
    for (const [key, max] of Object.entries(req.maxResources)) {
      const current = state.resources[key as keyof typeof state.resources];
      if (current > (max ?? Infinity)) return false;
    }
  }
  
  if (req.requiredFlags) {
    for (const flag of req.requiredFlags) {
      if (!state.flags[flag]) return false;
    }
  }
  
  if (req.excludedFlags) {
    for (const flag of req.excludedFlags) {
      if (state.flags[flag]) return false;
    }
  }
  
  if (req.shipTags) {
    const shipTags = getShipTags(state, context.cardDefs);
    if (!req.shipTags.every(tag => shipTags.has(tag))) return false;
  }
  
  return true;
}

function getShipTags(state: GameState, cardDefs: Map<string, CardDef>): Set<string> {
  const tags = new Set<string>();
  
  for (const instanceId of state.cards.deck) {
    const instance = state.cards.instances[instanceId];
    if (!instance) continue;
    
    const def = cardDefs.get(instance.cardDefId);
    if (!def) continue;
    
    for (const tag of def.tags) {
      tags.add(tag);
    }
    
    if (def.effects.grantsShipTags) {
      for (const tag of def.effects.grantsShipTags) {
        tags.add(tag);
      }
    }
  }
  
  for (const slot of Object.values(state.ship.modules)) {
    if (!slot) continue;
    const instance = state.cards.instances[slot];
    if (!instance) continue;
    
    const def = cardDefs.get(instance.cardDefId);
    if (!def) continue;
    
    for (const tag of def.tags) {
      tags.add(tag);
    }
    
    if (def.effects.grantsShipTags) {
      for (const tag of def.effects.grantsShipTags) {
        tags.add(tag);
      }
    }
  }
  
  return tags;
}

export function applyEffects(
  state: GameState,
  effects: SceneletEffects,
  cardDefs: Map<string, CardDef>
): GameState {
  let newState = { ...state };
  
  if (effects.resources) {
    newState.resources = {
      credits: Math.max(0, newState.resources.credits + (effects.resources.credits ?? 0)),
      fuel: Math.max(0, newState.resources.fuel + (effects.resources.fuel ?? 0)),
      supplies: Math.max(0, newState.resources.supplies + (effects.resources.supplies ?? 0)),
      hull: Math.max(0, Math.min(newState.ship.maxHull, newState.resources.hull + (effects.resources.hull ?? 0))),
      morale: Math.max(0, Math.min(100, newState.resources.morale + (effects.resources.morale ?? 0))),
    };
  }
  
  if (effects.setFlags) {
    newState.flags = { ...newState.flags, ...effects.setFlags };
  }
  
  if (effects.addCards) {
    const instances = { ...newState.cards.instances };
    const collection = [...newState.cards.collection];
    
    for (const cardDefId of effects.addCards) {
      const def = cardDefs.get(cardDefId);
      if (!def) continue;
      
      const instanceId = `card-${Date.now()}-${Math.random().toString(36).slice(2)}` as CardInstanceId;
      instances[instanceId] = {
        instanceId,
        cardDefId,
        level: 1,
        condition: 100,
        mods: [],
        acquiredAt: { era: newState.time.era, year: newState.time.year },
      };
      collection.push(instanceId);
    }
    
    newState.cards = { ...newState.cards, instances, collection };
  }
  
  if (effects.removeCards) {
    const instances = { ...newState.cards.instances };
    let collection = [...newState.cards.collection];
    let deck = [...newState.cards.deck];
    
    for (const instanceId of effects.removeCards) {
      delete instances[instanceId];
      collection = collection.filter(id => id !== instanceId);
      deck = deck.filter(id => id !== instanceId);
    }
    
    newState.cards = { ...newState.cards, instances, collection, deck };
  }
  
  if (effects.damage) {
    if (effects.damage.hull) {
      newState.resources = {
        ...newState.resources,
        hull: Math.max(0, newState.resources.hull - effects.damage.hull),
      };
    }
    if (effects.damage.morale) {
      newState.resources = {
        ...newState.resources,
        morale: Math.max(0, newState.resources.morale - effects.damage.morale),
      };
    }
  }
  
  if (effects.addChronicle) {
    const entry: ChronicleEntry = {
      id: createId.chronicleEntry(`event-${Date.now()}`),
      type: 'encounter',
      timestamp: { era: newState.time.era, year: newState.time.year },
      title: effects.addChronicle.title,
      text: effects.addChronicle.text,
      tags: ['event'],
    };
    newState.chronicle = [...newState.chronicle, entry];
  }
  
  return newState;
}

export function createSeededRng(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state * 1103515245 + 12345) & 0x7fffffff;
    return state / 0x7fffffff;
  };
}
