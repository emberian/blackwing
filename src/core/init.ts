import {
  type GameState,
  createId,
} from './types.js';

export function createInitialState(): GameState {
  const portHavenPrime = createId.port('port_haven_prime');
  const factionTraders = createId.faction('faction_free_traders');
  
  return {
    schemaVersion: 1,
    
    time: {
      lastSimulatedAt: Date.now(),
      era: 1,
      year: 0,
      ticks: 0,
      inTransit: false,
      transitDestination: undefined,
      transitDepartedAt: undefined,
      transitArrivesAt: undefined,
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
      credits: 100,
      fuel: 50,
      supplies: 30,
      hull: 100,
      morale: 75,
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
          lastVisited: { era: 1, year: 0 },
          marketModifiers: {},
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
          marketRefreshedAt: { era: 1, year: 0 },
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
          marketRefreshedAt: { era: 1, year: 0 },
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
          marketRefreshedAt: { era: 1, year: 0 },
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
      },
      knownPorts: [
        portHavenPrime,
        createId.port('port_frontier_station'),
      ],
      worldFlags: {},
    },
    
    chronicle: [
      {
        id: createId.chronicleEntry('genesis'),
        type: 'milestone',
        timestamp: { era: 1, year: 0 },
        title: 'The Beginning',
        text: 'The hold is cold. The engines hum. The void waits.',
        tags: ['start'],
      },
    ],
    
    flags: {},
    
    stats: {
      totalCreditsEarned: 0,
      totalDistanceTraveled: 0,
      portsVisited: 1,
      cardsAcquired: 0,
      crewLost: 0,
      contractsCompleted: 0,
      contractsFailed: 0,
      erasSurvived: 0,
    },
    
    rngSeed: Date.now(),
    rngState: Date.now(),
  };
}
