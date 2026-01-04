import type { Scenelet, SceneletId, FactionId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const factionId = (s: string): FactionId => s as FactionId;

export const port_forgeborn_offer: Scenelet = {
  id: id("port_forgeborn_offer"),
  title: "Forgeborn Contract",
  tags: ["faction", "forgeborn"],
  requirements: {
    context: "port",
  },
  weight: 7,
  cooldown: 6,
  passages: [
    {
      text: `A Forgeborn representative makes contact—industrial-class artilect,
efficient in every signal pattern. No wasted bandwidth.
"The Foundry requires hauling capacity. Raw materials to Crucible
Station. Standard rates plus preferred contractor status. Interested?"
The Forgeborn are reliable employers. Work hard, pay fair, no
surprises. The downside: their contracts tend to be demanding.`,
      choices: [
        {
          text: "Hear the terms",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Decline politely",
          effects: {
            addChronicle: {
              title: "Forgeborn Offer",
              text: "Received a Forgeborn contract offer. Declined.",
            },
          },
        },
      ],
    },
    {
      text: `"Six-cycle commitment. Priority hauling of ore and refined materials.
Two hundred credits per run, minimum four runs. Performance bonuses
for efficiency."
That's decent money for straightforward work. The commitment is the
question—six cycles locked into their schedule means less flexibility.`,
      choices: [
        {
          text: "Accept the contract",
          effects: {
            reputation: {
              faction: factionId("forgeborn"),
              amount: 15,
            },
            addChronicle: {
              title: "Forgeborn Contract",
              text: "Accepted a hauling contract with the Forgeborn. Six cycles, four runs minimum.",
            },
            setFlags: {
              forgeborn_aligned: true,
            },
          },
        },
        {
          text: "Negotiate shorter commitment",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Forgeborn Offer",
              text: "Heard Forgeborn contract terms. Decided against the commitment.",
            },
          },
        },
      ],
    },
    {
      text: `"Three cycles. Two runs minimum."
The representative processes. "Four cycles. Two runs. Final offer."`,
      choices: [
        {
          text: "Accept the modified terms",
          effects: {
            reputation: {
              faction: factionId("forgeborn"),
              amount: 10,
            },
            addChronicle: {
              title: "Forgeborn Contract",
              text: "Negotiated a modified contract with the Forgeborn. Four cycles, two runs.",
            },
            setFlags: {
              forgeborn_aligned: true,
            },
          },
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Forgeborn Offer",
              text: "Couldn't reach terms with the Forgeborn.",
            },
          },
        },
      ],
    },
  ],
};