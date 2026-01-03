import {
  type GameState,
  type GameAction,
  type CardDef,
  type CardInstance,
  type ChronicleEntry,
  type PortId,
  type CardDefId,
  type CardInstanceId,
  type SimulationConfig,
  DEFAULT_CONFIG,
} from './types.js';
import { calculateTravelTime } from './simulate.js';

export interface ActionResult {
  state: GameState;
  success: boolean;
  message?: string;
}

type ActionHandler<T extends GameAction['type']> = (
  state: GameState,
  payload: Extract<GameAction, { type: T }>['payload'],
  cardDefs: Map<string, CardDef>,
  config: SimulationConfig
) => ActionResult;

const handlers: { [K in GameAction['type']]: ActionHandler<K> } = {
  TICK: handleTick,
  TRAVEL: handleTravel,
  TRADE_BUY: handleTradeBuy,
  TRADE_SELL: handleTradeSell,
  CARD_EQUIP: handleCardEquip,
  CARD_UNEQUIP: handleCardUnequip,
  CARD_UPGRADE: handleCardUpgrade,
  MODULE_INSTALL: handleModuleInstall,
  MODULE_UNINSTALL: handleModuleUninstall,
  CONTRACT_ACCEPT: handleContractAccept,
  CONTRACT_COMPLETE: handleContractComplete,
  CONTRACT_ABANDON: handleContractAbandon,
  CREW_HIRE: handleCrewHire,
  CREW_DISMISS: handleCrewDismiss,
  EVENT_CHOICE: handleEventChoice,
  REPAIR: handleRepair,
  RESUPPLY: handleResupply,
  REFUEL: handleRefuel,
};

export function dispatch(
  state: GameState,
  action: GameAction,
  cardDefs: Map<string, CardDef>,
  config: SimulationConfig = DEFAULT_CONFIG
): ActionResult {
  const handler = handlers[action.type] as ActionHandler<typeof action.type>;
  return handler(state, action.payload as never, cardDefs, config);
}

function handleTick(
  state: GameState,
  _payload: { deltaMs: number },
  _cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  return { state, success: true };
}

function handleTravel(
  state: GameState,
  payload: { destination: PortId },
  cardDefs: Map<string, CardDef>,
  config: SimulationConfig
): ActionResult {
  if (state.time.inTransit) {
    return { state, success: false, message: 'Already in transit' };
  }

  const travelTime = calculateTravelTime(state, payload.destination, cardDefs);
  const fuelCost = config.baseFuelPerJump;

  if (state.resources.fuel < fuelCost) {
    return { state, success: false, message: 'Insufficient fuel' };
  }

  const departedAt = { era: state.time.era, year: state.time.year };
  const arrivesAt = { era: state.time.era, year: state.time.year + travelTime };

  const port = state.world.ports[payload.destination];
  const chronicle: ChronicleEntry = {
    id: `departure-${payload.destination}-${state.time.year}` as ChronicleEntry['id'],
    type: 'departure',
    timestamp: departedAt,
    title: `Departed for ${port?.name ?? 'Unknown'}`,
    text: `The hold is sealed. ${travelTime} years to destination. The long dark begins.`,
    tags: ['travel', 'departure'],
    refs: { portId: payload.destination },
  };

  return {
    state: {
      ...state,
      resources: {
        ...state.resources,
        fuel: state.resources.fuel - fuelCost,
      },
      time: {
        ...state.time,
        inTransit: true,
        transitDestination: payload.destination,
        transitDepartedAt: departedAt,
        transitArrivesAt: arrivesAt,
      },
      chronicle: [...state.chronicle, chronicle],
      stats: {
        ...state.stats,
        totalDistanceTraveled: state.stats.totalDistanceTraveled + 1,
      },
    },
    success: true,
    message: `Departing for ${port?.name ?? 'unknown port'}. ETA: ${travelTime} years.`,
  };
}

function handleTradeBuy(
  state: GameState,
  payload: { cardDefId: CardDefId; quantity: number },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const def = cardDefs.get(payload.cardDefId);
  if (!def) {
    return { state, success: false, message: 'Unknown card type' };
  }

  const port = state.world.ports[state.world.currentLocation];
  if (!port?.availableCards.includes(payload.cardDefId)) {
    return { state, success: false, message: 'Not available at this port' };
  }

  const basePrice = def.baseValue ?? 10;
  const modifier = port.marketModifiers[payload.cardDefId] ?? 1;
  const totalCost = Math.ceil(basePrice * modifier * payload.quantity);

  if (state.resources.credits < totalCost) {
    return { state, success: false, message: 'Insufficient credits' };
  }

  const newInstances: Record<CardInstanceId, CardInstance> = { ...state.cards.instances };
  const newCollection = [...state.cards.collection];

  for (let i = 0; i < payload.quantity; i++) {
    const instanceId = `card-${Date.now()}-${Math.random().toString(36).slice(2)}` as CardInstanceId;
    newInstances[instanceId] = {
      instanceId,
      cardDefId: payload.cardDefId,
      level: 1,
      condition: 100,
      mods: [],
      acquiredAt: { era: state.time.era, year: state.time.year },
      age: def.type === 'crew' ? 25 : undefined,
    };
    newCollection.push(instanceId);
  }

  return {
    state: {
      ...state,
      resources: {
        ...state.resources,
        credits: state.resources.credits - totalCost,
      },
      cards: {
        ...state.cards,
        instances: newInstances,
        collection: newCollection,
      },
      stats: {
        ...state.stats,
        cardsAcquired: state.stats.cardsAcquired + payload.quantity,
      },
    },
    success: true,
    message: `Purchased ${payload.quantity}x ${def.name} for ${totalCost} credits`,
  };
}

function handleTradeSell(
  state: GameState,
  payload: { instanceId: CardInstanceId },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const instance = state.cards.instances[payload.instanceId];
  if (!instance) {
    return { state, success: false, message: 'Card not found' };
  }

  const def = cardDefs.get(instance.cardDefId);
  if (!def) {
    return { state, success: false, message: 'Unknown card type' };
  }

  const port = state.world.ports[state.world.currentLocation];
  const basePrice = def.baseValue ?? 10;
  const modifier = port?.marketModifiers[instance.cardDefId] ?? 1;
  const sellPrice = Math.floor(basePrice * modifier * 0.7 * (instance.condition / 100));

  const newInstances = { ...state.cards.instances };
  delete newInstances[payload.instanceId];

  const newCollection = state.cards.collection.filter(id => id !== payload.instanceId);
  const newDeck = state.cards.deck.filter(id => id !== payload.instanceId);

  return {
    state: {
      ...state,
      resources: {
        ...state.resources,
        credits: state.resources.credits + sellPrice,
      },
      cards: {
        ...state.cards,
        instances: newInstances,
        collection: newCollection,
        deck: newDeck,
      },
      stats: {
        ...state.stats,
        totalCreditsEarned: state.stats.totalCreditsEarned + sellPrice,
      },
    },
    success: true,
    message: `Sold ${def.name} for ${sellPrice} credits`,
  };
}

function handleCardEquip(
  state: GameState,
  payload: { instanceId: CardInstanceId },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const instance = state.cards.instances[payload.instanceId];
  if (!instance) {
    return { state, success: false, message: 'Card not found' };
  }

  if (state.cards.deck.includes(payload.instanceId)) {
    return { state, success: false, message: 'Already equipped' };
  }

  const def = cardDefs.get(instance.cardDefId);
  const newActiveCrew = def?.type === 'crew' 
    ? [...state.cards.activeCrew, payload.instanceId]
    : state.cards.activeCrew;

  return {
    state: {
      ...state,
      cards: {
        ...state.cards,
        deck: [...state.cards.deck, payload.instanceId],
        collection: state.cards.collection.filter(id => id !== payload.instanceId),
        activeCrew: newActiveCrew,
      },
    },
    success: true,
  };
}

function handleCardUnequip(
  state: GameState,
  payload: { instanceId: CardInstanceId },
  _cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  if (!state.cards.deck.includes(payload.instanceId)) {
    return { state, success: false, message: 'Card not equipped' };
  }

  return {
    state: {
      ...state,
      cards: {
        ...state.cards,
        deck: state.cards.deck.filter(id => id !== payload.instanceId),
        collection: [...state.cards.collection, payload.instanceId],
        activeCrew: state.cards.activeCrew.filter(id => id !== payload.instanceId),
      },
    },
    success: true,
  };
}

function handleCardUpgrade(
  state: GameState,
  payload: { instanceId: CardInstanceId },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const instance = state.cards.instances[payload.instanceId];
  if (!instance) {
    return { state, success: false, message: 'Card not found' };
  }

  const def = cardDefs.get(instance.cardDefId);
  if (!def?.upgradesTo || !def.upgradeCost) {
    return { state, success: false, message: 'Card cannot be upgraded' };
  }

  const cost = def.upgradeCost;
  if ((cost.credits ?? 0) > state.resources.credits) {
    return { state, success: false, message: 'Insufficient credits' };
  }

  const newInstance: CardInstance = {
    ...instance,
    cardDefId: def.upgradesTo,
    level: instance.level + 1,
  };

  return {
    state: {
      ...state,
      resources: {
        ...state.resources,
        credits: state.resources.credits - (cost.credits ?? 0),
      },
      cards: {
        ...state.cards,
        instances: {
          ...state.cards.instances,
          [payload.instanceId]: newInstance,
        },
      },
    },
    success: true,
    message: `Upgraded to ${cardDefs.get(def.upgradesTo)?.name ?? 'unknown'}`,
  };
}

function handleModuleInstall(
  state: GameState,
  payload: { instanceId: CardInstanceId; slot: keyof typeof state.ship.modules },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const instance = state.cards.instances[payload.instanceId];
  if (!instance) {
    return { state, success: false, message: 'Module not found' };
  }

  const def = cardDefs.get(instance.cardDefId);
  if (def?.type !== 'module') {
    return { state, success: false, message: 'Not a module' };
  }

  if (state.ship.modules[payload.slot]) {
    return { state, success: false, message: 'Slot occupied' };
  }

  return {
    state: {
      ...state,
      ship: {
        ...state.ship,
        modules: {
          ...state.ship.modules,
          [payload.slot]: payload.instanceId,
        },
      },
      cards: {
        ...state.cards,
        collection: state.cards.collection.filter(id => id !== payload.instanceId),
      },
    },
    success: true,
  };
}

function handleModuleUninstall(
  state: GameState,
  payload: { slot: keyof typeof state.ship.modules },
  _cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const moduleId = state.ship.modules[payload.slot];
  if (!moduleId) {
    return { state, success: false, message: 'Slot empty' };
  }

  return {
    state: {
      ...state,
      ship: {
        ...state.ship,
        modules: {
          ...state.ship.modules,
          [payload.slot]: null,
        },
      },
      cards: {
        ...state.cards,
        collection: [...state.cards.collection, moduleId],
      },
    },
    success: true,
  };
}

function handleContractAccept(
  state: GameState,
  payload: { cardDefId: CardDefId },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const def = cardDefs.get(payload.cardDefId);
  if (!def || def.type !== 'contract') {
    return { state, success: false, message: 'Invalid contract' };
  }

  const instanceId = `contract-${Date.now()}` as CardInstanceId;
  const instance: CardInstance = {
    instanceId,
    cardDefId: payload.cardDefId,
    level: 1,
    condition: 100,
    mods: [],
    acquiredAt: { era: state.time.era, year: state.time.year },
    timeRemaining: def.contractTerms?.timeLimit,
  };

  return {
    state: {
      ...state,
      cards: {
        ...state.cards,
        instances: {
          ...state.cards.instances,
          [instanceId]: instance,
        },
        activeContracts: [...state.cards.activeContracts, instanceId],
      },
    },
    success: true,
    message: `Accepted contract: ${def.name}`,
  };
}

function handleContractComplete(
  state: GameState,
  payload: { instanceId: CardInstanceId },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const instance = state.cards.instances[payload.instanceId];
  if (!instance) {
    return { state, success: false, message: 'Contract not found' };
  }

  const def = cardDefs.get(instance.cardDefId);
  if (!def?.contractTerms) {
    return { state, success: false, message: 'Invalid contract' };
  }

  const reward = def.contractTerms.reward;
  const newInstances = { ...state.cards.instances };
  delete newInstances[payload.instanceId];

  return {
    state: {
      ...state,
      resources: {
        credits: state.resources.credits + (reward.credits ?? 0),
        fuel: state.resources.fuel + (reward.fuel ?? 0),
        supplies: state.resources.supplies + (reward.supplies ?? 0),
        hull: state.resources.hull,
        morale: Math.min(100, state.resources.morale + (reward.morale ?? 0)),
      },
      cards: {
        ...state.cards,
        instances: newInstances,
        activeContracts: state.cards.activeContracts.filter(id => id !== payload.instanceId),
      },
      stats: {
        ...state.stats,
        contractsCompleted: state.stats.contractsCompleted + 1,
      },
    },
    success: true,
    message: `Contract completed! Earned ${reward.credits ?? 0} credits`,
  };
}

function handleContractAbandon(
  state: GameState,
  payload: { instanceId: CardInstanceId },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const instance = state.cards.instances[payload.instanceId];
  if (!instance) {
    return { state, success: false, message: 'Contract not found' };
  }

  const def = cardDefs.get(instance.cardDefId);
  const penalty = def?.contractTerms?.penalty ?? {};
  
  const newInstances = { ...state.cards.instances };
  delete newInstances[payload.instanceId];

  return {
    state: {
      ...state,
      resources: {
        credits: Math.max(0, state.resources.credits - (penalty.credits ?? 0)),
        fuel: state.resources.fuel,
        supplies: state.resources.supplies,
        hull: state.resources.hull,
        morale: Math.max(0, state.resources.morale - (penalty.morale ?? 10)),
      },
      cards: {
        ...state.cards,
        instances: newInstances,
        activeContracts: state.cards.activeContracts.filter(id => id !== payload.instanceId),
      },
      stats: {
        ...state.stats,
        contractsFailed: state.stats.contractsFailed + 1,
      },
    },
    success: true,
    message: 'Contract abandoned',
  };
}

function handleCrewHire(
  state: GameState,
  payload: { cardDefId: CardDefId },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const def = cardDefs.get(payload.cardDefId);
  if (!def || def.type !== 'crew') {
    return { state, success: false, message: 'Invalid crew type' };
  }

  const hireCost = def.baseValue ?? 50;
  if (state.resources.credits < hireCost) {
    return { state, success: false, message: 'Insufficient credits' };
  }

  const instanceId = `crew-${Date.now()}` as CardInstanceId;
  const instance: CardInstance = {
    instanceId,
    cardDefId: payload.cardDefId,
    level: 1,
    condition: 100,
    mods: [],
    acquiredAt: { era: state.time.era, year: state.time.year },
    age: 20 + Math.floor(Math.random() * 20),
  };

  return {
    state: {
      ...state,
      resources: {
        ...state.resources,
        credits: state.resources.credits - hireCost,
      },
      cards: {
        ...state.cards,
        instances: {
          ...state.cards.instances,
          [instanceId]: instance,
        },
        activeCrew: [...state.cards.activeCrew, instanceId],
        deck: [...state.cards.deck, instanceId],
      },
      stats: {
        ...state.stats,
        cardsAcquired: state.stats.cardsAcquired + 1,
      },
    },
    success: true,
    message: `${def.name} joined the crew`,
  };
}

function handleCrewDismiss(
  state: GameState,
  payload: { instanceId: CardInstanceId },
  cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const instance = state.cards.instances[payload.instanceId];
  if (!instance) {
    return { state, success: false, message: 'Crew member not found' };
  }

  const def = cardDefs.get(instance.cardDefId);
  const newInstances = { ...state.cards.instances };
  delete newInstances[payload.instanceId];

  return {
    state: {
      ...state,
      cards: {
        ...state.cards,
        instances: newInstances,
        activeCrew: state.cards.activeCrew.filter(id => id !== payload.instanceId),
        deck: state.cards.deck.filter(id => id !== payload.instanceId),
      },
      resources: {
        ...state.resources,
        morale: Math.max(0, state.resources.morale - 5),
      },
    },
    success: true,
    message: `${def?.name ?? 'Crew member'} has left the ship`,
  };
}

function handleEventChoice(
  state: GameState,
  _payload: { sceneletId: string; choiceIndex: number },
  _cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  return { state, success: true };
}

function handleRepair(
  state: GameState,
  payload: { amount: number },
  _cardDefs: Map<string, CardDef>,
  config: SimulationConfig
): ActionResult {
  const cost = payload.amount * config.baseRepairCost;
  if (state.resources.credits < cost) {
    return { state, success: false, message: 'Insufficient credits' };
  }

  const newHull = Math.min(state.ship.maxHull, state.resources.hull + payload.amount);

  return {
    state: {
      ...state,
      resources: {
        ...state.resources,
        credits: state.resources.credits - cost,
        hull: newHull,
      },
    },
    success: true,
    message: `Repaired ${payload.amount} hull for ${cost} credits`,
  };
}

function handleResupply(
  state: GameState,
  payload: { amount: number },
  _cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const cost = payload.amount * 2;
  if (state.resources.credits < cost) {
    return { state, success: false, message: 'Insufficient credits' };
  }

  return {
    state: {
      ...state,
      resources: {
        ...state.resources,
        credits: state.resources.credits - cost,
        supplies: state.resources.supplies + payload.amount,
      },
    },
    success: true,
    message: `Purchased ${payload.amount} supplies for ${cost} credits`,
  };
}

function handleRefuel(
  state: GameState,
  payload: { amount: number },
  _cardDefs: Map<string, CardDef>,
  _config: SimulationConfig
): ActionResult {
  const cost = payload.amount * 3;
  if (state.resources.credits < cost) {
    return { state, success: false, message: 'Insufficient credits' };
  }

  return {
    state: {
      ...state,
      resources: {
        ...state.resources,
        credits: state.resources.credits - cost,
        fuel: state.resources.fuel + payload.amount,
      },
    },
    success: true,
    message: `Purchased ${payload.amount} fuel for ${cost} credits`,
  };
}
