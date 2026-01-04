import type { Scenelet, SceneletId, FactionId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const factionId = (s: string): FactionId => s as FactionId;

export const journey_dead_star: Scenelet = {
  id: id("journey_dead_star"),
  title: "Stellar Remnant",
  tags: ["environment", "cosmic"],
  requirements: {
    context: "journey",
  },
  weight: 7,
  cooldown: 7,
  passages: [
    {
      text: `Your route passes near a stellar corpse—a black dwarf,
cold and dark, radiating nothing but gravity. The star
that made the atoms in your hull died billions of years
before the humans evolved.
There's something humbling about it. Even stars end.
Your sensors detect unusual particle emissions from the
remnant. Exotic matter, perhaps. Valuable to the right
buyer.`,
      choices: [
        {
          text: "Harvest exotic particles",
          effects: {
            resources: {
              fuel: -6,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Scan for scientific data",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Observe and continue",
          effects: {
            addChronicle: {
              title: "Stellar Remnant",
              text: "Passed a dead star. Older than everything. Even stars end.",
            },
            resources: {
              integrity: 3,
            },
          },
        },
      ],
    },
    {
      text: `You edge closer to the remnant, extending collectors
toward the particle streams. The gravity is treacherous—
you have to fight for every meter.
But the particles flow in. Trace amounts of matter that
shouldn't exist, formed in the heart of a dying star.`,
      choices: [
        {
          text: "Take what you can and withdraw",
          effects: {
            addChronicle: {
              title: "Stellar Remnant",
              text: "Harvested exotic particles from a dead star. The gravity nearly had me.",
            },
            resources: {
              credits: 80,
            },
          },
        },
      ],
    },
    {
      text: `Your sensors sweep the remnant, recording data no one
has gathered before. The star's death throes, frozen
in particle streams. The echo of a supernova that
happened when the universe was young.
The data has value—scientific, historical, philosophical.
A record of endings.`,
      choices: [
        {
          text: "Archive it",
          effects: {
            addChronicle: {
              title: "Stellar Remnant",
              text: "Scanned a dead star. Recorded the echoes of an ancient supernova.",
            },
            setFlags: {
              stellar_data: true,
            },
          },
        },
        {
          text: "Sell it to the Remnant",
          effects: {
            reputation: {
              faction: factionId("remnant"),
              amount: 10,
            },
            addChronicle: {
              title: "Stellar Remnant",
              text: "Scanned a dead star. The Remnant will add it to their archives of endings.",
            },
          },
        },
      ],
    },
  ],
};