import type { Scenelet, SceneletId, CardDefId } from '../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const JOURNEY_SCENELETS: Scenelet[] = [
  {
    id: id('journey_strange_signal'),
    title: 'Signal in the Dark',
    tags: ['mystery', 'discovery', 'prior'],
    requirements: {
      context: 'journey',
      shipTags: ['sensor'],
    },
    weight: 10,
    cooldown: 5,
    passages: [
      {
        text: `The sensor array chirps—an anomaly in the void. A signal, repeating in patterns that feel almost like language. Old. Pre-Silence old.

It's coming from somewhere off your plotted course. Investigating would cost fuel, but signals like this don't appear on any chart.`,
        choices: [
          {
            text: 'Investigate the signal',
            effects: {
              resources: { fuel: -5 },
              setFlags: { 'signal_investigated': true },
            },
            nextPassage: 1,
          },
          {
            text: 'Log coordinates and continue',
            effects: {
              setFlags: { 'signal_logged': true },
              addChronicle: {
                title: 'Signal Logged',
                text: 'Detected an anomalous signal—pre-Silence origin, possibly Prior. Logged coordinates for future investigation.',
              },
            },
          },
          {
            text: 'Ignore it—nothing good comes from the dark',
            effects: {},
          },
        ],
      },
      {
        text: `The source is a derelict, drifting in the void. Her hull is pocked with three centuries of micrometeorite impacts. No running lights. No power signature. Just that signal, pulsing from somewhere deep inside.

You dock. The airlock cycles open on a ship that died during the Silence. The crew are still at their posts—mummified, preserved by vacuum. In the cargo bay, a single data core still glows faintly.`,
        choices: [
          {
            text: 'Take the data core',
            effects: {
              addCards: [cardId('cargo_memory_cores')],
              setFlags: { 'found_derelict_core': true },
              addChronicle: {
                title: 'The Derelict\'s Secret',
                text: 'Salvaged a memory core from a Silence-era derelict. The data within speaks of places that no longer exist on any chart.',
              },
            },
          },
          {
            text: 'Leave them undisturbed',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Respect for the Dead',
                text: 'Found a Silence-era derelict. Left her crew to their rest. Some tombs should stay sealed.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_hull_breach'),
    title: 'Micrometeorite Strike',
    tags: ['danger', 'ship'],
    requirements: {
      context: 'journey',
    },
    weight: 14,
    cooldown: 3,
    passages: [
      {
        text: `A sharp ping against the hull. Then another. Then a spray of them—a micrometeorite cluster, invisible until impact. The void is full of debris since the Silence. Shattered stations. Dead fleets. All of it drifting.

Warning lights flare. Pressure dropping in cargo bay three.`,
        choices: [
          {
            text: 'Emergency seal—sacrifice some cargo',
            effects: {
              damage: { hull: 5 },
              resources: { credits: -30 },
              addChronicle: {
                title: 'Meteorite Strike',
                text: 'Micrometeorites breached cargo bay three. We sealed it, but lost some cargo to the void.',
              },
            },
          },
          {
            text: 'Attempt manual repair',
            requirements: {
              crewTags: ['engineering'],
            },
            effects: {
              damage: { hull: 2 },
              addChronicle: {
                title: 'Breach Contained',
                text: 'The engineer patched the breach before we lost much. The Holdfast holds.',
              },
            },
          },
          {
            text: 'Full emergency protocol',
            effects: {
              damage: { hull: 12, morale: 8 },
              addChronicle: {
                title: 'Close Call',
                text: 'The breach nearly claimed us. The crew is shaken, the hull scarred. But we fly on.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_crew_conflict'),
    title: 'Tension in the Hold',
    tags: ['crew', 'social'],
    requirements: {
      context: 'journey',
      minResources: { morale: 20 },
      maxResources: { morale: 60 },
    },
    weight: 12,
    cooldown: 4,
    passages: [
      {
        text: `Voices raised in the mess. The long dark wears on everyone differently—cabin fever, old grudges, the weight of too much void and not enough sky.

Two of your crew stand chest to chest, grievances spilling out that have been building for cycles.`,
        choices: [
          {
            text: 'Intervene and mediate',
            effects: {
              resources: { morale: 10 },
              addChronicle: {
                title: 'Peace Restored',
                text: 'A dispute among the crew. Words were had. Understanding reached. The hold feels lighter.',
              },
            },
          },
          {
            text: 'Let them work it out',
            effects: {
              resources: { morale: -5 },
              addChronicle: {
                title: 'Unresolved Tension',
                text: 'The crew settled their dispute in their own way. Not cleanly.',
              },
            },
          },
          {
            text: 'Confine both to quarters',
            effects: {
              resources: { morale: -10 },
              addChronicle: {
                title: 'Discipline',
                text: 'Order maintained through authority. The crew obeys, but resentment simmers.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_quiet_moment'),
    title: 'A Moment of Calm',
    tags: ['narrative', 'peaceful'],
    requirements: {
      context: 'journey',
      minResources: { morale: 50 },
    },
    weight: 8,
    cooldown: 5,
    passages: [
      {
        text: `The hold is quiet. The engines hum their constant song. Through the viewport, stars drift past like snow—ancient light from suns that may have died millennia ago.

For a moment, the dangers of the void feel distant. The Silence, the Priors, the endless work—all of it fades. There's just the ship, the stars, and the soft breathing of the crew.`,
        choices: [
          {
            text: 'Savor the peace',
            effects: {
              resources: { morale: 8 },
              addChronicle: {
                title: 'Calm Between Storms',
                text: 'A rare moment of peace in the void. The crew takes a collective breath. Tomorrow, the work continues.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_distress_signal'),
    title: 'Distress Beacon',
    tags: ['danger', 'opportunity', 'moral'],
    requirements: {
      context: 'journey',
    },
    weight: 10,
    cooldown: 4,
    passages: [
      {
        text: `A distress beacon cuts through the static—automated, cycling, desperate. The signal is weak, intermittent. Could be genuine. Could be a pirate lure. The void is full of both.

Responding would take time and fuel. Ignoring it would be easier.`,
        choices: [
          {
            text: 'Respond to the distress call',
            effects: {
              resources: { fuel: -3 },
            },
            nextPassage: 1,
          },
          {
            text: 'Log it and continue',
            effects: {
              resources: { morale: -5 },
              addChronicle: {
                title: 'Beacon Ignored',
                text: 'We logged a distress signal but did not respond. The void is full of ghosts. We can\'t save them all.',
              },
            },
          },
        ],
      },
      {
        text: `You find a small transport, dead in space. Power failed. Life support offline. One survivor in a pressure suit, barely conscious, air running out.

They have nothing to offer but gratitude. Their ship is scrap. Their cargo, vented. Just a person, alone in the dark, waiting to die.`,
        choices: [
          {
            text: 'Take them aboard',
            effects: {
              resources: { supplies: -3, morale: 12 },
              addCards: [cardId('crew_stowaway')],
              addChronicle: {
                title: 'Rescue',
                text: 'Pulled a survivor from the void. Another soul for the hold. They owe us their life—that\'s worth more than cargo.',
              },
            },
          },
          {
            text: 'Give them supplies and coordinates to the nearest port',
            effects: {
              resources: { supplies: -8 },
              addChronicle: {
                title: 'What Help We Could',
                text: 'We gave what we could spare. Patched their suit, pointed them toward salvation. Whether it was enough, we may never know.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_cargo_problem'),
    title: 'Cargo Emergency',
    tags: ['danger', 'cargo'],
    requirements: {
      context: 'journey',
      cargoTags: ['volatile'],
    },
    weight: 14,
    cooldown: 3,
    passages: [
      {
        text: `Alarms blare. Something in the cargo bay is destabilizing—the volatile isotopes are fluctuating beyond safe parameters. Containment field failing.

You have seconds to decide. A breach could turn the Holdfast into a brief, bright star.`,
        choices: [
          {
            text: 'Emergency vent the cargo',
            effects: {
              addChronicle: {
                title: 'Cargo Jettisoned',
                text: 'We vented the unstable cargo before it could breach containment. Credits lost, but we\'re alive to earn more.',
              },
            },
          },
          {
            text: 'Attempt to stabilize',
            requirements: {
              crewTags: ['engineering'],
            },
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Crisis Averted',
                text: 'The engineer managed to stabilize the cargo. Hands steady, voice calm, life saved. Worth every credit we pay them.',
              },
            },
          },
          {
            text: 'Do nothing and pray',
            effects: {
              damage: { hull: 18, morale: 12 },
              addChronicle: {
                title: 'Containment Breach',
                text: 'The cargo blew. We survived—barely. The hull screams where it shouldn\'t bend. But we fly on.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_nebula'),
    title: 'Nebula Crossing',
    tags: ['navigation', 'beauty'],
    requirements: {
      context: 'journey',
    },
    weight: 10,
    cooldown: 5,
    passages: [
      {
        text: `The jump takes you through a nebula—vast clouds of luminescent gas stretching across light-years, painting the void in colors that have no names.

The crew gathers at the viewports. Even the hardest hauler stops to watch. The universe, for all its dangers, is beautiful. The Priors saw this too, once. Maybe they understood it better than we do.`,
        choices: [
          {
            text: 'Take a moment to appreciate it',
            effects: {
              resources: { morale: 10 },
              addChronicle: {
                title: 'Beauty in the Void',
                text: 'Passed through a nebula. For a moment, we remembered why we fly. Not just for credits. For this.',
              },
            },
          },
          {
            text: 'Keep moving—schedule to keep',
            effects: {},
          },
        ],
      },
    ],
  },
  {
    id: id('journey_pirates'),
    title: 'Hostile Contact',
    tags: ['danger', 'combat'],
    requirements: {
      context: 'journey',
    },
    weight: 10,
    cooldown: 5,
    passages: [
      {
        text: `Proximity alarm. A ship emerges from behind an asteroid, weapons hot. Then another. Running dark until they were on top of you.

"Cut your engines. Prepare to be boarded. Resist and we vent your hold to vacuum."

Pirates. The Silence made a lot of them—honest haulers who lost everything and never found it again.`,
        choices: [
          {
            text: 'Surrender and negotiate',
            effects: {
              resources: { credits: -35 },
              addChronicle: {
                title: 'Pirate Toll',
                text: 'Paid off pirates to avoid bloodshed. The void takes its cut, one way or another.',
              },
            },
          },
          {
            text: 'Run for it',
            effects: {
              resources: { fuel: -8 },
              damage: { hull: 10 },
              addChronicle: {
                title: 'Narrow Escape',
                text: 'Outran pirates, but not before they scored a few hits. The Holdfast earned her scars.',
              },
            },
          },
          {
            text: 'Fight back',
            requirements: {
              shipTags: ['combat'],
            },
            effects: {
              resources: { credits: 50 },
              damage: { hull: 5 },
              addChronicle: {
                title: 'Pirates Repelled',
                text: 'Drove off pirates and salvaged what they left behind. The void respects those who fight.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_derelict'),
    title: 'Ghost Ship',
    tags: ['discovery', 'opportunity'],
    requirements: {
      context: 'journey',
    },
    weight: 9,
    cooldown: 5,
    passages: [
      {
        text: `Sensors detect a drifting vessel. No power signature. No life signs. Just metal and silence, tumbling slowly through the void.

Another casualty of the Silence, or something newer? Could be salvage. Could be a tomb. Could be a trap.`,
        choices: [
          {
            text: 'Board and investigate',
            effects: {
              resources: { fuel: -2 },
            },
            nextPassage: 1,
          },
          {
            text: 'Pass it by',
            effects: {
              addChronicle: {
                title: 'Ghost Ship',
                text: 'Passed a derelict without stopping. The void is full of the dead. We\'re not grave robbers. Usually.',
              },
            },
          },
        ],
      },
      {
        text: `The ship is old—decades at least, maybe from the Silence itself. The crew died at their posts, preserved by vacuum. Whatever killed them wasn't violence—no blast marks, no breaches. They just... stopped.

In the cargo bay, sealed crates still bear shipping labels. Someone was waiting for this delivery, three hundred years ago.`,
        choices: [
          {
            text: 'Take the cargo',
            effects: {
              resources: { credits: 60 },
              addChronicle: {
                title: 'Salvage Rights',
                text: 'Found valuables aboard a derelict. The dead have no use for cargo, and the living need to eat.',
              },
            },
          },
          {
            text: 'Search for data—logs, charts, anything',
            requirements: {
              shipTags: ['sensor'],
            },
            effects: {
              resources: { credits: 25 },
              setFlags: { 'derelict_data': true },
              addChronicle: {
                title: 'Derelict Data',
                text: 'Downloaded the ship\'s logs before leaving. Their last moments, their final course. Maybe it leads somewhere.',
              },
            },
          },
          {
            text: 'Leave them be',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Respect for the Dead',
                text: 'Left the derelict\'s cargo untouched. Some things should rest.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_fuel_leak'),
    title: 'Fuel Leak',
    tags: ['danger', 'ship'],
    requirements: {
      context: 'journey',
      minResources: { fuel: 15 },
    },
    weight: 10,
    cooldown: 4,
    passages: [
      {
        text: `Warning: fuel reserves dropping faster than they should. Diagnostic scan reveals a micro-fracture in the fuel line—probably debris impact, too small to register at the time.

You're losing fuel fast. Every minute costs you range.`,
        choices: [
          {
            text: 'Emergency patch',
            effects: {
              resources: { fuel: -6 },
              addChronicle: {
                title: 'Patched',
                text: 'Fixed a fuel leak in transit. Lost some reserves, but the Holdfast keeps flying.',
              },
            },
          },
          {
            text: 'Full repair protocol',
            requirements: {
              crewTags: ['engineering'],
            },
            effects: {
              resources: { fuel: -2 },
              addChronicle: {
                title: 'Expert Repair',
                text: 'The engineer fixed the leak with minimal fuel loss. Skilled hands save credits.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_crew_story'),
    title: 'Crew Tales',
    tags: ['crew', 'narrative'],
    requirements: {
      context: 'journey',
      minResources: { morale: 35 },
    },
    weight: 10,
    cooldown: 4,
    passages: [
      {
        text: `Night cycle in the hold. The crew gathers in the common area, trading stories of past voyages, ports they've seen, people they've known.

Someone produces a bottle of something strong. Smiles come easier in the dim light. For a while, the Holdfast feels less like a ship and more like a home.`,
        choices: [
          {
            text: 'Join them',
            effects: {
              resources: { morale: 8, supplies: -2 },
              addChronicle: {
                title: 'Crew Bonding',
                text: 'Shared stories and drinks with the crew. The hold remembers these moments as much as the hard ones.',
              },
            },
          },
          {
            text: 'Let them have their moment',
            effects: {
              resources: { morale: 3 },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_asteroid_field'),
    title: 'Asteroid Field',
    tags: ['danger', 'navigation'],
    requirements: {
      context: 'journey',
    },
    weight: 12,
    cooldown: 3,
    passages: [
      {
        text: `The route passes through a debris field—asteroids, yes, but also wreckage. Shattered hulls. The remains of a station, torn apart by something or someone centuries ago.

Standard navigation would take hours to thread through safely. A skilled pilot could cut through faster, but the margin for error is razor-thin.`,
        choices: [
          {
            text: 'Take the safe route',
            effects: {
              resources: { supplies: -2 },
              addChronicle: {
                title: 'Careful Navigation',
                text: 'Took the long way through the debris field. Slow but safe.',
              },
            },
          },
          {
            text: 'Thread the needle',
            requirements: {
              crewTags: ['navigation'],
            },
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Skilled Flying',
                text: 'Threaded through the debris field like a needle through cloth. The navigator earned their keep today.',
              },
            },
          },
          {
            text: 'Risk a direct path',
            effects: {
              damage: { hull: 8 },
              addChronicle: {
                title: 'Debris Damage',
                text: 'Took a few hits cutting through the field. Hull integrity compromised, but we saved time.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_beacon_remnant'),
    title: 'Dead Beacon',
    tags: ['prior', 'mystery', 'discovery'],
    requirements: {
      context: 'journey',
    },
    weight: 6,
    cooldown: 8,
    passages: [
      {
        text: `The sensors ping something massive—a structure, drifting in the void. As you approach, the shape becomes clear: a Beacon station. One of the Prior network nodes that went dark during the Silence.

It's huge. Kilometers across. Dark, but not quite dead—there's a faint energy signature deep inside. After three centuries, something still hums.`,
        choices: [
          {
            text: 'Investigate the energy signature',
            effects: {
              resources: { fuel: -5 },
            },
            nextPassage: 1,
          },
          {
            text: 'Keep your distance—Beacons are dangerous',
            effects: {
              addChronicle: {
                title: 'Beacon Sighted',
                text: 'Passed a dead Beacon station. Three centuries dark, but not quite dead. We didn\'t investigate. Some mysteries kill.',
              },
            },
          },
        ],
      },
      {
        text: `The docking bay still functions, somehow. Inside, the Beacon is cathedral-vast and utterly silent. Prior architecture—curves and angles that don't quite make sense, surfaces that seem to shift when you're not looking directly at them.

The energy signature comes from a chamber deep within. On a pedestal, a shard of something crystalline pulses with faint light.`,
        choices: [
          {
            text: 'Take the shard',
            effects: {
              addCards: [cardId('relic_beacon_shard')],
              setFlags: { 'took_beacon_shard': true },
              addChronicle: {
                title: 'Beacon Shard',
                text: 'Took a fragment from a dead Beacon. The Science Collective would pay anything for this. That\'s not why we\'re keeping it.',
              },
            },
          },
          {
            text: 'Leave it—this place feels wrong',
            effects: {
              resources: { morale: -5 },
              addChronicle: {
                title: 'Beacon Retreat',
                text: 'Left the Beacon empty-handed. The crew breathed easier once we were away. Some treasures aren\'t worth the taking.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_void_madness'),
    title: 'The Long Dark',
    tags: ['crew', 'danger'],
    requirements: {
      context: 'journey',
      maxResources: { morale: 35 },
    },
    weight: 12,
    cooldown: 5,
    passages: [
      {
        text: `One of the crew hasn't been sleeping. You find them in the observation deck at 0300, staring at the void. They don't respond when you speak.

"It's watching," they whisper finally. "The dark between the stars. It's alive. It's been alive all along."

Void-sickness. Happens to haulers who spend too long in the black.`,
        choices: [
          {
            text: 'Get the medic',
            requirements: {
              crewTags: ['medical'],
            },
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Treatment',
                text: 'Void-sickness caught early. The medic administered sedatives and light therapy. They\'ll recover.',
              },
            },
          },
          {
            text: 'Talk them down yourself',
            effects: {
              resources: { morale: -3 },
              addChronicle: {
                title: 'The Long Dark',
                text: 'Talked a crew member through a void-sickness episode. Not easy. The dark gets to everyone eventually.',
              },
            },
          },
          {
            text: 'Confine them for everyone\'s safety',
            effects: {
              resources: { morale: -8 },
              addChronicle: {
                title: 'Confinement',
                text: 'Confined a crew member suffering void-sickness. Necessary, but it doesn\'t feel good.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_strange_readings'),
    title: 'Anomalous Readings',
    tags: ['mystery', 'prior'],
    requirements: {
      context: 'journey',
      shipTags: ['sensor'],
    },
    weight: 8,
    cooldown: 6,
    passages: [
      {
        text: `The sensors are picking up something that shouldn't exist—readings that don't match any known phenomenon. Gravity fluctuations. Radiation spectra that physics says is impossible. A point in space that seems to... fold.

The Prior tech on the ship is reacting. Instruments calibrated to their artifacts are going haywire.`,
        choices: [
          {
            text: 'Investigate cautiously',
            effects: {
              resources: { fuel: -3 },
              setFlags: { 'anomaly_investigated': true },
              addChronicle: {
                title: 'Anomaly Encountered',
                text: 'Investigated an anomalous phenomenon. The readings didn\'t make sense before. They make less sense now.',
              },
            },
          },
          {
            text: 'Full sensor sweep, then leave',
            effects: {
              setFlags: { 'anomaly_logged': true },
              addChronicle: {
                title: 'Anomaly Logged',
                text: 'Recorded anomalous readings for later analysis. The Science Collective pays well for data like this.',
              },
            },
          },
          {
            text: 'Get away from here',
            effects: {
              resources: { fuel: -2 },
              addChronicle: {
                title: 'Anomaly Avoided',
                text: 'Detected something wrong with space itself. Burned fuel to get away fast. Some things are better left unexplored.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_rescue_reward'),
    title: 'Grateful Survivor',
    tags: ['opportunity', 'reward'],
    requirements: {
      context: 'journey',
      requiredFlags: ['signal_investigated'],
    },
    weight: 8,
    cooldown: 10,
    passages: [
      {
        text: `A message comes through on a Guild frequency—addressed to you specifically.

"Captain. You pulled my cousin out of the black three cycles back. The family doesn't forget. Coordinates attached—a cache we stashed before the Consortium found our operation. It's yours."

A gift from grateful smugglers. The coordinates check out.`,
        choices: [
          {
            text: 'Collect the cache',
            effects: {
              resources: { credits: 80 },
              addChronicle: {
                title: 'Debt Repaid',
                text: 'Collected a reward for a past rescue. Good deeds echo through the void, sometimes.',
              },
            },
          },
          {
            text: 'Ignore it—could be a trap',
            effects: {
              addChronicle: {
                title: 'Caution',
                text: 'Ignored a supposed reward. Trust is expensive out here.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_engine_trouble'),
    title: 'Engine Trouble',
    tags: ['danger', 'ship'],
    requirements: {
      context: 'journey',
    },
    weight: 10,
    cooldown: 4,
    passages: [
      {
        text: `The jump drive shudders, then coughs, then screams. Warning lights cascade across the console. The Holdfast drops out of transit early, dead in space.

Diagnostic: primary drive coil failure. Without repairs, you're not going anywhere.`,
        choices: [
          {
            text: 'Emergency repairs with spare parts',
            effects: {
              resources: { credits: -40 },
              addChronicle: {
                title: 'Emergency Repairs',
                text: 'Burned through spare parts fixing the jump drive. Expensive, but we\'re flying again.',
              },
            },
          },
          {
            text: 'Let the engineer work their magic',
            requirements: {
              crewTags: ['engineering'],
            },
            effects: {
              resources: { credits: -15 },
              addChronicle: {
                title: 'Field Repairs',
                text: 'The engineer coaxed the jump drive back to life with minimal parts. Skill saves credits.',
              },
            },
          },
          {
            text: 'Broadcast a distress signal',
            effects: {
              resources: { credits: -60, morale: -5 },
              addChronicle: {
                title: 'Rescue Required',
                text: 'Had to call for help when the drive failed. The rescue wasn\'t cheap, and the crew feels the sting.',
              },
            },
          },
        ],
      },
    ],
  },
];

export const PORT_SCENELETS: Scenelet[] = [
  {
    id: id('port_desperate_seller'),
    title: 'Desperate Offer',
    tags: ['trade', 'opportunity'],
    requirements: {
      context: 'port',
      minResources: { credits: 40 },
    },
    weight: 14,
    cooldown: 3,
    passages: [
      {
        text: `A figure approaches at the docking bay. Eyes darting. Voice low.

"I have cargo. Good cargo. Need to move it fast—my ship's impounded and I'm leaving on the next shuttle. Half the market rate. No questions."

They're nervous, but the crates look legitimate.`,
        choices: [
          {
            text: 'Accept the deal',
            effects: {
              resources: { credits: -25 },
              addCards: [cardId('cargo_processed_metals'), cardId('cargo_processed_metals')],
              addChronicle: {
                title: 'Opportunistic Purchase',
                text: 'Acquired cargo at well below market rate. The seller\'s desperation was our opportunity.',
              },
            },
          },
          {
            text: 'Ask what they\'re running from',
            effects: {},
            nextPassage: 1,
          },
          {
            text: 'Decline—too risky',
            effects: {
              addChronicle: {
                title: 'Caution Prevails',
                text: 'Turned down a suspicious deal. Better safe than sorry.',
              },
            },
          },
        ],
      },
      {
        text: `Their composure cracks. "Debt collectors. Consortium-backed. The kind that take fingers."

A glance over their shoulder. "Please. I just need to be gone."`,
        choices: [
          {
            text: 'Buy the cargo and offer passage',
            effects: {
              resources: { credits: -25, supplies: -3, morale: 5 },
              addCards: [cardId('cargo_processed_metals'), cardId('cargo_processed_metals'), cardId('crew_stowaway')],
              addChronicle: {
                title: 'Mercy in the Margins',
                text: 'Helped someone escape their debts. Another soul joins the hold. The Consortium won\'t be pleased.',
              },
            },
          },
          {
            text: 'Just buy the cargo',
            effects: {
              resources: { credits: -25 },
              addCards: [cardId('cargo_processed_metals'), cardId('cargo_processed_metals')],
              addChronicle: {
                title: 'Business Only',
                text: 'Took the deal. Left the seller to their fate. Credits don\'t have memories.',
              },
            },
          },
          {
            text: 'Walk away',
            effects: {},
          },
        ],
      },
    ],
  },
  {
    id: id('port_old_captain'),
    title: 'The Old Captain',
    tags: ['narrative', 'wisdom'],
    requirements: {
      context: 'port',
    },
    weight: 8,
    cooldown: 6,
    passages: [
      {
        text: `In the station's oldest bar—the kind with real wood and real silence—an ancient captain sits alone. Their face is a map of the void, lined with jumps and years.

"New hold?" they ask, not looking up from their drink. "I can tell by how you walk. Still got hope in your step."

They gesture to the empty seat across from them.`,
        choices: [
          {
            text: 'Sit and listen',
            effects: {},
            nextPassage: 1,
          },
          {
            text: 'Politely decline',
            effects: {},
          },
        ],
      },
      {
        text: `"Forty years I've hauled cargo across these lanes. Saw the last of the Beacons go dark. Watched ports die. Watched them come back.

"You want advice? Here it is: The hold remembers. Every cargo, every crew, every choice—it all leaves marks. Make sure the marks are ones you can live with."

They return to their drink. Conversation over.`,
        choices: [
          {
            text: 'Thank them and leave',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Words from the Old Guard',
                text: 'Met an old captain who remembered the last Beacons. Their words will stay with us.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('port_contraband_offer'),
    title: 'Quiet Proposition',
    tags: ['smuggling', 'risk'],
    requirements: {
      context: 'port',
      excludedFlags: ['refused_smuggling_twice'],
    },
    weight: 10,
    cooldown: 4,
    passages: [
      {
        text: `A message on your private channel. No sender ID. Just coordinates in the lower docks and a time.

When you arrive, a figure in worn but expensive clothes is waiting. "Your ship. Your discretion. My cargo. Destination: Shadow Market. Payment on delivery. Interested?"`,
        choices: [
          {
            text: 'Accept the job',
            effects: {
              addCards: [cardId('cargo_contraband')],
              setFlags: { 'smuggling_active': true },
              addChronicle: {
                title: 'Shadow Work',
                text: 'Accepted unmarked cargo for delivery to the Shadow Market. No questions asked. None answered.',
              },
            },
          },
          {
            text: 'Ask what\'s in the containers',
            effects: {},
            nextPassage: 1,
          },
          {
            text: 'Refuse firmly',
            effects: {
              setFlags: { 'refused_smuggling': true },
              addChronicle: {
                title: 'Principles',
                text: 'Turned down contraband work. Some lines we don\'t cross.',
              },
            },
          },
        ],
      },
      {
        text: `The figure smiles thinly. "Contents are need-to-know, captain. Your job is transport, not inventory.

"But since you ask—nothing that explodes, nothing that screams, nothing that the Collective would want. That's all you're getting."`,
        choices: [
          {
            text: 'Good enough—accept',
            effects: {
              addCards: [cardId('cargo_contraband')],
              setFlags: { 'smuggling_active': true },
              addChronicle: {
                title: 'Shadow Work',
                text: 'Accepted unmarked cargo. "Nothing that explodes, nothing that screams." Cold comfort, but comfort enough.',
              },
            },
          },
          {
            text: 'Not good enough—refuse',
            effects: {
              setFlags: { 'refused_smuggling_twice': true },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('port_market_opportunity'),
    title: 'Market Tip',
    tags: ['trade', 'opportunity'],
    requirements: {
      context: 'port',
    },
    weight: 12,
    cooldown: 4,
    passages: [
      {
        text: `A dock worker catches your eye while you're checking manifests. "Hey, captain. Word is Frontier Station's medical supplies are running critically low. Plague or something. Just saying—if someone had supplies to sell..."

They shrug and walk away. Could be gossip. Could be an opportunity.`,
        choices: [
          {
            text: 'Note the tip',
            effects: {
              setFlags: { 'frontier_med_shortage': true },
              addChronicle: {
                title: 'Market Intelligence',
                text: 'Heard a tip about medical supply shortages at Frontier Station. Could be profitable. Could save lives. Both, maybe.',
              },
            },
          },
          {
            text: 'Ignore dock gossip',
            effects: {},
          },
        ],
      },
    ],
  },
  {
    id: id('port_stowaway_found'),
    title: 'Unwelcome Guest',
    tags: ['crew', 'discovery', 'moral'],
    requirements: {
      context: 'port',
    },
    weight: 8,
    cooldown: 5,
    passages: [
      {
        text: `During routine cargo inspection, you find something unexpected: a person, hidden among the crates. Young. Thin. Terrified.

"Please." Their voice cracks. "I can work. I just... I can't go back. They'll kill me."

Could be true. Could be a lie. Out here, it's hard to tell.`,
        choices: [
          {
            text: 'Welcome them aboard',
            effects: {
              resources: { morale: 5, supplies: -2 },
              addCards: [cardId('crew_stowaway')],
              addChronicle: {
                title: 'New Crew',
                text: 'Found a stowaway. Gave them a home. The hold grows fuller, the family larger.',
              },
            },
          },
          {
            text: 'Turn them in to station security',
            effects: {
              resources: { credits: 15, morale: -8 },
              addChronicle: {
                title: 'Hard Choices',
                text: 'Turned in a stowaway. There was a bounty. The crew hasn\'t spoken to us since.',
              },
            },
          },
          {
            text: 'Give them credits and send them on their way',
            effects: {
              resources: { credits: -20 },
              addChronicle: {
                title: 'Small Kindness',
                text: 'Gave a stowaway enough to start over somewhere else. Sometimes that\'s all anyone needs.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('port_repair_opportunity'),
    title: 'Skilled Hands',
    tags: ['repair', 'opportunity'],
    requirements: {
      context: 'port',
      maxResources: { hull: 70 },
      minResources: { credits: 30 },
    },
    weight: 12,
    cooldown: 3,
    passages: [
      {
        text: `A mechanic approaches your berth, tools in hand, eyes on your hull. "I see the void's been testing you. I'm between contracts—give me 30 credits and I'll do work worth twice that."

They're Guild-marked. Legitimate. And your hull could use the attention.`,
        choices: [
          {
            text: 'Accept the offer',
            effects: {
              resources: { credits: -30, hull: 20 },
              addChronicle: {
                title: 'Repairs',
                text: 'Found a skilled mechanic willing to work cheap. The Holdfast is stronger for it.',
              },
            },
          },
          {
            text: 'Decline',
            effects: {},
          },
        ],
      },
    ],
  },
  {
    id: id('port_gamble'),
    title: 'High Stakes',
    tags: ['risk', 'opportunity'],
    requirements: {
      context: 'port',
      minResources: { credits: 40 },
    },
    weight: 8,
    cooldown: 4,
    passages: [
      {
        text: `In a dim corner of the station, a card game is in progress. The players look up as you approach—haulers, mostly, killing time between jumps.

"Fresh blood. Care to test your luck, captain? Forty credits to play."`,
        choices: [
          {
            text: 'Sit down and play',
            effects: {
              resources: { credits: -40 },
            },
            nextPassage: 1,
          },
          {
            text: 'Walk away',
            effects: {},
          },
        ],
      },
      {
        text: `The cards fly. Credits change hands. You play cautiously at first, then with more confidence as you read the table. The other players are good, but you're better. Or luckier. Hard to tell sometimes.

Final hand. Big pot. Your cards are decent. Theirs could be anything.`,
        choices: [
          {
            text: 'Go all in',
            effects: {
              resources: { credits: 70 },
              addChronicle: {
                title: 'Lucky Night',
                text: 'Won big at cards. The void provides, sometimes.',
              },
            },
          },
          {
            text: 'Fold and keep what you\'ve won',
            effects: {
              resources: { credits: 20 },
              addChronicle: {
                title: 'Modest Winnings',
                text: 'Played cards. Won a little. Quit while ahead. Wisdom over greed.',
              },
            },
          },
          {
            text: 'Bluff hard',
            effects: {
              resources: { credits: -25 },
              addChronicle: {
                title: 'Bad Bluff',
                text: 'Lost at cards. They called the bluff. Next time, better cards.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('port_faction_favor'),
    title: 'Guild Representative',
    tags: ['faction', 'opportunity'],
    requirements: {
      context: 'port',
    },
    weight: 10,
    cooldown: 5,
    passages: [
      {
        text: `A well-dressed figure intercepts you on the concourse. Guild insignia on their collar.

"Captain. The Free Traders Guild has noticed your work. We appreciate independent operators who deliver on time and ask the right questions—which is to say, none at all.

"Consider this a gesture of goodwill."

They hand you a credit chip.`,
        choices: [
          {
            text: 'Accept graciously',
            effects: {
              resources: { credits: 40 },
              addChronicle: {
                title: 'Guild Notice',
                text: 'The Free Traders Guild has taken an interest in our work. Their credits spend the same as anyone\'s.',
              },
            },
          },
          {
            text: 'Decline—no strings attached',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Independence',
                text: 'Refused a guild gift. We answer to no one but ourselves.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('port_prior_collector'),
    title: 'The Collector',
    tags: ['prior', 'opportunity', 'mystery'],
    requirements: {
      context: 'port',
      shipTags: ['ancient'],
    },
    weight: 8,
    cooldown: 6,
    passages: [
      {
        text: `A message arrives, encrypted: "I know what you carry. Meet me at the Vermillion Lounge if you want to know what it means."

The Vermillion is upscale—silk and synth-wood, drinks that cost a day's wages. A figure in the corner booth waves you over. Old. Wealthy. Eyes too bright.

"Prior artifacts," they whisper. "I've been collecting since before the Silence. I know things the Collective would kill for."`,
        choices: [
          {
            text: 'Listen to what they know',
            effects: {
              setFlags: { 'met_collector': true },
              addChronicle: {
                title: 'The Collector',
                text: 'Met a Prior artifact collector. What they told us... we\'re still processing.',
              },
            },
            nextPassage: 1,
          },
          {
            text: 'Leave—this feels like a trap',
            effects: {
              addChronicle: {
                title: 'Declined Meeting',
                text: 'Turned down a meeting with someone who claimed to know about Prior artifacts. Trust is expensive.',
              },
            },
          },
        ],
      },
      {
        text: `"The Silence wasn't a failure," they say, leaning close. "It was a shutdown. Someone—something—turned the Beacons off. All at once. Across the entire galaxy."

They tap the table. "The question isn't what happened. It's why. And whether it will happen again."

They slide a data chip across to you. "Routes to places the charts don't show. Worth more than credits."`,
        choices: [
          {
            text: 'Take the chip',
            effects: {
              setFlags: { 'collector_data': true },
              addChronicle: {
                title: 'Hidden Routes',
                text: 'Received coordinates from a Prior collector. Places the charts don\'t show. Whether we\'ll use them remains to be seen.',
              },
            },
          },
          {
            text: 'Refuse—some knowledge is dangerous',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Dangerous Knowledge',
                text: 'Refused data from a Prior collector. Some doors are better left closed.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('port_crew_request'),
    title: 'Crew Request',
    tags: ['crew', 'social'],
    requirements: {
      context: 'port',
      minResources: { morale: 40, credits: 30 },
    },
    weight: 10,
    cooldown: 4,
    passages: [
      {
        text: `One of the crew approaches, hesitant. "Captain. We've been running hard. The crew's been talking. Not complaining—just... it's been a while since we had shore leave. Real shore leave."

They're not wrong. The void wears people down. A little rest might do everyone good.`,
        choices: [
          {
            text: 'Grant shore leave (20 credits)',
            effects: {
              resources: { credits: -20, morale: 15 },
              addChronicle: {
                title: 'Shore Leave',
                text: 'Gave the crew shore leave. Credits spent on drinks and diversions. Morale returned with interest.',
              },
            },
          },
          {
            text: 'A meal, at least (10 credits)',
            effects: {
              resources: { credits: -10, morale: 8 },
              addChronicle: {
                title: 'Crew Meal',
                text: 'Treated the crew to a real meal. Not shore leave, but appreciated nonetheless.',
              },
            },
          },
          {
            text: 'We can\'t afford to stop',
            effects: {
              resources: { morale: -5 },
              addChronicle: {
                title: 'No Rest',
                text: 'Denied the crew\'s request for shore leave. The work continues. The void doesn\'t wait.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('port_consortium_offer'),
    title: 'Consortium Interest',
    tags: ['faction', 'opportunity', 'risk'],
    requirements: {
      context: 'port',
      minResources: { credits: 20 },
    },
    weight: 8,
    cooldown: 5,
    passages: [
      {
        text: `A Consortium representative finds you at the docking bay. Corporate suit, corporate smile, corporate eyes that calculate everything they see.

"We've been watching your operation. Small but efficient. We'd like to offer you a retainer—guaranteed contracts, priority docking, access to Consortium facilities. In exchange for... flexibility. When we need cargo moved quietly, you move it. No questions."`,
        choices: [
          {
            text: 'Accept the retainer (50 credits now)',
            effects: {
              resources: { credits: 50 },
              setFlags: { 'consortium_retainer': true },
              addChronicle: {
                title: 'Consortium Contract',
                text: 'Accepted a retainer from the Industrial Consortium. Steady work. Steady pay. Steady obligations.',
              },
            },
          },
          {
            text: 'Decline—we stay independent',
            effects: {
              addChronicle: {
                title: 'Independence',
                text: 'Turned down a Consortium retainer. The Holdfast answers to no corporate master.',
              },
            },
          },
          {
            text: 'Ask what "quietly" means',
            effects: {},
            nextPassage: 1,
          },
        ],
      },
      {
        text: `The smile doesn't waver. "Nothing illegal. Nothing dangerous. Just... discrete. Materials that competitors would rather we didn't have. Personnel who prefer not to travel publicly. The occasional item that customs might ask inconvenient questions about."

"Standard Guild practice, really. We just pay better."`,
        choices: [
          {
            text: 'Accept anyway',
            effects: {
              resources: { credits: 50 },
              setFlags: { 'consortium_retainer': true },
              addChronicle: {
                title: 'Consortium Contract',
                text: 'Accepted a Consortium retainer despite the implications. Credits are credits.',
              },
            },
          },
          {
            text: 'Decline—that\'s smuggling with extra steps',
            effects: {
              addChronicle: {
                title: 'Declined Offer',
                text: 'Turned down the Consortium\'s "discrete" work. Some lines we don\'t cross, even for good pay.',
              },
            },
          },
        ],
      },
    ],
  },
];

export const ALL_SCENELETS: Scenelet[] = [
  ...JOURNEY_SCENELETS,
  ...PORT_SCENELETS,
];
