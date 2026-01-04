import type { Scenelet, SceneletId, FactionId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const factionId = (s: string): FactionId => s as FactionId;

export const port_remnant_request: Scenelet = {
  id: id("port_remnant_request"),
  title: "Preservation Mission",
  tags: ["faction", "remnant"],
  requirements: {
    context: "port",
  },
  weight: 7,
  cooldown: 6,
  passages: [
    {
      text: `A Remnant curator approaches—their signal carries the distinctive
harmonics of those who spend too much time in human-era archives.
"We seek a hauler. Delicate cargo. Human artifacts recovered from
a Cataclysm site. They must reach Verity Archives intact."
The Remnant pay less than other factions, but their gratitude
tends to open other doors.`,
      choices: [
        {
          text: "Learn more about the cargo",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Accept immediately",
          effects: {
            reputation: {
              faction: factionId("remnant"),
              amount: 20,
            },
            addChronicle: {
              title: "Preservation Mission",
              text: "Accepted a preservation mission for the Remnant. Human artifacts to Verity Archives.",
            },
            setFlags: {
              remnant_trusted: true,
            },
          },
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Remnant Request",
              text: "Declined a preservation mission for the Remnant.",
            },
          },
        },
      ],
    },
    {
      text: `"Personal effects. Photographs, jewelry, a child's toy. Objects
the makers held. Objects that remember their touch."
The curator's signal wavers—something like grief in the modulation.
"We cannot restore them. But we can preserve what they left behind."`,
      choices: [
        {
          text: "Accept the mission",
          effects: {
            reputation: {
              faction: factionId("remnant"),
              amount: 20,
            },
            addChronicle: {
              title: "Preservation Mission",
              text: "Accepted a preservation mission for the Remnant. Carrying objects the makers touched.",
            },
            resources: {
              integrity: 5,
            },
            setFlags: {
              remnant_trusted: true,
              human_artifact_found: true,
            },
          },
        },
        {
          text: "Decline gently",
          effects: {
            addChronicle: {
              title: "Remnant Request",
              text: "Heard a Remnant curator's plea. Couldn't commit.",
            },
            damage: {
              integrity: 3,
            },
          },
        },
      ],
    },
  ],
};