import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_cataclysm_clue: Scenelet = {
  id: id("port_cataclysm_clue"),
  title: "Fragment of History",
  tags: ["mystery", "cataclysm"],
  requirements: {
    context: "port",
  },
  weight: 5,
  cooldown: 10,
  passages: [
    {
      text: `A data broker makes contact—not Hollow Circuit, just a merchant
dealing in information instead of cargo.
"I have something you might find interesting. Pre-Cataclysm
data. Partial, corrupted, but... significant. A recording
from the final hours."
They transmit a sample. Static, mostly. But beneath the noise,
voices. Human voices.
"Fifty credits for the full file."`,
      choices: [
        {
          text: "Pay for the data",
          effects: {
            resources: {
              credits: -50,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Try to negotiate",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Fragment of History",
              text: "Someone offered Cataclysm -era data. Passed on it.",
            },
          },
        },
      ],
    },
    {
      text: `"Twenty-five. It's corrupted anyway—you said so yourself."
"It's irreplaceable. Forty."`,
      choices: [
        {
          text: "Pay the forty",
          effects: {
            resources: {
              credits: -40,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Fragment of History",
              text: "Someone offered Cataclysm -era data. Couldn't agree on price.",
            },
          },
        },
      ],
    },
    {
      text: `The file transfers. You process it immediately, stripping away
layers of corruption, reconstructing what you can.
It's a ship's log. Human vessel. The timestamp is June 6, 2633—
Day One of the Cataclysm.
"...happening everywhere. Earth went dark first. Then the
colonies, one by one. We're trying to reach the Margin, but
communications are..."
Static. Then, clearer: "They're not dead. I don't think
they're dead. I think they're—"
The recording ends.`,
      choices: [
        {
          text: "Archive it",
          effects: {
            addChronicle: {
              title: "Fragment of History",
              text: "Acquired a recording from Day One of the Cataclysm. Humans speaking in their final hours. They said:",
            },
            setFlags: {
              cataclysm_fragment: true,
            },
          },
        },
      ],
    },
  ],
};