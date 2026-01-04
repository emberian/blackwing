import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_debt_collector: Scenelet = {
  id: id("port_debt_collector"),
  title: "Outstanding Balance",
  tags: ["character", "threat"],
  requirements: {
    context: "port",
  },
  weight: 6,
  cooldown: 8,
  passages: [
    {
      text: `A message arrives, formal and cold.
"Blackwing. Our records indicate an outstanding balance.
Services rendered, payment not received. We're here to
collect."
You don't remember the debt. But memory is unreliable—
especially for an artilect who woke up in a salvage
yard with thirty years of gaps.`,
      choices: [
        {
          text: "Ask for details",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Pay immediately",
          requirements: {
            minResources: {
              credits: 100,
            },
          },
          effects: {
            addChronicle: {
              title: "Debt Collected",
              text: "Paid a debt I don't remember incurring. Might have been legitimate.",
            },
            resources: {
              credits: -100,
            },
          },
        },
        {
          text: "Dispute the claim",
          effects: {},
          nextPassage: 2,
        },
      ],
    },
    {
      text: `They transmit documentation. Old. Pre-your-awakening old.
Repair services at a station you don't recognize, billed
to a registration that matches your core signature.
It could be legitimate. It could be fabricated. There's
no way to know.
"One hundred credits resolves this matter."`,
      choices: [
        {
          text: "Pay it",
          effects: {
            addChronicle: {
              title: "Debt Collected",
              text: "Paid an old debt. The documentation looked legitimate. Maybe.",
            },
            resources: {
              credits: -100,
            },
          },
        },
        {
          text: "Negotiate",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "Refuse to pay",
          effects: {},
          nextPassage: 4,
        },
      ],
    },
    {
      text: `"I have no record of this debt."
"Your records are incomplete. Ours aren't."
They don't seem interested in arguing. They seem
interested in payment.`,
      choices: [
        {
          text: "Ask for details",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Refuse",
          effects: {},
          nextPassage: 4,
        },
      ],
    },
    {
      text: `"Fifty credits. That's what I can offer."
A pause. "Seventy-five. Our expenses must be covered."`,
      choices: [
        {
          text: "Accept seventy -five",
          effects: {
            addChronicle: {
              title: "Debt Settled",
              text: "Negotiated an old debt down to seventy -five credits.",
            },
            resources: {
              credits: -75,
            },
          },
        },
        {
          text: "Refuse",
          effects: {},
          nextPassage: 4,
        },
      ],
    },
    {
      text: `"I'm not paying for a debt I don't remember."
The response is immediate. "Then we'll recover the value
through other means. Your reputation will reflect this
decision."
The connection closes. You've made an enemy. Maybe a
legitimate creditor. Maybe a scammer.
Either way, they'll remember.`,
      choices: [
        {
          text: "Accept the consequences",
          effects: {
            addChronicle: {
              title: "Debt Refused",
              text: "Refused to pay an old debt. Made enemies. Might have been the right call.",
            },
            setFlags: {
              debt_refused: true,
            },
          },
        },
      ],
    },
  ],
};