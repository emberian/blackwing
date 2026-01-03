import type { CardDef, CardDefId } from '../../core/types.js';

const id = (s: string): CardDefId => s as CardDefId;

export const CARGO_CARDS: CardDef[] = [
  {
    id: id('cargo_raw_ore'),
    type: 'cargo',
    name: 'Raw Ore',
    description: 'Unprocessed minerals from asteroid belts. Heavy but valuable to industrial ports.',
    rarity: 'common',
    tags: ['mineral'],
    effects: {},
    baseValue: 15,
    idleBehavior: {
      generates: { credits: 0.1 },
    },
  },
  {
    id: id('cargo_processed_metals'),
    type: 'cargo',
    name: 'Processed Metals',
    description: 'Refined alloys ready for construction. The backbone of civilization.',
    rarity: 'common',
    tags: ['mineral', 'tech'],
    effects: {},
    baseValue: 35,
    upgradesTo: id('cargo_starship_components'),
    upgradeCost: { credits: 100 },
  },
  {
    id: id('cargo_starship_components'),
    type: 'cargo',
    name: 'Starship Components',
    description: 'Precision-manufactured parts. Worth their weight in decades.',
    rarity: 'uncommon',
    tags: ['tech'],
    effects: {},
    baseValue: 120,
  },
  {
    id: id('cargo_cryo_seeds'),
    type: 'cargo',
    name: 'Cryo-Seeds',
    description: 'Genetic repository of a thousand worlds. Life, suspended.',
    flavorText: 'Each seed is a forest waiting to breathe.',
    rarity: 'uncommon',
    tags: ['organic', 'frozen'],
    effects: {},
    baseValue: 80,
    idleBehavior: {
      decayRate: 0.005,
    },
  },
  {
    id: id('cargo_medical_supplies'),
    type: 'cargo',
    name: 'Medical Supplies',
    description: 'Nanopharm kits and tissue regenerators. Mercy in a crate.',
    rarity: 'common',
    tags: ['medicine', 'tech'],
    effects: {
      modifiers: { morale: 0.05 },
    },
    baseValue: 45,
    idleBehavior: {
      decayRate: 0.002,
    },
  },
  {
    id: id('cargo_memory_cores'),
    type: 'cargo',
    name: 'Memory Cores',
    description: 'Crystalline data stores containing centuries of knowledge.',
    flavorText: 'The thoughts of the dead, preserved in light.',
    rarity: 'rare',
    tags: ['data', 'ancient'],
    effects: {
      grantsShipTags: ['data'],
    },
    baseValue: 200,
  },
  {
    id: id('cargo_luxury_goods'),
    type: 'cargo',
    name: 'Luxury Goods',
    description: 'Art, wine, textiles—remnants of cultures long silent.',
    rarity: 'uncommon',
    tags: ['luxury', 'cultural'],
    effects: {},
    baseValue: 90,
    idleBehavior: {
      generates: { morale: 0.02 },
    },
  },
  {
    id: id('cargo_contraband'),
    type: 'cargo',
    name: 'Unmarked Containers',
    description: 'You did not ask. They did not say.',
    flavorText: 'Some cargo is best not inventoried.',
    rarity: 'uncommon',
    tags: ['contraband'],
    effects: {},
    baseValue: 150,
    idleBehavior: {
      eventChance: 0.01,
    },
  },
  {
    id: id('cargo_volatile_isotopes'),
    type: 'cargo',
    name: 'Volatile Isotopes',
    description: 'Unstable materials with extreme energy potential. Handle with reverence.',
    rarity: 'rare',
    tags: ['volatile', 'tech'],
    effects: {},
    baseValue: 300,
    idleBehavior: {
      decayRate: 0.01,
      eventChance: 0.005,
    },
  },
  {
    id: id('cargo_living_specimens'),
    type: 'cargo',
    name: 'Living Specimens',
    description: 'Creatures in stasis. Some from worlds that no longer exist.',
    rarity: 'rare',
    tags: ['living', 'organic', 'frozen'],
    effects: {},
    baseValue: 180,
    idleBehavior: {
      consumes: { supplies: 0.05 },
      decayRate: 0.003,
    },
  },
  {
    id: id('cargo_ancient_artifacts'),
    type: 'cargo',
    name: 'Ancient Artifacts',
    description: 'Relics of Prior civilizations. Purpose unknown. Value incalculable.',
    flavorText: 'They watched the first stars die.',
    rarity: 'legendary',
    tags: ['ancient', 'artifact'],
    effects: {
      grantsShipTags: ['ancient'],
    },
    baseValue: 500,
  },
];

export const CREW_CARDS: CardDef[] = [
  {
    id: id('crew_navigator'),
    type: 'crew',
    name: 'Navigator',
    description: 'Charts courses through the long dark. Essential for efficient travel.',
    rarity: 'common',
    tags: ['navigation'],
    effects: {
      modifiers: { jumpSpeed: 0.1 },
    },
    baseValue: 50,
    lifespan: 80,
  },
  {
    id: id('crew_engineer'),
    type: 'crew',
    name: 'Engineer',
    description: 'Keeps the hold together when the void tries to tear it apart.',
    rarity: 'common',
    tags: ['engineering'],
    effects: {
      modifiers: { hullIntegrity: 0.1, fuelEfficiency: 0.05 },
    },
    baseValue: 50,
    lifespan: 75,
  },
  {
    id: id('crew_medic'),
    type: 'crew',
    name: 'Medic',
    description: 'Heals bodies. Sometimes heals minds. Always in demand.',
    rarity: 'common',
    tags: ['medical'],
    effects: {
      modifiers: { morale: 0.1 },
    },
    baseValue: 60,
    lifespan: 85,
  },
  {
    id: id('crew_quartermaster'),
    type: 'crew',
    name: 'Quartermaster',
    description: 'Knows every crate, every manifest, every hidden corner of the hold.',
    rarity: 'uncommon',
    tags: ['cargo', 'social'],
    effects: {
      modifiers: { cargoCapacity: 0.15, creditMultiplier: 0.05 },
    },
    baseValue: 70,
    lifespan: 70,
  },
  {
    id: id('crew_gunner'),
    type: 'crew',
    name: 'Gunner',
    description: 'The void has teeth. So does the hold.',
    rarity: 'uncommon',
    tags: ['combat'],
    effects: {
      grantsShipTags: ['combat'],
    },
    baseValue: 65,
    lifespan: 60,
  },
  {
    id: id('crew_ai_fragment'),
    type: 'crew',
    name: 'AI Fragment',
    description: 'A shard of ancient machine intelligence. Helpful. Probably.',
    flavorText: 'It remembers things that never happened.',
    rarity: 'rare',
    tags: ['tech', 'ancient'],
    effects: {
      modifiers: { jumpSpeed: 0.15, fuelEfficiency: 0.1 },
      grantsShipTags: ['tech'],
    },
    baseValue: 200,
    lifespan: -1,
  },
  {
    id: id('crew_stowaway'),
    type: 'crew',
    name: 'Stowaway',
    description: 'Found in the hold after departure. Background unknown.',
    flavorText: 'Everyone is running from something.',
    rarity: 'common',
    tags: ['survival'],
    effects: {},
    baseValue: 0,
    lifespan: 70,
  },
];

export const MODULE_CARDS: CardDef[] = [
  {
    id: id('module_sensor_array'),
    type: 'module',
    name: 'Sensor Array',
    description: 'See further into the dark. Detect opportunities. Avoid threats.',
    rarity: 'common',
    tags: ['sensor'],
    effects: {
      grantsShipTags: ['sensor'],
      unlocks: ['scan_anomaly', 'detect_signal'],
    },
    baseValue: 100,
    installRequirements: {
      slotType: 'sensor',
    },
  },
  {
    id: id('module_point_defense'),
    type: 'module',
    name: 'Point Defense',
    description: 'Automated turrets that discourage the desperate.',
    rarity: 'uncommon',
    tags: ['defense', 'weapon'],
    effects: {
      grantsShipTags: ['defense'],
    },
    baseValue: 150,
    installRequirements: {
      slotType: 'defense',
    },
  },
  {
    id: id('module_expanded_hold'),
    type: 'module',
    name: 'Expanded Hold',
    description: 'More space. More cargo. More possibilities.',
    rarity: 'common',
    tags: ['cargo'],
    effects: {
      modifiers: { cargoCapacity: 0.25 },
    },
    baseValue: 120,
    installRequirements: {
      slotType: 'cargo',
    },
  },
  {
    id: id('module_cryo_bay'),
    type: 'module',
    name: 'Cryo Bay',
    description: 'Sleep through the centuries. Wake as if no time has passed.',
    flavorText: 'Death deferred is not life preserved.',
    rarity: 'uncommon',
    tags: ['life-support', 'frozen'],
    effects: {
      unlocks: ['cryo_sleep'],
      grantsShipTags: ['frozen'],
    },
    baseValue: 200,
    installRequirements: {
      slotType: 'utility',
    },
  },
  {
    id: id('module_efficient_drives'),
    type: 'module',
    name: 'Efficient Drives',
    description: 'Squeeze more distance from every unit of fuel.',
    rarity: 'common',
    tags: ['propulsion'],
    effects: {
      modifiers: { fuelEfficiency: 0.2 },
    },
    baseValue: 130,
    installRequirements: {
      slotType: 'propulsion',
    },
  },
  {
    id: id('module_reinforced_hull'),
    type: 'module',
    name: 'Reinforced Hull',
    description: 'Extra plating against the void\'s indifference.',
    rarity: 'uncommon',
    tags: ['defense'],
    effects: {
      modifiers: { hullIntegrity: 0.2 },
    },
    baseValue: 140,
    installRequirements: {
      slotType: 'utility',
    },
  },
];

export const CONTRACT_CARDS: CardDef[] = [
  {
    id: id('contract_standard_delivery'),
    type: 'contract',
    name: 'Standard Delivery',
    description: 'Move cargo from here to there. Simple work.',
    rarity: 'common',
    tags: ['delivery'],
    effects: {},
    contractTerms: {
      destination: 'port_haven_prime' as any,
      timeLimit: 100,
      reward: { credits: 200 },
    },
  },
  {
    id: id('contract_medical_emergency'),
    type: 'contract',
    name: 'Medical Emergency',
    description: 'Frontier colony needs supplies. Lives hang in the balance.',
    rarity: 'uncommon',
    tags: ['delivery', 'medicine'],
    effects: {},
    contractTerms: {
      destination: 'port_frontier_station' as any,
      cargoRequired: { cardDefId: id('cargo_medical_supplies'), quantity: 3 },
      timeLimit: 50,
      reward: { credits: 500, morale: 10 },
      penalty: { morale: -20 },
      reputationReward: { faction: 'faction_frontier_alliance' as any, amount: 15 },
    },
  },
  {
    id: id('contract_discrete_cargo'),
    type: 'contract',
    name: 'Discrete Cargo',
    description: 'No questions. No manifests. Good pay.',
    flavorText: 'Some employers value silence above all.',
    rarity: 'uncommon',
    tags: ['smuggling', 'contraband'],
    effects: {},
    contractTerms: {
      destination: 'port_shadow_market' as any,
      timeLimit: 75,
      reward: { credits: 400 },
      penalty: { credits: -100 },
    },
  },
];

export const ECHO_CARDS: CardDef[] = [
  {
    id: id('echo_founders_manifest'),
    type: 'echo',
    name: 'Founder\'s Manifest',
    description: 'The original cargo list. Your ship\'s first cargo, centuries ago.',
    flavorText: 'Names of things that no longer exist.',
    rarity: 'legendary',
    tags: ['memory', 'lineage'],
    effects: {
      modifiers: { creditMultiplier: 0.05 },
    },
  },
  {
    id: id('echo_star_chart_fragment'),
    type: 'echo',
    name: 'Star Chart Fragment',
    description: 'Part of an ancient navigation system. Points to somewhere.',
    rarity: 'rare',
    tags: ['ancient', 'navigation'],
    effects: {
      unlocks: ['secret_route'],
    },
  },
  {
    id: id('echo_line_token'),
    type: 'echo',
    name: 'Line Token',
    description: 'Recognition from a Shatterling Line. You are known.',
    flavorText: 'The Lines remember across millennia.',
    rarity: 'legendary',
    tags: ['lineage', 'artifact'],
    effects: {},
  },
];

export const ALL_CARDS: CardDef[] = [
  ...CARGO_CARDS,
  ...CREW_CARDS,
  ...MODULE_CARDS,
  ...CONTRACT_CARDS,
  ...ECHO_CARDS,
];

export function buildCardDefMap(cards: CardDef[] = ALL_CARDS): Map<string, CardDef> {
  const map = new Map<string, CardDef>();
  for (const card of cards) {
    map.set(card.id, card);
  }
  return map;
}
