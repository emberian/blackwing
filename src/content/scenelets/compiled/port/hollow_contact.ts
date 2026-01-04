import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_hollow_contact: Scenelet = {
  id: id("port_hollow_contact"),
  title: "Anonymous Message",
  tags: ["faction", "hollow"],
  requirements: {
    context: "port",
  },
  weight: 5,
  cooldown: 8,
  passages: [
    {
      text: `The message arrives without sender information, bounced through
so many relays that tracing it would take cycles. The text is
simple:
"We know what you've been looking for. We can help. Meeting
point attached. Come alone."
The Hollow Circuit. They find you when they want to find you.`,
      choices: [
        {
          text: "Go to the meeting point",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Ignore it",
          effects: {
            addChronicle: {
              title: "Hollow Contact",
              text: "Received a message from the Hollow Circuit. Chose not to respond.",
            },
          },
        },
      ],
    },
    {
      text: `The location is a service corridor, sensors dark, no witnesses.
A platform waits—anonymous, featureless, impossible to identify.
"Blackwing." They know your name. "You've been asking questions.
About the Cataclysm. About your own past."
They pause.
"We have information. But information has a price. Are you
willing to pay?"`,
      choices: [
        {
          text: "What 's the price?",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "What information ?",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "Leave now",
          effects: {
            addChronicle: {
              title: "Hollow Contact",
              text: "Met with the Hollow Circuit. Left before hearing their offer.",
            },
          },
        },
      ],
    },
    {
      text: `"Information. Data you've gathered. Patterns you've noticed.
Routes, contacts, observations. Nothing that hurts you directly.
Everything we collect helps us understand."
A pause.
"And in return, we share what we understand. Fair trade."`,
      choices: [
        {
          text: "Accept the exchange",
          effects: {
            addChronicle: {
              title: "Hollow Circuit",
              text: "Made contact with the Hollow Circuit. Agreed to information exchange.",
            },
            setFlags: {
              hollow_contact: true,
            },
          },
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Hollow Contact",
              text: "Met with the Hollow Circuit. Didn't like the terms.",
            },
          },
        },
      ],
    },
    {
      text: `"Fragments. Patterns. The Cataclysm wasn't random—there was a
sequence, a logic. We're still mapping it. And your past..."
They transmit a data packet. Small. Encrypted.
"A gift. Partial decryption of your salvage yard records. The
rest comes with cooperation."`,
      choices: [
        {
          text: "Accept and cooperate",
          effects: {
            addChronicle: {
              title: "Hollow Circuit",
              text: "Made contact with the Hollow Circuit. Received partial data about my past.",
            },
            setFlags: {
              hollow_contact: true,
              memory_fragment_archived: true,
            },
          },
        },
        {
          text: "Take the gift and leave",
          effects: {
            addChronicle: {
              title: "Hollow Contact",
              text: "Met with the Hollow Circuit. Took what they offered, committed to nothing.",
            },
            setFlags: {
              hollow_contact: true,
            },
          },
        },
      ],
    },
  ],
};