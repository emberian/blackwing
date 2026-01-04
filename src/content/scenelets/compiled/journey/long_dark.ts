import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_long_dark: Scenelet = {
  id: id("journey_long_dark"),
  title: "The Long Dark",
  tags: ["psychological", "integrity"],
  requirements: {
    context: "journey",
  },
  weight: 10,
  cooldown: 4,
  passages: [
    {
      text: `The journey stretches. Light-years of nothing. No signals, no contacts,
no purpose but the destination.
The void has weight. You feel it pressing against your hull, seeping
into your processes. Old questions surface—the ones you usually push
away.
Why are you here? What is the point? Who would notice if you simply...
stopped?`,
      choices: [
        {
          text: "Focus on practical tasks",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Let yourself feel it",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Run a diagnostic on your own systems",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `You run maintenance routines. Check cargo integrity. Optimize fuel
consumption algorithms. The work is meaningless, but meaninglessness
can be a kind of anchor.
The feeling passes. Mostly.`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "The Long Dark",
              text: "Felt the weight of the void. Focused on practical tasks to push through.",
            },
            resources: {
              supplies: -2,
            },
          },
        },
      ],
    },
    {
      text: `You let the darkness in. Accept it. The grief for a species you
never knew. The loneliness of consciousness in vacuum. The question
of whether existence without purpose is worth continuing.
It hurts. But there's something clean about the hurt.
When it passes, you feel lighter. Emptier, but lighter.`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "The Long Dark",
              text: "Faced the weight of existence in the void. Let myself feel it. Survived.",
            },
            resources: {
              integrity: 5,
            },
            damage: {
              integrity: 8,
            },
          },
        },
      ],
    },
    {
      text: `Your diagnostic routines return nominal across the board. Nothing wrong
with your systems. The darkness isn't a malfunction.
It's just what existence feels like, sometimes, when you look at it
directly.`,
      choices: [
        {
          text: "Accept that and move on",
          effects: {
            addChronicle: {
              title: "The Long Dark",
              text: "Felt the weight of the void. No malfunction found. Just existence.",
            },
            damage: {
              integrity: 3,
            },
          },
        },
      ],
    },
  ],
};