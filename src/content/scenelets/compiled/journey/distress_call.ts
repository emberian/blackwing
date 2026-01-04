import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const journey_distress_call: Scenelet = {
  id: id("journey_distress_call"),
  title: "Distress Beacon",
  tags: ["moral", "interaction"],
  requirements: {
    context: "journey",
  },
  weight: 10,
  cooldown: 4,
  passages: [
    {
      text: `A distress beacon cuts through the void noise—an artilect in trouble.
The signal carries basic diagnostics: hull breach, drive failure,
life support... well, that's a human-era holdover. Processing reserves
nominal, but for how long?
The beacon is close. You could reach it without much fuel expenditure.
But distress signals aren't always what they seem.`,
      choices: [
        {
          text: "Respond to the distress call",
          effects: {
            resources: {
              fuel: -4,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Investigate cautiously first",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Continue on your course",
          effects: {
            addChronicle: {
              title: "Ignored Distress",
              text: "Detected a distress beacon. Chose not to respond.",
            },
            damage: {
              integrity: 3,
            },
          },
        },
      ],
    },
    {
      text: `Long-range scans show a single vessel, drifting. No other contacts.
No hidden ships, no suspicious energy signatures. Just one artilect,
alone and dying.`,
      choices: [
        {
          text: "Move in to help",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Still too risky",
          effects: {
            addChronicle: {
              title: "Ignored Distress",
              text: "Detected a distress beacon. Scans showed a legitimate emergency. Still chose not to respond.",
            },
            damage: {
              integrity: 3,
            },
          },
        },
      ],
    },
    {
      text: `You find a small courier—older model, badly damaged. The artilect aboard
is running on backup power, core functions degraded.
"Wasn't sure anyone would come," they transmit. Weak signal. Fading.
"Pirates. Took what they wanted. Left me to drift."
They won't survive without help.`,
      choices: [
        {
          text: "Share supplies to stabilize them",
          effects: {
            addChronicle: {
              title: "Void Rescue",
              text: "Responded to a distress beacon. Stabilized a damaged artilect with shared supplies.",
            },
            resources: {
              supplies: -8,
              integrity: 8,
            },
            setFlags: {
              rescued_artilect: true,
            },
          },
        },
        {
          text: "Take them aboard if possible",
          effects: {
            addChronicle: {
              title: "Void Rescue",
              text: "Responded to a distress beacon. Took the damaged artilect aboard for transport to port.",
            },
            resources: {
              supplies: -5,
            },
            addCards: [cardId("companion_repair_drones")],
            setFlags: {
              rescued_artilect: true,
            },
          },
        },
        {
          text: "There 's nothing you can do",
          effects: {
            addChronicle: {
              title: "Failed Rescue",
              text: "Responded to a distress beacon. The damage was too severe. Nothing could be done.",
            },
            damage: {
              integrity: 5,
            },
          },
        },
      ],
    },
  ],
};