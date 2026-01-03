import type { Scenelet, SceneletId } from '../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const JOURNEY_SCENELETS: Scenelet[] = [
  {
    id: id('journey_strange_signal'),
    title: 'Strange Signal',
    tags: ['mystery', 'discovery'],
    requirements: {
      context: 'journey',
      shipTags: ['sensor'],
    },
    weight: 10,
    cooldown: 5,
    passages: [
      {
        text: `The sensor array chirps—an anomaly in the void. A signal, old beyond measure, repeating in patterns that feel almost like language.

It comes from somewhere off your plotted course. Investigating would cost fuel.`,
        choices: [
          {
            text: 'Investigate the signal',
            effects: {
              resources: { fuel: -5 },
              setFlags: { 'signal_investigated': true },
              addChronicle: {
                title: 'Signal in the Dark',
                text: 'We diverted to investigate an ancient signal. What we found there... requires time to understand.',
              },
            },
            nextPassage: 1,
          },
          {
            text: 'Log coordinates and continue',
            effects: {
              setFlags: { 'signal_logged': true },
              addChronicle: {
                title: 'Signal Logged',
                text: 'An anomalous signal was detected and logged. Perhaps another time.',
              },
            },
          },
          {
            text: 'Ignore it',
            effects: {},
          },
        ],
      },
      {
        text: `The source is a derelict—older than your records can identify. Its hull is pocked with micrometeorite impacts.

Inside, you find only silence and a single data core, still faintly powered.`,
        choices: [
          {
            text: 'Take the data core',
            effects: {
              addCards: ['cargo_memory_cores' as any],
              setFlags: { 'found_derelict_core': true },
              addChronicle: {
                title: 'The Derelict\'s Secret',
                text: 'From the ancient ship, a memory core. The data within speaks of places that no longer exist on any chart.',
              },
            },
          },
          {
            text: 'Leave it undisturbed',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Respect for the Dead',
                text: 'We left the derelict as we found it. Some things should remain undisturbed.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_hull_breach'),
    title: 'Micrometeorite Impact',
    tags: ['danger', 'ship'],
    requirements: {
      context: 'journey',
    },
    weight: 15,
    cooldown: 3,
    passages: [
      {
        text: `A sharp ping against the hull. Then another. Then a spray of them—a micrometeorite cluster, invisible until impact.

Warning lights flare. Pressure dropping in cargo bay three.`,
        choices: [
          {
            text: 'Emergency seal—sacrifice some cargo',
            effects: {
              damage: { hull: 5 },
              resources: { credits: -50 },
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
                text: 'The engineer patched the breach before we lost much. Could have been worse.',
              },
            },
          },
          {
            text: 'Full emergency protocol',
            effects: {
              damage: { hull: 10, morale: 10 },
              addChronicle: {
                title: 'Close Call',
                text: 'The breach nearly claimed us. The crew is shaken, the hull scarred.',
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
        text: `Voices raised in the mess. The journey wears on everyone differently.

Two of your crew stand chest to chest, grievances spilling out that have been building for a while.`,
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
                text: 'Order maintained through confinement. The crew obeys, but resentment simmers.',
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
        text: `The hold is quiet. The engines hum their constant song. Through the viewport, stars drift past like snow.

For a moment, everything feels right.`,
        choices: [
          {
            text: 'Savor the peace',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Calm Between Storms',
                text: 'A rare moment of peace in the void. The crew takes a collective breath.',
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
    tags: ['danger', 'opportunity'],
    requirements: {
      context: 'journey',
    },
    weight: 10,
    cooldown: 4,
    passages: [
      {
        text: `A distress beacon cuts through the static. The signal is weak, intermittent. Could be genuine. Could be a trap.

Responding would take time and fuel.`,
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
                text: 'We logged a distress signal but did not respond. The void is full of ghosts.',
              },
            },
          },
        ],
      },
      {
        text: `You find a small transport, dead in space. Power failed. Life support offline. One survivor in a pressure suit, barely conscious.

They have nothing to offer but gratitude.`,
        choices: [
          {
            text: 'Take them aboard',
            effects: {
              resources: { supplies: -5, morale: 10 },
              addCards: ['crew_stowaway' as any],
              addChronicle: {
                title: 'Rescue',
                text: 'Pulled a survivor from the void. Another soul for the hold.',
              },
            },
          },
          {
            text: 'Give them supplies and coordinates to the nearest port',
            effects: {
              resources: { supplies: -10 },
              addChronicle: {
                title: 'What Help We Could',
                text: 'We gave what we could spare. Whether it was enough, we may never know.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_cargo_problem'),
    title: 'Cargo Trouble',
    tags: ['danger', 'cargo'],
    requirements: {
      context: 'journey',
      cargoTags: ['volatile'],
    },
    weight: 12,
    cooldown: 3,
    passages: [
      {
        text: `Alarms blare. Something in the cargo bay is destabilizing. The volatile isotopes are fluctuating beyond safe parameters.

You have seconds to decide.`,
        choices: [
          {
            text: 'Emergency vent the cargo',
            effects: {
              addChronicle: {
                title: 'Cargo Jettisoned',
                text: 'We vented the unstable cargo before it could breach containment. A loss, but we\'re alive.',
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
                text: 'The engineer managed to stabilize the cargo. Close call.',
              },
            },
          },
          {
            text: 'Do nothing and hope',
            effects: {
              damage: { hull: 15, morale: 10 },
              addChronicle: {
                title: 'Containment Breach',
                text: 'The cargo blew. We survived, but the hull took heavy damage.',
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
    cooldown: 4,
    passages: [
      {
        text: `The jump takes you through a nebula—vast clouds of luminescent gas stretching across light-years.

The crew gathers at the viewports. For a moment, the dangers of the void feel distant.`,
        choices: [
          {
            text: 'Take a moment to appreciate it',
            effects: {
              resources: { morale: 10 },
              addChronicle: {
                title: 'Beauty in the Void',
                text: 'Passed through a nebula. Sometimes the universe reminds us why we travel.',
              },
            },
          },
          {
            text: 'Keep moving—we have a schedule',
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
    weight: 8,
    cooldown: 5,
    passages: [
      {
        text: `Proximity alarm. A ship emerges from behind an asteroid, weapons hot.

"Cut your engines. Prepare to be boarded. Resist and we vent your hold to vacuum."

Pirates. Two of them, closing fast.`,
        choices: [
          {
            text: 'Surrender and negotiate',
            effects: {
              resources: { credits: -40 },
              addChronicle: {
                title: 'Pirate Toll',
                text: 'Paid off pirates to avoid bloodshed. The void takes its cut.',
              },
            },
          },
          {
            text: 'Run for it',
            effects: {
              resources: { fuel: -10 },
              damage: { hull: 8 },
              addChronicle: {
                title: 'Narrow Escape',
                text: 'Outran pirates, but not before they scored a few hits.',
              },
            },
          },
          {
            text: 'Fight back',
            requirements: {
              shipTags: ['combat'],
            },
            effects: {
              resources: { credits: 60 },
              damage: { hull: 5 },
              addChronicle: {
                title: 'Pirate Hunters',
                text: 'Drove off pirates and salvaged what they left behind.',
              },
            },
          },
        ],
      },
    ],
  },
  {
    id: id('journey_derelict'),
    title: 'Abandoned Ship',
    tags: ['discovery', 'opportunity'],
    requirements: {
      context: 'journey',
    },
    weight: 8,
    cooldown: 5,
    passages: [
      {
        text: `Sensors detect a drifting vessel. No power signature. No life signs. Just metal and silence.

Could be salvage. Could be a tomb. Could be a trap.`,
        choices: [
          {
            text: 'Board and investigate',
            effects: {
              resources: { fuel: -3 },
            },
            nextPassage: 1,
          },
          {
            text: 'Pass it by',
            effects: {
              addChronicle: {
                title: 'Ghost Ship',
                text: 'Passed a derelict without stopping. Some mysteries are best left unsolved.',
              },
            },
          },
        ],
      },
      {
        text: `The ship is old—decades, maybe centuries. The crew died at their posts, mummified by vacuum.

In the cargo bay, a few crates remain sealed. Their contents might be valuable.`,
        choices: [
          {
            text: 'Take the cargo',
            effects: {
              resources: { credits: 75 },
              addChronicle: {
                title: 'Salvage Rights',
                text: 'Found valuables aboard a derelict. The dead have no use for cargo.',
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
        text: `Warning: fuel reserves dropping unexpectedly. Diagnostic scan reveals a micro-fracture in the fuel line.

You're losing fuel fast.`,
        choices: [
          {
            text: 'Emergency patch',
            effects: {
              resources: { fuel: -8 },
              addChronicle: {
                title: 'Patched',
                text: 'Fixed a fuel leak in transit. Lost some fuel, but could have been worse.',
              },
            },
          },
          {
            text: 'Full repair protocol',
            requirements: {
              crewTags: ['engineering'],
            },
            effects: {
              resources: { fuel: -3 },
              addChronicle: {
                title: 'Expert Repair',
                text: 'The engineer fixed the leak with minimal fuel loss.',
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
      minResources: { morale: 30 },
    },
    weight: 10,
    cooldown: 4,
    passages: [
      {
        text: `Night cycle in the hold. The crew gathers in the common area, trading stories of past voyages.

Someone produces a bottle of something strong. The conversation flows.`,
        choices: [
          {
            text: 'Join them',
            effects: {
              resources: { morale: 8, supplies: -2 },
              addChronicle: {
                title: 'Crew Bonding',
                text: 'Shared stories and drinks with the crew. The hold feels more like home.',
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
        text: `The route passes through a dense asteroid field. Standard navigation would take hours. A skilled pilot could cut through faster.`,
        choices: [
          {
            text: 'Take the safe route',
            effects: {
              resources: { supplies: -3 },
              addChronicle: {
                title: 'Careful Navigation',
                text: 'Took the long way through the asteroid field. Slow but safe.',
              },
            },
          },
          {
            text: 'Cut through the field',
            requirements: {
              crewTags: ['navigation'],
            },
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Skilled Flying',
                text: 'Threaded through the asteroid field like a needle through cloth.',
              },
            },
          },
          {
            text: 'Risk a direct path',
            effects: {
              damage: { hull: 10 },
              addChronicle: {
                title: 'Asteroid Damage',
                text: 'Took a few hits cutting through the field. Hull integrity compromised.',
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
      minResources: { credits: 50 },
    },
    weight: 15,
    cooldown: 3,
    passages: [
      {
        text: `A figure approaches at the docking bay. Eyes darting. Voice low.

"I have cargo. Good cargo. Need to move it fast—leaving on the next shuttle. Half the market rate. No questions."

They're nervous, but the crates look legitimate.`,
        choices: [
          {
            text: 'Accept the deal',
            effects: {
              resources: { credits: -30 },
              addCards: ['cargo_processed_metals' as any, 'cargo_processed_metals' as any],
              addChronicle: {
                title: 'Opportunistic Purchase',
                text: 'Acquired cargo at well below market rate. The seller\'s urgency was their loss.',
              },
            },
          },
          {
            text: 'Ask what they\'re running from',
            effects: {
              setFlags: { 'questioned_seller': true },
            },
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
        text: `Their composure cracks. "Debt collectors. The kind that take fingers."

A glance over their shoulder. "Please. I just need to be gone."`,
        choices: [
          {
            text: 'Buy the cargo and offer passage',
            effects: {
              resources: { credits: -30, supplies: -5, morale: 5 },
              addCards: ['cargo_processed_metals' as any, 'cargo_processed_metals' as any, 'crew_stowaway' as any],
              addChronicle: {
                title: 'Mercy in the Margins',
                text: 'Helped someone escape their debts. Another soul joins the hold.',
              },
            },
          },
          {
            text: 'Just buy the cargo',
            effects: {
              resources: { credits: -30 },
              addCards: ['cargo_processed_metals' as any, 'cargo_processed_metals' as any],
              addChronicle: {
                title: 'Business Only',
                text: 'Took the deal. Left the seller to their fate.',
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
        text: `In the station's oldest bar—the kind with real wood and real silence—an ancient captain sits alone.

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
        text: `"Thirty years I've hauled cargo across these lanes. Seen ports rise and fall. Seen crews come and go.

"You want advice? Here it is: The hold remembers. Every cargo, every crew, every choice—it all leaves marks. Make sure the marks are ones you can live with."

They return to their drink. Conversation over.`,
        choices: [
          {
            text: 'Thank them and leave',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Words from the Old Guard',
                text: 'Met an old captain. Their words will stay with us.',
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
      excludedFlags: ['refused_smuggling'],
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
              addCards: ['cargo_contraband' as any],
              setFlags: { 'smuggling_active': true },
              addChronicle: {
                title: 'Shadow Work',
                text: 'Accepted unmarked cargo for delivery to the Shadow Market. Questions weren\'t asked.',
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

"But since you ask—nothing that explodes, nothing that screams. That's all you're getting."`,
        choices: [
          {
            text: 'Good enough—accept',
            effects: {
              addCards: ['cargo_contraband' as any],
              setFlags: { 'smuggling_active': true },
              addChronicle: {
                title: 'Shadow Work',
                text: 'Accepted unmarked cargo. "Nothing that explodes, nothing that screams." Cold comfort.',
              },
            },
          },
          {
            text: 'Not good enough—refuse',
            effects: {
              setFlags: { 'refused_smuggling': true },
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
        text: `A dock worker catches your eye. "Hey, captain. Word is the Frontier Station's medical supplies are running low. Real low. Just saying—if someone had supplies to sell..."

They shrug and walk away.`,
        choices: [
          {
            text: 'Note the tip',
            effects: {
              setFlags: { 'frontier_med_shortage': true },
              addChronicle: {
                title: 'Market Intelligence',
                text: 'Heard a tip about medical supply shortages at Frontier Station. Could be profitable.',
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
    tags: ['crew', 'discovery'],
    requirements: {
      context: 'port',
    },
    weight: 8,
    cooldown: 5,
    passages: [
      {
        text: `During routine cargo inspection, your engineer finds something unexpected: a person, hidden among the crates.

They're young, thin, terrified. "Please. I can work. I just... I can't go back."`,
        choices: [
          {
            text: 'Welcome them aboard',
            effects: {
              resources: { morale: 5 },
              addCards: ['crew_stowaway' as any],
              addChronicle: {
                title: 'New Crew',
                text: 'Found a stowaway. Gave them a home. The hold grows fuller.',
              },
            },
          },
          {
            text: 'Turn them in to station security',
            effects: {
              resources: { credits: 20, morale: -10 },
              addChronicle: {
                title: 'Hard Choices',
                text: 'Turned in a stowaway for the bounty. The crew is quiet.',
              },
            },
          },
          {
            text: 'Give them credits and send them on their way',
            effects: {
              resources: { credits: -25 },
              addChronicle: {
                title: 'Small Kindness',
                text: 'Gave a stowaway enough to start over. Sometimes that\'s all anyone needs.',
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
      minResources: { credits: 40 },
    },
    weight: 12,
    cooldown: 3,
    passages: [
      {
        text: `A mechanic approaches your berth, tools in hand. "I see your hull's taken some hits. I'm between jobs—give me 40 credits and I'll do work worth twice that."`,
        choices: [
          {
            text: 'Accept the offer',
            effects: {
              resources: { credits: -40, hull: 25 },
              addChronicle: {
                title: 'Repairs',
                text: 'Found a skilled mechanic willing to work cheap. The hold is stronger.',
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
      minResources: { credits: 50 },
    },
    weight: 8,
    cooldown: 4,
    passages: [
      {
        text: `In a dim corner of the station, a card game is in progress. The players look up as you approach.

"Fresh blood. Care to test your luck, captain? Fifty credits to play."`,
        choices: [
          {
            text: 'Sit down and play',
            effects: {
              resources: { credits: -50 },
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
        text: `The cards fly. Credits change hands. Fortune favors...`,
        choices: [
          {
            text: 'See the result',
            effects: {
              resources: { credits: 100 },
              addChronicle: {
                title: 'Lucky Night',
                text: 'Won big at cards. The void provides.',
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
        text: `A well-dressed figure intercepts you. "Captain. The Free Traders Guild has noticed your work. We like independent operators who get results.

"Consider this a gift. A sign of goodwill. Perhaps in the future, we might do business."

They hand you a credit chip.`,
        choices: [
          {
            text: 'Accept graciously',
            effects: {
              resources: { credits: 50 },
              addChronicle: {
                title: 'Guild Notice',
                text: 'The Free Traders Guild has taken an interest in our work. Could be useful.',
              },
            },
          },
          {
            text: 'Decline—no strings attached',
            effects: {
              resources: { morale: 5 },
              addChronicle: {
                title: 'Independence',
                text: 'Refused a guild bribe. We answer to no one.',
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
