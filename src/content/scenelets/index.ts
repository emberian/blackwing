import type { Scenelet, SceneletId } from '../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const TRANSIT_SCENELETS: Scenelet[] = [
  {
    id: id('transit_strange_signal'),
    title: 'Strange Signal',
    tags: ['mystery', 'discovery'],
    requirements: {
      location: 'transit',
      shipTags: ['sensor'],
    },
    weight: 10,
    cooldown: 50,
    passages: [
      {
        text: `The sensor array chirps—an anomaly in the void. A signal, old beyond measure, repeating in patterns that feel almost like language.

It comes from somewhere off your plotted course. Investigating would cost time and fuel.`,
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
        text: `The source is a derelict—older than your records can identify. Its hull is pocked with micrometeorite impacts spanning millennia.

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
    id: id('transit_hull_breach'),
    title: 'Micrometeorite Impact',
    tags: ['danger', 'ship'],
    requirements: {
      location: 'transit',
    },
    weight: 15,
    cooldown: 20,
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
    id: id('transit_crew_conflict'),
    title: 'Tension in the Hold',
    tags: ['crew', 'social'],
    requirements: {
      location: 'transit',
      minResources: { morale: 20 },
      maxResources: { morale: 60 },
    },
    weight: 12,
    cooldown: 30,
    passages: [
      {
        text: `Voices raised in the mess. The long dark wears on everyone differently.

Two of your crew stand chest to chest, grievances spilling out that have been building for years—subjective years, anyway.`,
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
    id: id('transit_dream'),
    title: 'Dreams of Elsewhere',
    tags: ['mystery', 'narrative'],
    requirements: {
      location: 'transit',
    },
    weight: 5,
    cooldown: 100,
    passages: [
      {
        text: `You wake from cryo-doze with a memory that isn't yours.

A world of amber skies. A name you've never heard spoken. The certainty that you've been here before, in some other life, some other hold.

The feeling fades with waking, but something lingers.`,
        choices: [
          {
            text: 'Record the vision',
            effects: {
              setFlags: { 'dream_recorded': true },
              addChronicle: {
                title: 'A Dream Remembered',
                text: 'Dreams in the long dark. They feel too real to dismiss.',
              },
            },
          },
          {
            text: 'Dismiss it as transit fatigue',
            effects: {},
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
      location: 'port',
      minResources: { credits: 50 },
    },
    weight: 15,
    cooldown: 25,
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
      location: 'port',
    },
    weight: 8,
    cooldown: 75,
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
        text: `"Three hundred years I've hauled cargo. Subjective years, mind. The void doesn't count time the same.

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
          {
            text: 'Ask about the Lines',
            requirements: {
              requiredFlags: ['signal_investigated'],
            },
            effects: {
              setFlags: { 'knows_about_lines': true },
              addChronicle: {
                title: 'The Lines',
                text: 'The old captain spoke of the Shatterling Lines—immortal travelers who measure time in civilizations.',
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
      location: 'port',
      excludedFlags: ['refused_smuggling'],
    },
    weight: 10,
    cooldown: 40,
    passages: [
      {
        text: `A message on your private channel. No sender ID. Just coordinates in the lower docks and a time.

When you arrive, a figure in worn but expensive clothes is waiting. "Your ship. Your discretion. My cargo. Destination: Shadow Market. Payment on delivery. Interested?"`,
        choices: [
          {
            text: 'Accept the job',
            effects: {
              addCards: ['cargo_contraband' as any],
              setFlags: { 'smuggling_active': true, 'smuggling_destination': 'port_shadow_market' },
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
];

export const ALL_SCENELETS: Scenelet[] = [
  ...TRANSIT_SCENELETS,
  ...PORT_SCENELETS,
];
