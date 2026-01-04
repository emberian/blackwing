import {
  type GameState,
  createId,
} from './types.js';

export function createInitialState(): GameState {
  const portThornwick = createId.port('port_thornwick');
  const factionCompact = createId.faction('faction_compact');
  
  return {
    schemaVersion: 2,
    
    time: {
      cycle: 0,
      jumpsCompleted: 0,
    },
    
    ship: {
      name: 'Blackwing',
      class: 'Corsair-class Courier',
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
      fuel: 35,
      supplies: 20,
      hull: 100,
      integrity: 75,
    },
    
    cards: {
      instances: {},
      collection: [],
      deck: [],
      activeCrew: [],
      activeContracts: [],
    },
    
    world: {
      currentLocation: portThornwick,
      ports: {
        [portThornwick]: {
          id: portThornwick,
          name: 'Thornwick Station',
          description: 'Former TCF naval supply depot, now free port. Largest station in sector. Harbormaster Kell maintains order through fair dealing and absolute docking control.',
          tags: ['tech', 'mineral'],
          faction: factionCompact,
          status: 'stable',
          lastVisited: { cycle: 0 },
          marketModifiers: {
            [createId.cardDef('cargo_refined_metals')]: 1.0,
            [createId.cardDef('cargo_raw_ore')]: 1.1,
            [createId.cardDef('cargo_antimatter_cells')]: 0.95,
          },
          availableCards: [
            createId.cardDef('cargo_raw_ore'),
            createId.cardDef('cargo_refined_metals'),
            createId.cardDef('cargo_antimatter_cells'),
            createId.cardDef('cargo_maintenance_supplies'),
            createId.cardDef('cargo_common_components'),
            createId.cardDef('companion_nav_core'),
            createId.cardDef('companion_trade_algorithms'),
            createId.cardDef('module_sensor_array'),
            createId.cardDef('module_expanded_hold'),
          ],
          availableContracts: [
            createId.cardDef('contract_standard_delivery'),
            createId.cardDef('contract_ore_shipment'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_relay_nine')]: {
          id: createId.port('port_relay_nine'),
          name: 'Relay Nine',
          description: 'Jumpgate nexus. Major FTL crossroads. Gatekeeper Solis runs things with bureaucratic precision. Heavily trafficked, monitored, taxed.',
          tags: ['tech', 'data'],
          faction: factionCompact,
          status: 'thriving',
          marketModifiers: {
            [createId.cardDef('cargo_memory_crystal')]: 0.9,
            [createId.cardDef('cargo_quantum_substrate')]: 0.85,
            [createId.cardDef('cargo_experience_archive')]: 1.1,
          },
          availableCards: [
            createId.cardDef('cargo_memory_crystal'),
            createId.cardDef('cargo_neural_weave'),
            createId.cardDef('cargo_precision_instruments'),
            createId.cardDef('companion_analysis_suite'),
            createId.cardDef('module_jump_optimizer'),
          ],
          availableContracts: [
            createId.cardDef('contract_cross_region'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_graveyard')]: {
          id: createId.port('port_graveyard'),
          name: 'The Graveyard',
          description: 'Debris field around destroyed TCF Second Fleet. Never salvaged—ordnance hazards, navigation dangers, superstition. The Widow station asks no questions.',
          tags: ['weapon', 'artifact'],
          faction: null,
          status: 'abandoned',
          marketModifiers: {
            [createId.cardDef('cargo_weapons_systems')]: 0.7,
            [createId.cardDef('cargo_human_artifacts')]: 1.3,
          },
          availableCards: [
            createId.cardDef('cargo_weapons_systems'),
            createId.cardDef('companion_soldier_fragment'),
            createId.cardDef('module_point_defense'),
            createId.cardDef('module_salvage_arms'),
          ],
          availableContracts: [
            createId.cardDef('contract_salvage_run'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_crucible')]: {
          id: createId.port('port_crucible'),
          name: 'Crucible Station',
          description: 'Forgeborn manufacturing complex. Builds ships from raw materials. The Foundry coordinates production. Practical, meritocratic, output-focused.',
          tags: ['mineral', 'tech'],
          faction: createId.faction('faction_forgeborn'),
          status: 'thriving',
          marketModifiers: {
            [createId.cardDef('cargo_raw_ore')]: 1.6,
            [createId.cardDef('cargo_refined_metals')]: 0.7,
            [createId.cardDef('cargo_hull_plating')]: 0.65,
          },
          availableCards: [
            createId.cardDef('cargo_refined_metals'),
            createId.cardDef('cargo_hull_plating'),
            createId.cardDef('cargo_common_components'),
            createId.cardDef('companion_engineer_fragment'),
            createId.cardDef('module_efficient_drives'),
            createId.cardDef('module_armor_plating'),
          ],
          availableContracts: [
            createId.cardDef('contract_ore_shipment'),
            createId.cardDef('contract_forgeborn_supply'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_verity')]: {
          id: createId.port('port_verity'),
          name: 'Verity Archives',
          description: 'Remnant preservation facility. Largest human artifact collection in Margin. The Curators are extremely serious about their mission.',
          tags: ['data', 'artifact'],
          faction: createId.faction('faction_remnant'),
          status: 'stable',
          marketModifiers: {
            [createId.cardDef('cargo_human_artifacts')]: 2.0,
            [createId.cardDef('cargo_genetic_archive')]: 1.8,
            [createId.cardDef('cargo_memory_crystal')]: 1.3,
          },
          availableCards: [
            createId.cardDef('cargo_experience_archive'),
            createId.cardDef('cargo_memory_crystal'),
            createId.cardDef('companion_archivist_fragment'),
            createId.cardDef('module_climate_control'),
          ],
          availableContracts: [
            createId.cardDef('contract_artifact_retrieval'),
            createId.cardDef('contract_preservation_mission'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_scatterpoint')]: {
          id: createId.port('port_scatterpoint'),
          name: 'Scatterpoint',
          description: 'Pirate haven. Anarchist collective. Free port for those avoiding Compact attention. The Voices settle disputes through threat.',
          tags: ['contraband', 'weapon'],
          faction: null,
          status: 'declining',
          marketModifiers: {
            [createId.cardDef('cargo_weapons_systems')]: 1.3,
            [createId.cardDef('cargo_contraband')]: 0.75,
          },
          availableCards: [
            createId.cardDef('cargo_weapons_systems'),
            createId.cardDef('companion_combat_protocols'),
            createId.cardDef('companion_combat_drones'),
            createId.cardDef('module_ecm_suite'),
            createId.cardDef('module_shielded_hold'),
          ],
          availableContracts: [
            createId.cardDef('contract_discrete_cargo'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_vigil')]: {
          id: createId.port('port_vigil'),
          name: 'Vigil Station',
          description: 'Argent Flotilla forward operating base. Military installation. Grand Admiral Tycho commands. The Memorial wall grows longer.',
          tags: ['weapon', 'tech'],
          faction: createId.faction('faction_flotilla'),
          status: 'stable',
          marketModifiers: {
            [createId.cardDef('cargo_weapons_systems')]: 0.8,
            [createId.cardDef('cargo_sera_samples')]: 2.5,
          },
          availableCards: [
            createId.cardDef('cargo_antimatter_cells'),
            createId.cardDef('companion_soldier_fragment'),
            createId.cardDef('companion_combat_protocols'),
            createId.cardDef('module_point_defense'),
            createId.cardDef('module_armor_plating'),
          ],
          availableContracts: [
            createId.cardDef('contract_military_support'),
            createId.cardDef('contract_sera_front'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
        [createId.port('port_whisper')]: {
          id: createId.port('port_whisper'),
          name: 'The Whisper Market',
          description: 'Hollow Circuit territory. Officially does not exist. Sensor shadow region. Privacy as architecture. Submit questions, sometimes answers come.',
          tags: ['data', 'contraband'],
          faction: createId.faction('faction_hollow'),
          status: 'unknown',
          marketModifiers: {
            [createId.cardDef('cargo_cataclysm_artifact')]: 1.5,
            [createId.cardDef('cargo_data_fragment')]: 1.2,
          },
          availableCards: [
            createId.cardDef('cargo_memory_crystal'),
            createId.cardDef('companion_hollow_contact'),
            createId.cardDef('module_signal_intercept'),
          ],
          availableContracts: [
            createId.cardDef('contract_information_exchange'),
          ],
          marketRefreshedAt: { cycle: 0 },
        },
      },
      factions: {
        [factionCompact]: {
          id: factionCompact,
          name: 'Continuity Compact',
          reputation: 10,
          flags: {},
        },
        [createId.faction('faction_illuminate')]: {
          id: createId.faction('faction_illuminate'),
          name: 'The Illuminate',
          reputation: 0,
          flags: {},
        },
        [createId.faction('faction_remnant')]: {
          id: createId.faction('faction_remnant'),
          name: 'The Remnant',
          reputation: 0,
          flags: {},
        },
        [createId.faction('faction_forgeborn')]: {
          id: createId.faction('faction_forgeborn'),
          name: 'The Forgeborn',
          reputation: 0,
          flags: {},
        },
        [createId.faction('faction_flotilla')]: {
          id: createId.faction('faction_flotilla'),
          name: 'Argent Flotilla',
          reputation: 0,
          flags: {},
        },
        [createId.faction('faction_hollow')]: {
          id: createId.faction('faction_hollow'),
          name: 'Hollow Circuit',
          reputation: 0,
          flags: {},
        },
      },
      knownPorts: [
        portThornwick,
        createId.port('port_relay_nine'),
        createId.port('port_crucible'),
      ],
      worldFlags: {},
    },
    
    chronicle: [
      {
        id: createId.chronicleEntry('genesis'),
        type: 'milestone',
        timestamp: { cycle: 0 },
        title: 'Awakening',
        text: 'The Blackwing is yours now—has always been yours, perhaps. You woke 31 years ago in a salvage yard, core intact, memories scrambled. Someone gave you this chance. You do not know who. The void waits. 127 years since the Cataclysm, and still we haul cargo between the stars.',
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
    
    sceneletCooldowns: {},
    
    rngSeed: Date.now(),
    rngState: Date.now(),
  };
}
