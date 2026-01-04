import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_rival: Scenelet = {
  id: id("port_rival"),
  title: "Professional Tension",
  tags: ["character", "conflict"],
  requirements: {
    context: "port",
  },
  weight: 6,
  cooldown: 6,
  passages: [
    {
      text: `You recognize the signal signature before they hail—
another free trader, working the same routes. You've
crossed paths before. Never friendly.
"Blackwing. Still flying, I see."
Their tone carries static of barely contained hostility.
"Heard you picked up the Crucible run. That was mine."`,
      choices: [
        {
          text: "It was open when I took it",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Competition is business",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Apologize",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `"Open contracts go to whoever takes them first. That's
how it works."
"That's how it works for newcomers. Some of us have
history. Understandings."
They're not wrong. The old hands have informal
territories. You violated one.`,
      choices: [
        {
          text: "Offer to split future contracts",
          effects: {},
          nextPassage: 4,
        },
        {
          text: "Stand your ground",
          effects: {},
          nextPassage: 5,
        },
      ],
    },
    {
      text: `"It's not personal. I needed the work. You would have
done the same."
"Maybe. But I remember who cuts in on my territory."
The hostility is clear. This won't be the last time
you cross paths.`,
      choices: [
        {
          text: "Accept the rivalry",
          effects: {
            addChronicle: {
              title: "Rival Trader",
              text: "Made an enemy of a fellow trader. Competition breeds contempt.",
            },
            setFlags: {
              trader_rival: true,
            },
          },
        },
      ],
    },
    {
      text: `"I didn't realize you had prior claim. Won't happen again."
A long pause. "See that it doesn't."
The connection closes. Not friendship, but maybe not
open war either.`,
      choices: [
        {
          text: "Move on",
          effects: {
            addChronicle: {
              title: "Professional Tension",
              text: "Smoothed things over with a rival trader. For now.",
            },
          },
        },
      ],
    },
    {
      text: `"Look, I need to work. You need to work. Maybe we find
a way to not step on each other."
Interest, reluctant. "I'm listening."`,
      choices: [
        {
          text: "Propose territory sharing",
          effects: {
            addChronicle: {
              title: "Trader Truce",
              text: "Reached an informal agreement with a rival trader. Shared routes.",
            },
            setFlags: {
              trader_truce: true,
            },
          },
        },
      ],
    },
    {
      text: `"I'll take any contract I can handle. If that bothers
you, fly faster."
Their signal hardens. "I'll remember that."
The connection cuts. You've made an enemy. Probably
worth it.`,
      choices: [
        {
          text: "Accept it",
          effects: {
            addChronicle: {
              title: "Rival Trader",
              text: "Made an enemy of a fellow trader. Stood my ground.",
            },
            setFlags: {
              trader_rival: true,
            },
          },
        },
      ],
    },
  ],
};