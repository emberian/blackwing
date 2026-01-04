import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_void_whispers: Scenelet = {
  id: id("journey_void_whispers"),
  title: "Void Signals",
  tags: ["environment", "mystery"],
  requirements: {
    context: "journey",
  },
  weight: 5,
  cooldown: 7,
  passages: [
    {
      text: `Your receivers catch something in the background radiation—patterns
that almost resolve into meaning. Not a signal, exactly. More like
the ghost of one, echoing through the void.
The patterns feel old. Older than the Cataclysm. Older than humanity's
expansion into the stars.
Or maybe you're hearing things. Maybe the void is just noise, and
meaning is something you're projecting onto chaos.`,
      choices: [
        {
          text: "Listen more carefully",
          effects: {
            damage: {
              integrity: 3,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Run analysis algorithms",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Mark it as sensor noise and move on",
          effects: {
            addChronicle: {
              title: "Void Signals",
              text: "Detected patterns in the background radiation. Marked as noise.",
            },
          },
        },
      ],
    },
    {
      text: `You tune your receivers, filter out everything else, focus on the
whispers. They shift, flow, almost form words—
Then static. The pattern collapses.
You're left with fragments. Coordinates, maybe. Or a warning. Or
nothing at all.`,
      choices: [
        {
          text: "Archive what you heard",
          effects: {
            addChronicle: {
              title: "Void Signals",
              text: "Listened to patterns in the void. Captured fragments. Meaning uncertain.",
            },
            setFlags: {
              void_whispers_heard: true,
            },
          },
        },
      ],
    },
    {
      text: `Your algorithms churn through the data. The results are inconclusive—
the patterns could be natural phenomena, ancient transmissions, or
statistical artifacts.
But there's a frequency match. The patterns share characteristics
with pre-Cataclysm communications protocols.
Coincidence? Or something else?`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "Void Signals",
              text: "Analyzed patterns in the background radiation. Frequency match to pre -Cataclysm protocols. Significance unknown.",
            },
            setFlags: {
              void_whispers_analyzed: true,
            },
          },
        },
      ],
    },
  ],
};