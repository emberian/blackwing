import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const port_contraband_offer: Scenelet = {
  id: id("port_contraband_offer"),
  title: "Discrete Cargo",
  tags: ["trade", "illegal"],
  requirements: {
    context: "port",
  },
  weight: 8,
  cooldown: 5,
  passages: [
    {
      text: `The message comes through anonymous channels—voice synthesized,
signal bounced through a dozen relays.
"I have cargo that needs to move. Discrete transport. No Compact
manifest. Pays well."
Smuggling. The speaker doesn't hide what they're asking.`,
      choices: [
        {
          text: "Hear the details",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Not interested",
          effects: {
            addChronicle: {
              title: "Contraband Offer",
              text: "Received an offer for smuggling work. Declined to hear details.",
            },
          },
        },
      ],
    },
    {
      text: `"Sealed containers. You don't open them, don't scan them, don't
ask what's inside. Delivery to Scatterpoint. Two hundred credits
on arrival."
That's good money for a cargo run. The catch is obvious: Compact
patrols, inspections, the risk if you're caught carrying contraband.`,
      choices: [
        {
          text: "Accept the job",
          effects: {
            addChronicle: {
              title: "Discrete Cargo",
              text: "Accepted a smuggling contract. Sealed containers to Scatterpoint.",
            },
            addCards: [cardId("cargo_contraband")],
            setFlags: {
              smuggling_job_accepted: true,
            },
          },
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Contraband Offer",
              text: "Received a smuggling offer. Decided the risk wasn't worth it.",
            },
            setFlags: {
              refused_smuggling: true,
            },
          },
        },
        {
          text: "Decline firmly —no smuggling work, ever",
          effects: {
            addChronicle: {
              title: "Contraband Offer",
              text: "Received a smuggling offer. Made clear I don't do that kind of work.",
            },
            setFlags: {
              refused_smuggling_twice: true,
            },
          },
        },
      ],
    },
  ],
};