import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_jumpgate_debris: Scenelet = {
  id: id("journey_jumpgate_debris"),
  title: "Broken Gate",
  tags: ["environment", "discovery"],
  requirements: {
    context: "journey",
  },
  weight: 6,
  cooldown: 8,
  passages: [
    {
      text: `The jumpgate appears on sensors as it should—a massive
ring of alien-derived technology, promising instant
transit across light-years.
Then you get closer and see the truth.
It's broken. The ring is shattered, fragments spinning
slowly in the void. Whatever happened here, the gate
won't be opening again.`,
      choices: [
        {
          text: "Investigate the damage",
          effects: {
            resources: {
              fuel: -3,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Salvage what you can",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Log it and continue",
          effects: {
            addChronicle: {
              title: "Broken Gate",
              text: "Found a destroyed jumpgate. Another connection severed. Logged and moved on.",
            },
          },
        },
      ],
    },
    {
      text: `The damage patterns tell a story. Not impact—the fragments
would be different. Not age—the materials should last
millennia. Something overloaded the gate's systems. Something
made it tear itself apart.
The Cataclysm? A Sera attack? Sabotage? The wreckage
holds no answers.
But there's something in the debris. A navigation buoy,
still broadcasting. Still trying to guide ships to a
gate that no longer exists.`,
      choices: [
        {
          text: "Shut down the buoy",
          effects: {
            addChronicle: {
              title: "Broken Gate",
              text: "Found a destroyed jumpgate. Shut down the navigation buoy. No one should look for this anymore.",
            },
            resources: {
              integrity: 3,
            },
          },
        },
        {
          text: "Leave it broadcasting",
          effects: {
            addChronicle: {
              title: "Broken Gate",
              text: "Found a destroyed jumpgate. Left the buoy running. Maybe someone will investigate.",
            },
            setFlags: {
              broken_gate_logged: true,
            },
          },
        },
      ],
    },
    {
      text: `Jumpgate materials are valuable—the alloys alone are
worth significant credits. You pick through the debris,
harvesting what you can.
It feels wrong, somehow. Like grave robbing. But the
gate's not using the material anymore.`,
      choices: [
        {
          text: "Take the components",
          effects: {
            addChronicle: {
              title: "Broken Gate",
              text: "Salvaged components from a destroyed jumpgate. The gate won't miss them.",
            },
            resources: {
              credits: 60,
            },
          },
        },
        {
          text: "Leave it intact",
          effects: {
            addChronicle: {
              title: "Broken Gate",
              text: "Found a destroyed jumpgate. Couldn't bring myself to salvage it.",
            },
            resources: {
              integrity: 2,
            },
          },
        },
      ],
    },
  ],
};