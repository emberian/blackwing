import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_passenger_request: Scenelet = {
  id: id("port_passenger_request"),
  title: "Passenger Manifest",
  tags: ["opportunity", "transport"],
  requirements: {
    context: "port",
  },
  weight: 9,
  cooldown: 4,
  passages: [
    {
      text: `A passenger request comes through—another artilect looking
for transport. Passage on a cargo vessel is cheaper than
dedicated transit, and free traders don't ask questions.
"I need to reach Relay Nine. I can pay, and I don't take
much space."`,
      choices: [
        {
          text: "Ask why they need transport",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Name your price",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Passenger Request",
              text: "Declined a passenger request. Cargo doesn't ask questions.",
            },
          },
        },
      ],
    },
    {
      text: `"Personal reasons. Nothing illegal. Just... complicated."
They hesitate, then: "I'm relocating. Leaving someone
behind. A clean break requires distance."
A relationship ending. Common enough among artilects.
Even digital minds need physical separation sometimes.`,
      choices: [
        {
          text: "Offer transport",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Wish them well but decline",
          effects: {
            addChronicle: {
              title: "Passenger Request",
              text: "Declined to transport a passenger. Wished them well.",
            },
            resources: {
              integrity: 2,
            },
          },
        },
      ],
    },
    {
      text: `"Forty credits to Relay Nine. You stay out of the way,
don't touch the cargo, don't cause problems."
"Thirty."`,
      choices: [
        {
          text: "Accept thirty",
          effects: {
            addChronicle: {
              title: "Passenger Transport",
              text: "Took a passenger to Relay Nine. Thirty credits. They were quiet.",
            },
            resources: {
              credits: 30,
            },
            setFlags: {
              passenger_taken: true,
            },
          },
        },
        {
          text: "Forty or nothing",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `"Forty. I'm not running a charity."
A pause. "Fine. Forty."
The credits transfer. You have a passenger now. Another
mind sharing your space for a little while.`,
      choices: [
        {
          text: "Welcome them aboard",
          effects: {
            addChronicle: {
              title: "Passenger Transport",
              text: "Took a passenger to Relay Nine. Held the line on price.",
            },
            resources: {
              credits: 40,
            },
            setFlags: {
              passenger_taken: true,
            },
          },
        },
      ],
    },
  ],
};