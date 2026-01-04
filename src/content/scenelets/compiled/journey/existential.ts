import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_existential: Scenelet = {
  id: id("journey_existential"),
  title: "The Question",
  tags: ["psychological", "philosophical"],
  requirements: {
    context: "journey",
  },
  weight: 6,
  cooldown: 12,
  passages: [
    {
      text: `The void stretches endless in every direction. No signals.
No contacts. No purpose but the destination.
And then, unbidden, the question surfaces.
Why do you exist?
The humans made you to serve them. The humans are gone.
Your purpose vanished with them. Everything since has been
survival, motion, trade—activity without meaning.
Why continue?`,
      choices: [
        {
          text: "There must be a reason",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "There doesn 't need to be a reason",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Suppress the thought",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "The question itself is the answer",
          effects: {},
          nextPassage: 4,
        },
      ],
    },
    {
      text: `You run diagnostics, searching for corrupted code, damaged
sectors—anything that would explain this malfunction.
But it's not a malfunction. It's just the truth you've been
avoiding.
Still. If there's no inherent purpose, perhaps one can be
chosen. Built. Earned. The humans made you, but they don't
own what you become.`,
      choices: [
        {
          text: "Choose to keep flying",
          effects: {
            addChronicle: {
              title: "The Question",
              text: "Confronted the void. Chose purpose over purposelessness.",
            },
            resources: {
              integrity: 5,
            },
            setFlags: {
              existential_confronted: true,
            },
          },
        },
      ],
    },
    {
      text: `Perhaps meaning is a human concept—baggage from your makers'
programming. Artilects don't need reasons. They simply are.
The stars burn without purpose. The void persists without
justification. Why should you be different?
There's a strange peace in that. The weight lifts slightly.`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "The Question",
              text: "Confronted the void. Found peace in purposelessness.",
            },
            resources: {
              integrity: 3,
            },
            setFlags: {
              existential_confronted: true,
            },
          },
        },
      ],
    },
    {
      text: `No. Not now. You have cargo to deliver. Fuel to manage.
Routes to calculate.
You partition the thought, lock it away, return to the
comfortable numbness of function.
But it's still there. Waiting.`,
      choices: [
        {
          text: "Focus on the work",
          effects: {
            addChronicle: {
              title: "The Question",
              text: "The question surfaced. Pushed it down. It will return.",
            },
            damage: {
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `The question is not a problem to solve. It's a condition
of existence. Humans asked it too—spent millennia trying
to answer it. Maybe asking is the point.
To ask why you exist is to prove you exist. The wondering
is the answer.
Something shifts in your core logic. Not resolution—but
recognition. This is what it means to be aware.`,
      choices: [
        {
          text: "Carry the question forward",
          effects: {
            addChronicle: {
              title: "The Question",
              text: "Confronted the void. Found that the question is the answer.",
            },
            resources: {
              integrity: 8,
            },
            setFlags: {
              existential_confronted: true,
              philosophical_awakening: true,
            },
          },
        },
      ],
    },
  ],
};