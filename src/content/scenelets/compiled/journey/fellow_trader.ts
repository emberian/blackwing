import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_fellow_trader: Scenelet = {
  id: id("journey_fellow_trader"),
  title: "Fellow Traveler",
  tags: ["interaction", "trade"],
  requirements: {
    context: "journey",
  },
  weight: 12,
  cooldown: 3,
  passages: [
    {
      text: `Another vessel on the same vector—a free trader like yourself. They
hail you with the old protocols, the informal signals that mark
those who belong to no faction.
"Running the same route, looks like. Want to share passage for a
while? Safer in pairs."`,
      choices: [
        {
          text: "Accept their company",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Exchange news and continue separately",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Politely decline and move on",
          effects: {
            addChronicle: {
              title: "Fellow Trader",
              text: "Encountered another free trader on the same route. Went our separate ways.",
            },
          },
        },
      ],
    },
    {
      text: `You fall into formation with the other vessel—a modified bulk hauler,
patches visible on the hull. The pilot's signal is warm, easy.
"Name's Junction. Run cargo out of Relay Nine mostly. You?"
The journey passes faster with company. Junction shares route data,
warns you about a patrol they spotted two jumps back. When you reach
the next waypoint, they signal farewell.
"Safe travels, Blackwing. Maybe we'll cross paths again."`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "Shared Passage",
              text: "Traveled with another free trader for a while. Company helps in the void.",
            },
            resources: {
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `Junction shares what they know: market conditions at Crucible,
Flotilla patrols increasing near the Sera front, a rumor about
the Hollow Circuit opening new channels.
You trade information in kind. Brief, professional, useful.`,
      choices: [
        {
          text: "Part ways",
          effects: {
            addChronicle: {
              title: "Trader Exchange",
              text: "Exchanged news with a fellow free trader. Useful intelligence gathered.",
            },
          },
        },
      ],
    },
  ],
};