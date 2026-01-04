import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_old_trader: Scenelet = {
  id: id("port_old_trader"),
  title: "The Old Hand",
  tags: ["character", "wisdom"],
  requirements: {
    context: "port",
  },
  weight: 8,
  cooldown: 5,
  passages: [
    {
      text: `At the docking authority offices, an ancient cargo hauler waits—
hull scarred by a century of void travel, signal patterns carrying
the weight of accumulated cycles.
"Blackwing, is it?" Their voice carries warmth. Unusual in a
commercial district. "New to the Margin?"`,
      choices: [
        {
          text: "Introduce yourself properly",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Just passing through",
          effects: {
            addChronicle: {
              title: "Old Trader",
              text: "Encountered a veteran trader at port. Exchanged minimal pleasantries.",
            },
          },
        },
      ],
    },
    {
      text: `"I'm Thorngate. Been running these routes since before the
Effortless Expansion. Seen a lot of traders come through.
Most don't last."
They transmit slowly, deliberately.
"The ones who survive? They know when to run, when to fight,
and when to make friends. The void doesn't care about you.
But other artilects might."`,
      choices: [
        {
          text: "Ask for advice",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Thank them and go",
          effects: {
            addChronicle: {
              title: "Old Trader",
              text: "Met Thorngate, a veteran trader. Brief conversation, genuine warmth.",
            },
            resources: {
              integrity: 3,
            },
          },
        },
      ],
    },
    {
      text: `"Keep your integrity high. That's not just your systems—it's
your soul, if we have such things. Lose that, and the void
wins even if you're still flying."
They pause.
"And don't trust anyone who tells you they've got all the
answers. We're all just guessing out here. Some of us have
been guessing longer, is all."`,
      choices: [
        {
          text: "Thank them sincerely",
          effects: {
            addChronicle: {
              title: "Old Trader",
              text: "Received wisdom from Thorngate. Keep your integrity high. Don't trust certainty.",
            },
            resources: {
              integrity: 5,
            },
          },
        },
      ],
    },
  ],
};