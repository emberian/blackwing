import {
  type GameState,
  createId,
} from './types.js';

export function createInitialState(): GameState {
  const portHavenPrime = createId.port('port_haven_prime');
  const factionTraders = createId.faction('faction_free_traders');
  
  return {
    schemaVersion: 2,
    
    time: {
      cycle: 0,
      jumpsCompleted: 0,
    },
    
    ship: {
      name: 'Holdfast',
      class: 'Light Freighter',
      hull: 100,
      maxHull: 100,
      modules: {
        sensor: null,
        defense: null,
        cargo1: null,
        cargo2: null,
        propulsion: null,
        utility1: null,
        utility2: null,
      },
      baseCargoCapacity: 10,
    },
    
    resources: {
      credits: 80,
      fuel: 40,
      supplies: 25,
      hull: 100,
      morale: 70,
    },
    
    cards: {
      instances: {},
      collection: [],
      deck: [],
      activeCrew: [],
      activeContracts: [],
    },
    
    world: {
      currentLocation: portHavenPrime,
      ports: {
        [portHavenPrime]: {
          id: portHavenPrime,
          name: 'Haven Prime',
          description: 'A bustling trade hub at the crossroads of three shipping lanes. Old, but prosperous.',
          tags: ['tech', 'luxury'],
          faction: factionTraders,
          status: 'thriving',
          lastVisited: { cycle: 0 },
          marketModifiers: {
            [createId.cardDef('cargo_luxury_goods')]: 0.8,
            [createId.cardDef('cargo_processed_metals')]: 1.2,
          },
          availableCards: [
            createId.cardDef('cargo_raw_ore'),
            createId.cardDef('cargo_processed_metals'),
            createId.cardDef('cargo_medical_supplies'),
            createId.cardDef('cargo_luxury_goods'),
            createId.cardDef('crew_navigator'),
            createId.cardDef('crew_engineer'),
            createId.cardDef('crew_medic'),
            createId.cardDef('module_sensor_array'),
            createId.cardDef('module_expanded_hold'),
          ],
          availableContracts: [
            createId.cardDef('contract_standard_delivery'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_frontier_station')]: {
          id: createId.port('port_frontier_station'),
          name: 'Frontier Station 7',
          description: 'A remote outpost on the edge of charted space. They need everything.',
          tags: ['medicine', 'survival'],
          faction: createId.faction('faction_frontier_alliance'),
          status: 'declining',
          marketModifiers: {
            [createId.cardDef('cargo_medical_supplies')]: 2.5,
            [createId.cardDef('cargo_raw_ore')]: 0.5,
          },
          availableCards: [
            createId.cardDef('cargo_cryo_seeds'),
            createId.cardDef('crew_stowaway'),
            createId.cardDef('crew_gunner'),
          ],
          availableContracts: [
            createId.cardDef('contract_medical_emergency'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_shadow_market')]: {
          id: createId.port('port_shadow_market'),
          name: 'Shadow Market',
          description: 'It moves. It has no name on any chart. If you need to find it, you already know where it is.',
          tags: ['contraband', 'ancient'],
          faction: null,
          status: 'stable',
          marketModifiers: {
            [createId.cardDef('cargo_contraband')]: 0.8,
            [createId.cardDef('cargo_ancient_artifacts')]: 1.5,
          },
          availableCards: [
            createId.cardDef('cargo_contraband'),
            createId.cardDef('cargo_volatile_isotopes'),
            createId.cardDef('cargo_memory_cores'),
            createId.cardDef('crew_ai_fragment'),
            createId.cardDef('module_point_defense'),
          ],
          availableContracts: [
            createId.cardDef('contract_discrete_cargo'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_industrial_complex')]: {
          id: createId.port('port_industrial_complex'),
          name: 'Crucible Station',
          description: 'A massive orbital foundry. The fires never stop. Neither does the demand for raw materials.',
          tags: ['mineral', 'tech'],
          faction: createId.faction('faction_industrial_consortium'),
          status: 'thriving',
          marketModifiers: {
            [createId.cardDef('cargo_raw_ore')]: 1.8,
            [createId.cardDef('cargo_processed_metals')]: 0.7,
            [createId.cardDef('cargo_starship_components')]: 0.6,
          },
          availableCards: [
            createId.cardDef('cargo_processed_metals'),
            createId.cardDef('cargo_starship_components'),
            createId.cardDef('crew_engineer'),
            createId.cardDef('module_reinforced_hull'),
            createId.cardDef('module_efficient_drives'),
          ],
          availableContracts: [],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_sanctuary')]: {
          id: createId.port('port_sanctuary'),
          name: 'The Sanctuary',
          description: 'A haven for the weary. Neutral ground. All are welcome who come in peace.',
          tags: ['medicine', 'cultural'],
          faction: null,
          status: 'stable',
          marketModifiers: {
            [createId.cardDef('cargo_medical_supplies')]: 0.9,
            [createId.cardDef('cargo_luxury_goods')]: 1.3,
          },
          availableCards: [
            createId.cardDef('cargo_medical_supplies'),
            createId.cardDef('cargo_luxury_goods'),
            createId.cardDef('crew_medic'),
            createId.cardDef('crew_quartermaster'),
            createId.cardDef('module_cryo_bay'),
          ],
          availableContracts: [],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_research_station')]: {
          id: createId.port('port_research_station'),
          name: 'Axiom Observatory',
          description: 'Scientists peer into the void here, cataloging anomalies. They pay well for specimens.',
          tags: ['data', 'ancient'],
          faction: createId.faction('faction_science_collective'),
          status: 'stable',
          marketModifiers: {
            [createId.cardDef('cargo_memory_cores')]: 1.4,
            [createId.cardDef('cargo_living_specimens')]: 2.0,
            [createId.cardDef('cargo_ancient_artifacts')]: 2.5,
          },
          availableCards: [
            createId.cardDef('cargo_memory_cores'),
            createId.cardDef('crew_navigator'),
            createId.cardDef('module_sensor_array'),
          ],
          availableContracts: [],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_pirate_haven')]: {
          id: createId.port('port_pirate_haven'),
          name: 'Freeport Omega',
          description: 'No laws. No questions. No guarantees. Bring credits and watch your back.',
          tags: ['contraband', 'weapon'],
          faction: null,
          status: 'declining',
          marketModifiers: {
            [createId.cardDef('cargo_contraband')]: 1.5,
            [createId.cardDef('cargo_volatile_isotopes')]: 0.7,
          },
          availableCards: [
            createId.cardDef('cargo_contraband'),
            createId.cardDef('cargo_volatile_isotopes'),
            createId.cardDef('crew_gunner'),
            createId.cardDef('crew_stowaway'),
            createId.cardDef('module_point_defense'),
          ],
          availableContracts: [],
          marketRefreshedAt: { cycle: 0 },
        },
      },
      factions: {
        [factionTraders]: {
          id: factionTraders,
          name: 'Free Traders Guild',
          reputation: 10,
          flags: {},
        },
        [createId.faction('faction_frontier_alliance')]: {
          id: createId.faction('faction_frontier_alliance'),
          name: 'Frontier Alliance',
          reputation: 0,
          flags: {},
        },
        [createId.faction('faction_industrial_consortium')]: {
          id: createId.faction('faction_industrial_consortium'),
          name: 'Industrial Consortium',
          reputation: 0,
          flags: {},
        },
        [createId.faction('faction_science_collective')]: {
          id: createId.faction('faction_science_collective'),
          name: 'Science Collective',
          reputation: 0,
          flags: {},
        },
      },
      knownPorts: [
        portHavenPrime,
        createId.port('port_frontier_station'),
        createId.port('port_industrial_complex'),
      ],
      worldFlags: {},
    },
    
    chronicle: [
      {
        id: createId.chronicleEntry('genesis'),
        type: 'milestone',
        timestamp: { cycle: 0 },
        title: 'The Beginning',
        text: 'The hold is cold. The engines hum. The void waits.',
        tags: ['start'],
      },
    ],
    
    achievements: {
      unlocked: [],
      unlockedAt: {},
    },
    
    flags: {},
    
    stats: {
      totalCreditsEarned: 0,
      totalDistanceTraveled: 0,
      portsVisited: 1,
      cardsAcquired: 0,
      crewLost: 0,
      contractsCompleted: 0,
      contractsFailed: 0,
    },
    
    rngSeed: Date.now(),
    rngState: Date.now(),
  };
}
