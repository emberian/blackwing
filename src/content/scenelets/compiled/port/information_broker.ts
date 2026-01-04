import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_information_broker: Scenelet = {
  id: id("port_information_broker"),
  title: "Data Dealer",
  tags: ["opportunity", "information"],
  requirements: {
    context: "port",
  },
  weight: 8,
  cooldown: 5,
  passages: [
    {
      text: `A data broker makes contact—anonymous signal, professional
demeanor. They deal in information: routes, manifests,
secrets, and coordinates.
"I have inventory. Are you buying, selling, or both?"`,
      choices: [
        {
          text: "What are you selling ?",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "What are you buying ?",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Not interested",
          effects: {
            addChronicle: {
              title: "Data Broker",
              text: "Declined a data broker's offer. Information is dangerous.",
            },
          },
        },
      ],
    },
    {
      text: `"Route intelligence—which lanes are hot, which are cold.
Market predictions—where to buy, where to sell. And
special items for special clients."
They list prices. Route data is affordable. Market
predictions cost more. The "special items" are expensive.`,
      choices: [
        {
          text: "Buy route intelligence",
          effects: {
            addChronicle: {
              title: "Route Intelligence",
              text: "Bought route data from a broker. Knowledge is worth the credits.",
            },
            resources: {
              credits: -30,
            },
          },
        },
        {
          text: "Buy market predictions",
          effects: {
            addChronicle: {
              title: "Market Intelligence",
              text: "Bought market predictions from a broker. Hopefully accurate.",
            },
            resources: {
              credits: -60,
            },
            setFlags: {
              market_intel: true,
            },
          },
        },
        {
          text: "Ask about special items",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `"Anything unusual. Cataclysm-era data. Faction movements.
Sera intelligence. The Hollow Circuit pays well for
certain things."
They pause meaningfully.
"If you find something interesting out there, bring it
to me first."`,
      choices: [
        {
          text: "I 'll keep that in mind",
          effects: {
            addChronicle: {
              title: "Data Broker",
              text: "Made contact with a data broker. They're buying unusual information.",
            },
            setFlags: {
              broker_contact: true,
            },
          },
        },
      ],
    },
    {
      text: `"Coordinates to a salvage site no one's found yet. Fifty
credits. Could be profitable. Could be dangerous."
They don't guarantee anything.`,
      choices: [
        {
          text: "Buy the coordinates",
          effects: {
            addChronicle: {
              title: "Secret Coordinates",
              text: "Bought coordinates to an unknown salvage site. Roll the dice.",
            },
            resources: {
              credits: -50,
            },
            setFlags: {
              secret_coordinates: true,
            },
          },
        },
        {
          text: "Too risky",
          effects: {
            addChronicle: {
              title: "Data Broker",
              text: "Passed on expensive coordinates. Too much unknown.",
            },
          },
        },
      ],
    },
  ],
};