import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const port_price_crash: Scenelet = {
  id: id("port_price_crash"),
  title: "Market Volatility",
  tags: ["trade", "event"],
  requirements: {
    context: "port",
  },
  weight: 7,
  cooldown: 6,
  passages: [
    {
      text: `The market alerts cascade through your systems—prices
shifting, opportunities appearing, chaos in the exchanges.
Something has disrupted supply lines. A Sera incursion,
maybe. A major contract falling through. The cause
doesn't matter. What matters is the opportunity.
Prices are moving fast. You need to decide faster.`,
      choices: [
        {
          text: "Buy commodities before prices rise",
          effects: {},
        },
      ],
    },
    {
      text: `You dump credits into the market, buying whatever's
undervalued before the rest of the traders catch on.
The timing is good. Prices stabilize higher than you
paid. Profit.`,
      choices: [
        {
          text: "Secure the goods",
          effects: {
            addChronicle: {
              title: "Market Play",
              text: "Bought commodities during a market dip. Good timing.",
            },
            resources: {
              credits: -80,
            },
            addCards: [cardId("cargo_refined_metals"), cardId("cargo_common_components")],
          },
        },
      ],
    },
    {
      text: `You move fast, liquidating cargo before the price floor
drops. Other traders are doing the same—it's a race to
the exits.
You make it out ahead. Not maximum value, but better
than holding through the crash.`,
      choices: [
        {
          text: "Take the credits",
          effects: {
            addChronicle: {
              title: "Market Play",
              text: "Sold cargo during market volatility. Got out before the crash.",
            },
            resources: {
              credits: 50,
            },
          },
        },
      ],
    },
  ],
};