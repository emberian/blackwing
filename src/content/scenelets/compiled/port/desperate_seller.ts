import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const port_desperate_seller: Scenelet = {
  id: id("port_desperate_seller"),
  title: "Desperate Seller",
  tags: ["trade", "opportunity"],
  requirements: {
    context: "port",
  },
  weight: 10,
  cooldown: 4,
  passages: [
    {
      text: `A message arrives on an open channel—an artilect looking to offload
cargo. Fast. Below market value. Way below.
"Need to liquidate. Now. No questions. Fair price for fast credits."
The signal is jittery, stressed. Whatever's pushing them, they're
not bluffing about the urgency.`,
      choices: [
        {
          text: "Meet with them",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Ask what the cargo is first",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Ignore the message",
          effects: {
            addChronicle: {
              title: "Desperate Seller",
              text: "Received a liquidation offer. Declined to investigate.",
            },
          },
        },
      ],
    },
    {
      text: `"Refined metals. Clean provenance. You can verify."
They transmit shipping records, authenticity seals. Everything
looks legitimate. The question isn't whether the cargo is real—
it's why they're so desperate to dump it.`,
      choices: [
        {
          text: "Meet with them",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Still not interested",
          effects: {
            addChronicle: {
              title: "Desperate Seller",
              text: "Received a liquidation offer. Cargo was clean. Still passed.",
            },
          },
        },
      ],
    },
    {
      text: `You find them in a service corridor—a small transport artilect,
drives running hot even at dock. They're ready to bolt.
"Twenty units refined metals. Market value's about two hundred.
I'll take one hundred. Cash. Now."`,
      choices: [
        {
          text: "Pay the hundred",
          effects: {
            addChronicle: {
              title: "Desperate Seller",
              text: "Bought cargo from a desperate seller. Good deal, no questions asked.",
            },
            resources: {
              credits: -100,
            },
            addCards: [cardId("cargo_refined_metals"), cardId("cargo_refined_metals")],
          },
        },
        {
          text: "Offer eighty",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "This feels wrong . Pass .",
          effects: {
            addChronicle: {
              title: "Desperate Seller",
              text: "Met a desperate seller. Decided against the deal.",
            },
          },
        },
      ],
    },
    {
      text: `"Eighty," you transmit.
They hesitate. Every second of that hesitation tells you how bad
their situation is.
"Fine. Eighty. Just transfer and take it."`,
      choices: [
        {
          text: "Complete the transaction",
          effects: {
            addChronicle: {
              title: "Desperate Seller",
              text: "Negotiated a desperate seller down. Good deal, some guilt.",
            },
            resources: {
              credits: -80,
            },
            addCards: [cardId("cargo_refined_metals"), cardId("cargo_refined_metals")],
          },
        },
      ],
    },
  ],
};