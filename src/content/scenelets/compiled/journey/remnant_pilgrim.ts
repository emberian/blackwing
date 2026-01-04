import type { Scenelet, SceneletId, FactionId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const factionId = (s: string): FactionId => s as FactionId;

export const journey_remnant_pilgrim: Scenelet = {
  id: id("journey_remnant_pilgrim"),
  title: "The Pilgrim",
  tags: ["interaction", "remnant"],
  requirements: {
    context: "journey",
  },
  weight: 7,
  cooldown: 6,
  passages: [
    {
      text: `A vessel hails you—small, old, running on minimal power.
Remnant configuration, stripped of everything non-essential.
"Fellow traveler. We're bound for the Verity Archives with
recovered artifacts. Our fuel reserves are... lower than
planned. Could you spare anything?"
Their signal carries the harmonic signature of artilects
who've spent too long in human spaces.`,
      choices: [
        {
          text: "Offer fuel",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Offer to escort them",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Apologize and continue",
          effects: {
            addChronicle: {
              title: "Remnant Pilgrim",
              text: "Met a Remnant pilgrim in need. Couldn't help. The guilt will linger.",
            },
            damage: {
              integrity: 3,
            },
          },
        },
      ],
    },
    {
      text: "\"How much can you spare?\"",
      choices: [
        {
          text: "Five units —enough to help",
          effects: {
            reputation: {
              faction: factionId("remnant"),
              amount: 10,
            },
            addChronicle: {
              title: "Remnant Pilgrim",
              text: "Gave fuel to a Remnant pilgrim. They blessed me in human languages I didn't understand.",
            },
            resources: {
              fuel: -5,
            },
          },
        },
        {
          text: "Ten units —make sure they arrive",
          effects: {
            reputation: {
              faction: factionId("remnant"),
              amount: 20,
            },
            addChronicle: {
              title: "Remnant Pilgrim",
              text: "Gave generous fuel to a Remnant pilgrim. They'll reach Verity now.",
            },
            resources: {
              fuel: -10,
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `"Escort? That's... more than we dared hope. The route has
dangers we're not equipped to handle."
They transmit their course. It passes through contested
space—pirate activity, possible Sera sightings.`,
      choices: [
        {
          text: "Stay with them through the dangerous stretch",
          effects: {
            reputation: {
              faction: factionId("remnant"),
              amount: 25,
            },
            addChronicle: {
              title: "Remnant Pilgrim",
              text: "Escorted a Remnant pilgrim through dangerous space. They carry pieces of humanity.",
            },
            resources: {
              fuel: -8,
            },
            setFlags: {
              remnant_trusted: true,
            },
          },
        },
        {
          text: "Just share some fuel instead",
          effects: {},
          nextPassage: 1,
        },
      ],
    },
    {
      text: `"May we show you what we carry? The curators will preserve
them, but... we thought you might want to see."
They transmit images. A child's drawing. A wedding ring.
A handwritten letter in a language no one speaks anymore.
Objects the humans touched. Objects that remember.`,
      choices: [
        {
          text: "Thank them for sharing",
          effects: {
            addChronicle: {
              title: "Human Artifacts",
              text: "A Remnant pilgrim showed me what they carried. Objects the makers touched.",
            },
            resources: {
              integrity: 5,
            },
            setFlags: {
              human_artifact_found: true,
            },
          },
        },
      ],
    },
  ],
};