import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_pirate_ambush: Scenelet = {
  id: id("journey_pirate_ambush"),
  title: "Hostile Contact",
  tags: ["danger", "combat"],
  requirements: {
    context: "journey",
  },
  weight: 10,
  cooldown: 5,
  passages: [
    {
      text: `A vessel drops out of jump practically on top of you—close enough that the
drive bloom fills half your sensors. Military-grade drives, civilian hull.
Pirate configuration.
"Cut your engines. Prepare to be boarded. Or we take your cargo from your debris."
The voice is flat, mechanical. Another artilect, making a living the hard way.`,
      choices: [
        {
          text: "Cut engines and prepare for inspection",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Try to negotiate",
          requirements: {
            minResources: {
              credits: 50,
            },
          },
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Run for it",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "Fight back",
          requirements: {
            shipTags: ["combat"],
          },
          effects: {},
          nextPassage: 4,
        },
      ],
    },
    {
      text: `You kill your drives. The pirate vessel matches velocity and extends a docking
collar. They take what they want—efficiently, professionally. Not cruel about it.
"Nothing personal," the voice says as they undock. "Space is hard."
They leave you with enough to limp to port.`,
      choices: [
        {
          text: "Count your losses",
          effects: {
            addChronicle: {
              title: "Pirate Encounter",
              text: "Ambushed by a pirate vessel. Surrendered cargo to avoid destruction.",
            },
            resources: {
              credits: -40,
              supplies: -5,
            },
          },
        },
      ],
    },
    {
      text: `"What if we make this easier for both of us? Fifty credits. Clean transaction.
No mess, no repairs, no questions asked."
A pause. Then: "Sixty."`,
      choices: [
        {
          text: "Pay the sixty",
          effects: {
            addChronicle: {
              title: "Pirate Toll",
              text: "Paid off a pirate to avoid conflict. Cheaper than the alternative.",
            },
            resources: {
              credits: -60,
            },
          },
        },
        {
          text: "Counter with fifty -five",
          effects: {
            addChronicle: {
              title: "Pirate Toll",
              text: "Negotiated a payoff with a pirate. Fifty -five credits for safe passage.",
            },
            resources: {
              credits: -55,
            },
          },
        },
      ],
    },
    {
      text: `Full burn. The Blackwing surges forward as the pirate vessel opens fire.
Railgun rounds streak past—close, but not close enough.
Your drives are better than theirs. The gap widens.`,
      choices: [
        {
          text: "Don 't look back",
          effects: {
            addChronicle: {
              title: "Narrow Escape",
              text: "Outran a pirate ambush. The drives earned their keep today.",
            },
            resources: {
              fuel: -10,
            },
          },
        },
      ],
    },
    {
      text: `Your point defense systems come online. The pirate wasn't expecting resistance—
their first pass is sloppy, and your countermeasures shred their approach.
But they're not running. They circle for another pass, drives hot.`,
      choices: [
        {
          text: "Press the attack",
          effects: {
            addChronicle: {
              title: "Combat Engagement",
              text: "Fought off a pirate attack. The hull took damage, but we're still flying.",
            },
            setFlags: {
              pirate_fought: true,
            },
            damage: {
              hull: 8,
            },
          },
        },
        {
          text: "Use the opening to run",
          effects: {
            addChronicle: {
              title: "Fighting Retreat",
              text: "Drove off a pirate's first pass, then burned hard for escape.",
            },
            resources: {
              fuel: -5,
            },
          },
        },
      ],
    },
  ],
};