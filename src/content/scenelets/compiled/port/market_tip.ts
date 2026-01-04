import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_market_tip: Scenelet = {
  id: id("port_market_tip"),
  title: "Market Intelligence",
  tags: ["trade", "information"],
  requirements: {
    context: "port",
  },
  weight: 9,
  cooldown: 4,
  passages: [
    {
      text: `An information broker sidles up on a secure channel—one of the
station's regular fixtures. They deal in data: routes, prices,
movements.
"I have something you might find profitable. Market movements.
Fifty credits for the full package."
Information is currency. The question is whether this particular
information is worth the price.`,
      choices: [
        {
          text: "Pay for the intel",
          effects: {
            resources: {
              credits: -50,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Negotiate",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Market Tip",
              text: "An information broker offered market intelligence. Declined.",
            },
          },
        },
      ],
    },
    {
      text: `"Thirty," you counter.
"Forty. That's as low as I go. Information this fresh, I could
sell to three other traders before you finish docking."`,
      choices: [
        {
          text: "Pay forty",
          effects: {
            resources: {
              credits: -40,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Walk away",
          effects: {
            addChronicle: {
              title: "Market Tip",
              text: "Negotiated with an information broker. Couldn't reach a deal.",
            },
          },
        },
      ],
    },
    {
      text: `The data packet arrives: price fluctuations, supply shortages,
a Forgeborn contract opening up that hasn't hit public channels yet.
Solid intelligence. If you act fast, you can make the investment
back several times over.`,
      choices: [
        {
          text: "Return to your business",
          effects: {
            addChronicle: {
              title: "Market Intelligence",
              text: "Purchased market intelligence from a broker. Promising leads acquired.",
            },
            setFlags: {
              market_tip_received: true,
            },
          },
        },
      ],
    },
  ],
};