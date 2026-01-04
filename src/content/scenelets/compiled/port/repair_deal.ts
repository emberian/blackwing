import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_repair_deal: Scenelet = {
  id: id("port_repair_deal"),
  title: "Repair Opportunity",
  tags: ["opportunity", "maintenance"],
  requirements: {
    context: "port",
  },
  weight: 9,
  cooldown: 4,
  passages: [
    {
      text: `A repair drone cluster approaches while you're docked—small units,
probably running semi-independent. Their operator's signal tags them
as independent contractors.
"Noticed your hull plating. Could use some work. I'm between jobs,
got materials. Good rate if you're interested."
Your hull is showing wear. The offer is either a good deal or a setup.`,
      choices: [
        {
          text: "Accept the offer",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Inspect their work first",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Repair Opportunity",
              text: "Received an offer for discount repairs. Passed.",
            },
          },
        },
      ],
    },
    {
      text: `You scan the contractor's drones—clean, well-maintained, professional
grade equipment. Their previous work references check out. This looks
legitimate.`,
      choices: [
        {
          text: "Accept the offer",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Still decline",
          effects: {
            addChronicle: {
              title: "Repair Opportunity",
              text: "Inspected an independent repair contractor. Work looked good, but still passed.",
            },
          },
        },
      ],
    },
    {
      text: `The drones swarm over your hull, patching micro-fractures, replacing
worn plating, sealing stress points. It's efficient work—clearly
this contractor knows what they're doing.
When they're done, the Blackwing feels... better. More solid. The
minor aches you'd learned to ignore are gone.`,
      choices: [
        {
          text: "Pay and thank them",
          effects: {
            addChronicle: {
              title: "Repair Deal",
              text: "Had hull repairs done by an independent contractor. Good work, fair price.",
            },
            resources: {
              credits: -35,
              hull: 15,
            },
          },
        },
      ],
    },
  ],
};