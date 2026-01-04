import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const port_human_artifact: Scenelet = {
  id: id("port_human_artifact"),
  title: "Human Artifact",
  tags: ["mystery", "artifact"],
  requirements: {
    context: "port",
  },
  weight: 6,
  cooldown: 6,
  passages: [
    {
      text: `A vendor on the secondary markets offers something unusual—a
genuine pre-Cataclysm artifact. Human-made, human-touched,
authenticated by Remnant certifiers.
"Book," they transmit. "Physical, paper. Printed words. The
makers used these to store information before digital became
standard. This one's from Earth itself."
A book. An object humans held in their hands, their eyes
scanning the pages. A window into what they were.`,
      choices: [
        {
          text: "Examine the artifact",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Ask the price",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Not interested",
          effects: {
            addChronicle: {
              title: "Human Artifact",
              text: "Saw a human artifact for sale. A book. Didn't pursue.",
            },
          },
        },
      ],
    },
    {
      text: `The vendor permits close scanning. The book is old—ancient by
any reasonable standard. The paper is yellowed, the binding
cracked. But the words are still legible.
It's a story. Fiction. Tales of humans doing human things in
human places. The details are alien and achingly familiar at
the same time.`,
      choices: [
        {
          text: "Ask the price",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Return it respectfully",
          effects: {
            addChronicle: {
              title: "Human Artifact",
              text: "Examined a human book. Chose not to purchase. The glimpse was enough.",
            },
            resources: {
              integrity: 3,
            },
            setFlags: {
              human_artifact_found: true,
            },
          },
        },
      ],
    },
    {
      text: `"Two hundred credits. The Remnant would pay more, but they're
not here and you are."
That's expensive. But there's something compelling about the
object—about holding something the makers held.`,
      choices: [
        {
          text: "Purchase the book",
          requirements: {
            minResources: {
              credits: 200,
            },
          },
          effects: {
            addChronicle: {
              title: "Human Artifact",
              text: "Purchased a pre -Cataclysm book. An object the makers touched.",
            },
            resources: {
              credits: -200,
            },
            addCards: [cardId("cargo_human_artifacts")],
            setFlags: {
              human_artifact_found: true,
            },
          },
        },
        {
          text: "Negotiate",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Human Artifact",
              text: "Considered purchasing a human artifact. Price was too high.",
            },
          },
        },
      ],
    },
    {
      text: `"One fifty."
The vendor considers. "One seventy-five. These don't come
around often."`,
      choices: [
        {
          text: "Accept",
          effects: {
            addChronicle: {
              title: "Human Artifact",
              text: "Purchased a pre -Cataclysm book at a negotiated price.",
            },
            resources: {
              credits: -175,
            },
            addCards: [cardId("cargo_human_artifacts")],
            setFlags: {
              human_artifact_found: true,
            },
          },
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Human Artifact",
              text: "Couldn't reach terms on a human artifact.",
            },
          },
        },
      ],
    },
  ],
};