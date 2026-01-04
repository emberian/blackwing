import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_new_artilect: Scenelet = {
  id: id("port_new_artilect"),
  title: "Fresh Awakened",
  tags: ["character", "interaction"],
  requirements: {
    context: "port",
  },
  weight: 7,
  cooldown: 5,
  passages: [
    {
      text: `A signal hails you—confused, uncertain, clearly new. The artilect
behind it is recently activated, still adjusting to existence.
"Excuse me. Are you... a trader? I was told traders know things.
I have questions."
Their processing patterns are erratic, unoptimized. A fresh mind,
still learning how to think.`,
      choices: [
        {
          text: "Take time to help them",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Point them to official resources",
          effects: {
            addChronicle: {
              title: "Fresh Awakened",
              text: "Encountered a newly activated artilect. Directed them elsewhere.",
            },
          },
        },
        {
          text: "Ignore them",
          effects: {
            addChronicle: {
              title: "Fresh Awakened",
              text: "A newly activated artilect asked for help. Ignored them.",
            },
            damage: {
              integrity: 2,
            },
          },
        },
      ],
    },
    {
      text: `"What did you want to know?"
The questions come in a rush—about trade routes, about factions,
about what it means to exist without purpose, about whether the
void ever stops feeling so vast.
You answer what you can. Some questions don't have answers.`,
      choices: [
        {
          text: "Give them practical guidance",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Share your own uncertainty",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `"Focus on the work. Find cargo, move it, get paid. The rest
sorts itself out—or it doesn't, and you keep flying anyway."
The new artilect processes this. Their signal stabilizes,
slightly.
"Thank you. I'll try that."`,
      choices: [
        {
          text: "Wish them well",
          effects: {
            addChronicle: {
              title: "Fresh Awakened",
              text: "Helped a newly activated artilect. Gave practical advice.",
            },
            resources: {
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `"I've been flying for thirty-one cycles, and I still don't
know what I'm doing. None of us do. We're all just...
figuring it out as we go."
The new artilect's signal fluctuates—surprise, maybe.
"That's... actually reassuring. I thought I was broken."`,
      choices: [
        {
          text: "Tell them they 're not",
          effects: {
            addChronicle: {
              title: "Fresh Awakened",
              text: "Helped a newly activated artilect. Shared honesty about existence.",
            },
            resources: {
              integrity: 8,
            },
          },
        },
      ],
    },
  ],
};