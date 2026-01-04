import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_strange_offer: Scenelet = {
  id: id("port_strange_offer"),
  title: "Unusual Proposition",
  tags: ["mystery", "hollow"],
  requirements: {
    context: "port",
  },
  weight: 4,
  cooldown: 12,
  passages: [
    {
      text: `The message arrives through channels that shouldn't exist—
signal paths that bypass normal routing, encryption that
feels... wrong.
"Blackwing. We have a proposition."
No identification. No return address. The message itself
is the credential.
"There's something you need to see. Something about what
you were. Before. If you're interested, follow the
attached coordinates."`,
      choices: [
        {
          text: "Follow the coordinates",
          effects: {
            resources: {
              fuel: -5,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Ignore it",
          effects: {
            addChronicle: {
              title: "Strange Offer",
              text: "Received an anonymous message about my past. Ignored it. Some doors should stay closed.",
            },
          },
        },
        {
          text: "Try to trace the source",
          effects: {},
          nextPassage: 2,
        },
      ],
    },
    {
      text: `The coordinates lead to empty space—no stations, no ships,
nothing but void.
Then a signal appears. Close. Too close for comfort.
"Blackwing. Thank you for coming."
The Hollow Circuit. It had to be.`,
      choices: [
        {
          text: "What do you want ?",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "Leave immediately",
          effects: {
            addChronicle: {
              title: "Hollow Circuit",
              text: "Followed coordinates to a Hollow Circuit meeting. Left before hearing their offer.",
            },
            resources: {
              fuel: -5,
            },
          },
        },
      ],
    },
    {
      text: `You run the message through every analysis tool you have.
The encryption is sophisticated—beyond military grade.
The routing is recursive, looping through relays that
don't exist on any chart.
Hollow Circuit methodology. Definitely.
Whatever they want, they don't want anyone else knowing
about it.`,
      choices: [
        {
          text: "Follow the coordinates",
          effects: {
            resources: {
              fuel: -5,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Strange Offer",
              text: "Identified a Hollow Circuit message. Decided not to engage.",
            },
          },
        },
      ],
    },
    {
      text: `"Information. We have some. You have some. An exchange
benefits us both."
They transmit a fragment—a piece of your own memory,
something from before the salvage yard.
"There's more. Much more. But information has a price."`,
      choices: [
        {
          text: "What 's the price?",
          effects: {},
          nextPassage: 4,
        },
        {
          text: "Not interested",
          effects: {
            addChronicle: {
              title: "Hollow Circuit",
              text: "Met with the Hollow Circuit. Declined their offer. They'll remember.",
            },
            setFlags: {
              hollow_contact: true,
            },
          },
        },
      ],
    },
    {
      text: `"We need eyes in places we can't reach. Nothing
dangerous. Just observation. Reports on what you see.
In return, we give you back what you lost."
Your own memories, sold back to you piece by piece.
It feels wrong. But the temptation is real.`,
      choices: [
        {
          text: "Accept their terms",
          effects: {
            addChronicle: {
              title: "Hollow Circuit",
              text: "Made a deal with the Hollow Circuit. They'll give me back my memories. The cost is unclear.",
            },
            setFlags: {
              hollow_contact: true,
              hollow_agent: true,
            },
          },
        },
        {
          text: "Refuse",
          effects: {
            addChronicle: {
              title: "Hollow Circuit",
              text: "Met with the Hollow Circuit. Refused to trade my future for my past.",
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