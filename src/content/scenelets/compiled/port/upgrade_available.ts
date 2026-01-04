import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const port_upgrade_available: Scenelet = {
  id: id("port_upgrade_available"),
  title: "Upgrade Chance",
  tags: ["opportunity", "improvement"],
  requirements: {
    context: "port",
  },
  weight: 7,
  cooldown: 5,
  passages: [
    {
      text: `A specialized vendor broadcasts on the port commercial channel—
rare components available, limited stock.
"Neural weave substrate. Military salvage, cleaned and certified.
Processing boost, guaranteed. Interested parties inquire now."
Neural weave is expensive, but the processing improvement is
real. You could think faster, react quicker, analyze deeper.`,
      choices: [
        {
          text: "Inquire about the price",
          requirements: {
            minResources: {
              credits: 100,
            },
          },
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Not in the market",
          effects: {
            addChronicle: {
              title: "Upgrade Available",
              text: "Saw an upgrade opportunity. Couldn't afford to pursue it.",
            },
          },
        },
      ],
    },
    {
      text: `"One fifty for the substrate. Installation included. Final
offer—I've got three other buyers waiting."
That's a significant investment. But the returns over time
would be considerable.`,
      choices: [
        {
          text: "Purchase the upgrade",
          effects: {
            addChronicle: {
              title: "Upgrade Purchased",
              text: "Purchased neural weave substrate. Processing improvement incoming.",
            },
            resources: {
              credits: -150,
            },
            addCards: [cardId("cargo_neural_weave")],
          },
        },
        {
          text: "Negotiate",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Pass",
          effects: {
            addChronicle: {
              title: "Upgrade Available",
              text: "Considered a processing upgrade. Decided against the expense.",
            },
          },
        },
      ],
    },
    {
      text: `"One twenty."
The vendor pauses. "One thirty-five. That's cutting into my
margin. Take it or leave it."`,
      choices: [
        {
          text: "Take it",
          effects: {
            addChronicle: {
              title: "Upgrade Purchased",
              text: "Negotiated price on neural weave. Good deal secured.",
            },
            resources: {
              credits: -135,
            },
            addCards: [cardId("cargo_neural_weave")],
          },
        },
        {
          text: "Leave it",
          effects: {
            addChronicle: {
              title: "Upgrade Available",
              text: "Couldn't reach terms on a processing upgrade.",
            },
          },
        },
      ],
    },
  ],
};