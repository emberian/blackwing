/**
 * CARGO HOLD - Core Type Definitions
 * 
 * The fundamental types that define the game's state and mechanics.
 * Everything flows from these types.
 */

// =============================================================================
// IDENTIFIERS
// =============================================================================

/** Unique identifier for card definitions (static data) */
export type CardDefId = string & { readonly __brand: 'CardDefId' };

/** Unique identifier for card instances (player-owned) */
export type CardInstanceId = string & { readonly __brand: 'CardInstanceId' };

/** Unique identifier for ports/locations */
export type PortId = string & { readonly __brand: 'PortId' };

/** Unique identifier for factions */
export type FactionId = string & { readonly __brand: 'FactionId' };

/** Unique identifier for scenelets */
export type SceneletId = string & { readonly __brand: 'SceneletId' };

/** Unique identifier for chronicle entries */
export type ChronicleEntryId = string & { readonly __brand: 'ChronicleEntryId' };

// Helper to create branded IDs
export const createId = {
  cardDef: (id: string): CardDefId => id as CardDefId,
  cardInstance: (id: string): CardInstanceId => id as CardInstanceId,
  port: (id: string): PortId => id as PortId,
  faction: (id: string): FactionId => id as FactionId,
  scenelet: (id: string): SceneletId => id as SceneletId,
  chronicleEntry: (id: string): ChronicleEntryId => id as ChronicleEntryId,
};

// =============================================================================
// CARDS
// =============================================================================

export type CardType = 'cargo' | 'crew' | 'module' | 'contract' | 'echo';

export type CardRarity = 'common' | 'uncommon' | 'rare' | 'legendary';

/** Tags for card categorization and event matching */
export type CardTag = 
  | 'organic' | 'mineral' | 'tech' | 'data' | 'contraband' | 'luxury' | 'medicine'
  | 'ancient' | 'volatile' | 'living' | 'frozen' | 'weapon' | 'cultural'
  | 'navigation' | 'engineering' | 'medical' | 'combat' | 'social' | 'survival'
  | 'sensor' | 'defense' | 'cargo' | 'life-support' | 'propulsion'
  | 'delivery' | 'smuggling' | 'rescue' | 'exploration' | 'diplomacy'
  | 'memory' | 'artifact' | 'lineage';

/** Static card definition - what a card IS */
export interface CardDef {
  readonly id: CardDefId;
  readonly type: CardType;
  readonly name: string;
  readonly description: string;
  readonly flavorText?: string;
  readonly rarity: CardRarity;
  readonly tags: readonly CardTag[];
  
  /** Base effects when card is active/equipped */
  readonly effects: CardEffects;
  
  /** How this card behaves during idle ticks */
  readonly idleBehavior?: IdleBehavior;
  
  /** Upgrade path if any */
  readonly upgradesTo?: CardDefId;
  readonly upgradeCost?: ResourceBundle;
  
  /** For cargo: base trade value */
  readonly baseValue?: number;
  
  /** For crew: lifespan in game-years (-1 for immortal like AI) */
  readonly lifespan?: number;
  
  /** For contracts: requirements and rewards */
  readonly contractTerms?: ContractTerms;
  
  /** For modules: installation requirements */
  readonly installRequirements?: InstallRequirements;
}

export interface CardEffects {
  /** Flat resource modifiers per tick */
  readonly resourcesPerTick?: Partial<Resources>;
  
  /** Percentage modifiers to various stats */
  readonly modifiers?: {
    readonly creditMultiplier?: number;
    readonly fuelEfficiency?: number;
    readonly cargoCapacity?: number;
    readonly jumpSpeed?: number;
    readonly morale?: number;
    readonly hullIntegrity?: number;
  };
  
  /** Tags this card adds to the ship (for event matching) */
  readonly grantsShipTags?: readonly CardTag[];
  
  /** Unlocks certain actions or features */
  readonly unlocks?: readonly string[];
}

export interface IdleBehavior {
  /** Resource generation per tick when active */
  readonly generates?: Partial<Resources>;
  
  /** Resource consumption per tick */
  readonly consumes?: Partial<Resources>;
  
  /** Decay rate (0-1) per tick - for perishables */
  readonly decayRate?: number;
  
  /** Chance per tick to trigger special event */
  readonly eventChance?: number;
  readonly eventPool?: readonly SceneletId[];
}

export interface ContractTerms {
  readonly destination: PortId;
  readonly cargoRequired?: { cardDefId: CardDefId; quantity: number };
  readonly timeLimit: number; // in game-years
  readonly reward: ResourceBundle;
  readonly penalty?: ResourceBundle;
  readonly reputationReward?: { faction: FactionId; amount: number };
}

export interface InstallRequirements {
  readonly minHull?: number;
  readonly requiredModules?: readonly CardDefId[];
  readonly excludesModules?: readonly CardDefId[];
  readonly slotType: 'sensor' | 'defense' | 'cargo' | 'propulsion' | 'utility';
}

/** A card instance - what the player actually owns */
export interface CardInstance {
  readonly instanceId: CardInstanceId;
  readonly cardDefId: CardDefId;
  readonly level: number;
  readonly condition: number;
  readonly mods: readonly CardMod[];
  readonly acquiredAt: GameTimestamp;
  
  /** For crew: current age in years */
  readonly age?: number | undefined;
  
  /** For contracts: time remaining */
  readonly timeRemaining?: number | undefined;
  
  /** Custom data for special cards */
  readonly customData?: Record<string, unknown> | undefined;
}

export interface CardMod {
  readonly type: string;
  readonly value: number;
  readonly source: string;
}

// =============================================================================
// RESOURCES
// =============================================================================

export interface Resources {
  credits: number;
  fuel: number;
  supplies: number;
  hull: number;
  morale: number;
}

export type ResourceBundle = Partial<Resources>;

// =============================================================================
// TIME
// =============================================================================

export interface GameTimestamp {
  readonly era: number;
  readonly year: number;
}

export interface TimeState {
  /** Real-world timestamp of last simulation */
  lastSimulatedAt: number;
  
  /** Current game era (major epoch) */
  era: number;
  
  /** Year within current era */
  year: number;
  
  /** Ticks elapsed (smallest unit) */
  ticks: number;
  
  /** Is the ship currently in transit? */
  inTransit: boolean;
  
  /** If in transit, destination and ETA */
  transitDestination?: PortId | undefined;
  transitDepartedAt?: GameTimestamp | undefined;
  transitArrivesAt?: GameTimestamp | undefined;
}

// =============================================================================
// SHIP
// =============================================================================

export interface ShipState {
  name: string;
  class: string;
  hull: number;
  maxHull: number;
  
  /** Module slots and what's installed */
  modules: {
    readonly sensor: CardInstanceId | null;
    readonly defense: CardInstanceId | null;
    readonly cargo1: CardInstanceId | null;
    readonly cargo2: CardInstanceId | null;
    readonly propulsion: CardInstanceId | null;
    readonly utility1: CardInstanceId | null;
    readonly utility2: CardInstanceId | null;
  };
  
  /** Base cargo capacity (modified by modules) */
  baseCargoCapacity: number;
}

// =============================================================================
// WORLD
// =============================================================================

export interface PortState {
  readonly id: PortId;
  readonly name: string;
  readonly description: string;
  readonly tags: readonly CardTag[];
  readonly faction: FactionId | null;
  
  /** Current state - can change between visits */
  status: 'thriving' | 'stable' | 'declining' | 'ruined' | 'abandoned' | 'unknown';
  
  /** Last visited timestamp (if ever) */
  lastVisited?: GameTimestamp;
  
  /** Market prices - deviation from base */
  marketModifiers: Record<CardDefId, number>;
  
  /** Available cards to purchase */
  availableCards: CardDefId[];
  
  /** Available contracts */
  availableContracts: CardDefId[];
  
  /** When the market last refreshed */
  marketRefreshedAt: GameTimestamp;
}

export interface FactionState {
  readonly id: FactionId;
  readonly name: string;
  reputation: number; // -100 to 100
  
  /** Flags tracking faction-specific story progress */
  flags: Record<string, boolean | number | string>;
}

export interface WorldState {
  currentLocation: PortId;
  ports: Record<PortId, PortState>;
  factions: Record<FactionId, FactionState>;
  
  /** Discovered but not yet visited */
  knownPorts: PortId[];
  
  /** Global world flags */
  worldFlags: Record<string, boolean | number | string>;
}

// =============================================================================
// CARDS COLLECTION
// =============================================================================

export interface CardsState {
  /** All card instances owned by player */
  instances: Record<CardInstanceId, CardInstance>;
  
  /** Cards in the "collection" (not actively equipped) */
  collection: CardInstanceId[];
  
  /** Currently equipped/active cards */
  deck: CardInstanceId[];
  
  /** Crew roster (subset of deck that are crew type) */
  activeCrew: CardInstanceId[];
  
  /** Active contracts */
  activeContracts: CardInstanceId[];
}

// =============================================================================
// CHRONICLE (Narrative Log)
// =============================================================================

export type ChronicleEntryType = 
  | 'arrival' | 'departure' | 'trade' | 'acquisition' | 'loss'
  | 'crew_event' | 'encounter' | 'discovery' | 'contract' | 'death'
  | 'era_change' | 'milestone';

export interface ChronicleEntry {
  readonly id: ChronicleEntryId;
  readonly type: ChronicleEntryType;
  readonly timestamp: GameTimestamp;
  readonly title: string;
  readonly text: string;
  readonly tags: readonly string[];
  
  /** References to entities involved */
  readonly refs?: {
    readonly portId?: PortId;
    readonly cardIds?: readonly CardInstanceId[];
    readonly factionId?: FactionId;
  };
}

// =============================================================================
// GAME STATE (Top Level)
// =============================================================================

export interface GameState {
  readonly schemaVersion: number;
  
  time: TimeState;
  ship: ShipState;
  resources: Resources;
  cards: CardsState;
  world: WorldState;
  chronicle: ChronicleEntry[];
  
  /** Narrative flags for event tracking */
  flags: Record<string, boolean | number | string>;
  
  /** Statistics for achievements/tracking */
  stats: GameStats;
  
  /** RNG state for reproducibility */
  rngSeed: number;
  rngState: number;
}

export interface GameStats {
  totalCreditsEarned: number;
  totalDistanceTraveled: number;
  portsVisited: number;
  cardsAcquired: number;
  crewLost: number;
  contractsCompleted: number;
  contractsFailed: number;
  erasSurvived: number;
}

// =============================================================================
// ACTIONS (Player Input)
// =============================================================================

export type GameAction =
  | { type: 'TICK'; payload: { deltaMs: number } }
  | { type: 'TRAVEL'; payload: { destination: PortId } }
  | { type: 'TRADE_BUY'; payload: { cardDefId: CardDefId; quantity: number } }
  | { type: 'TRADE_SELL'; payload: { instanceId: CardInstanceId } }
  | { type: 'CARD_EQUIP'; payload: { instanceId: CardInstanceId } }
  | { type: 'CARD_UNEQUIP'; payload: { instanceId: CardInstanceId } }
  | { type: 'CARD_UPGRADE'; payload: { instanceId: CardInstanceId } }
  | { type: 'MODULE_INSTALL'; payload: { instanceId: CardInstanceId; slot: keyof ShipState['modules'] } }
  | { type: 'MODULE_UNINSTALL'; payload: { slot: keyof ShipState['modules'] } }
  | { type: 'CONTRACT_ACCEPT'; payload: { cardDefId: CardDefId } }
  | { type: 'CONTRACT_COMPLETE'; payload: { instanceId: CardInstanceId } }
  | { type: 'CONTRACT_ABANDON'; payload: { instanceId: CardInstanceId } }
  | { type: 'CREW_HIRE'; payload: { cardDefId: CardDefId } }
  | { type: 'CREW_DISMISS'; payload: { instanceId: CardInstanceId } }
  | { type: 'EVENT_CHOICE'; payload: { sceneletId: SceneletId; choiceIndex: number } }
  | { type: 'REPAIR'; payload: { amount: number } }
  | { type: 'RESUPPLY'; payload: { amount: number } }
  | { type: 'REFUEL'; payload: { amount: number } };

// =============================================================================
// EVENTS / SCENELETS
// =============================================================================

export interface Scenelet {
  readonly id: SceneletId;
  readonly title: string;
  readonly tags: readonly string[];
  
  /** Requirements to appear */
  readonly requirements: SceneletRequirements;
  
  /** Weight for random selection (higher = more likely) */
  readonly weight: number;
  
  /** Cooldown in game-years before can appear again */
  readonly cooldown: number;
  
  /** The narrative content */
  readonly passages: SceneletPassage[];
}

export interface SceneletRequirements {
  /** Must be in transit / at port */
  readonly location?: 'transit' | 'port' | 'any';
  
  /** Must have these ship tags (from modules/cargo) */
  readonly shipTags?: readonly CardTag[];
  
  /** Must have crew with these tags */
  readonly crewTags?: readonly CardTag[];
  
  /** Must have cargo with these tags */
  readonly cargoTags?: readonly CardTag[];
  
  /** Resource thresholds */
  readonly minResources?: Partial<Resources>;
  readonly maxResources?: Partial<Resources>;
  
  /** Reputation requirements */
  readonly factionRep?: { faction: FactionId; min?: number; max?: number };
  
  /** Must have these flags set */
  readonly requiredFlags?: readonly string[];
  
  /** Must NOT have these flags set */
  readonly excludedFlags?: readonly string[];
  
  /** Custom predicate (serialized as string, evaluated at runtime) */
  readonly customPredicate?: string;
}

export interface SceneletPassage {
  readonly text: string;
  readonly choices?: SceneletChoice[];
}

export interface SceneletChoice {
  readonly text: string;
  readonly requirements?: Partial<SceneletRequirements>;
  readonly effects: SceneletEffects;
  readonly nextPassage?: number; // index into passages array
}

export interface SceneletEffects {
  readonly resources?: Partial<Resources>;
  readonly addCards?: readonly CardDefId[];
  readonly removeCards?: readonly CardInstanceId[];
  readonly setFlags?: Record<string, boolean | number | string>;
  readonly addChronicle?: { title: string; text: string };
  readonly reputation?: { faction: FactionId; amount: number };
  readonly triggerScenelet?: SceneletId;
  
  /** Damage to ship/crew */
  readonly damage?: { hull?: number; morale?: number; crewCasualties?: number };
}

// =============================================================================
// SIMULATION CONFIG
// =============================================================================

export interface SimulationConfig {
  /** Real-world milliseconds per game tick */
  readonly msPerTick: number;
  
  /** Game ticks per game year */
  readonly ticksPerYear: number;
  
  /** Maximum ticks to simulate in one catch-up (prevents lag) */
  readonly maxCatchUpTicks: number;
  
  /** Base costs */
  readonly baseFuelPerJump: number;
  readonly baseSuppliesPerTick: number;
  readonly baseRepairCost: number;
  
  /** Decay/degradation rates */
  readonly hullDecayPerTick: number;
  readonly moraleDecayPerTick: number;
  readonly cargoDecayChance: number;
  
  /** Event chances per tick */
  readonly transitEventChance: number;
  readonly portEventChance: number;
}

export const DEFAULT_CONFIG: SimulationConfig = {
  msPerTick: 60_000, // 1 minute real time = 1 tick
  ticksPerYear: 60, // 1 hour real time = 1 game year
  maxCatchUpTicks: 1440, // Max 24 hours of catch-up
  
  baseFuelPerJump: 10,
  baseSuppliesPerTick: 0.1,
  baseRepairCost: 5,
  
  hullDecayPerTick: 0.01,
  moraleDecayPerTick: 0.05,
  cargoDecayChance: 0.001,
  
  transitEventChance: 0.02,
  portEventChance: 0.05,
};
