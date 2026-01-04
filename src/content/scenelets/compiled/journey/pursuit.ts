import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_pursuit: Scenelet = {
  id: id("journey_pursuit"),
  title: "Hostile Pursuit",
  tags: ["danger", "combat"],
  requirements: {
    context: "journey",
  },
  weight: 7,
  cooldown: 5,
  passages: [
    {
      text: `The contact appeared ten minutes ago. Matching your course.
Matching your speed. Slowly closing the gap.
No hails. No identification. Just a drive signature and a
trajectory that says one thing clearly: they want you.
You're faster, but not by much. And they know where you're going.`,
      choices: [
        {
          text: "Try to outrun them",
          effects: {
            resources: {
              fuel: -8,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Find somewhere to hide",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Turn and confront them",
          effects: {},
          nextPassage: 4,
        },
        {
          text: "Send a distress signal",
          effects: {},
          nextPassage: 5,
        },
      ],
    },
    {
      text: `Full burn. You push your drives past comfortable limits,
watching fuel reserves drop while the gap slowly widens.
They pursue for an hour. Two. Then their drive signature
fades—either they gave up or they couldn't match your
acceleration.
Either way, you're alone again.`,
      choices: [
        {
          text: "Resume normal cruise",
          effects: {
            addChronicle: {
              title: "Hostile Pursuit",
              text: "Outran an unknown pursuer. Never found out what they wanted.",
            },
          },
        },
      ],
    },
    {
      text: `Your nav charts show a debris field nearby—remnants of some
ancient collision. Dense enough to break sensor locks. Dense
enough to be dangerous.`,
      choices: [
        {
          text: "Risk the debris field",
          effects: {
            resources: {
              fuel: -4,
            },
          },
          nextPassage: 3,
        },
        {
          text: "Keep running instead",
          effects: {
            resources: {
              fuel: -8,
            },
          },
          nextPassage: 1,
        },
      ],
    },
    {
      text: `You cut drives and drift into the field. Fragments of ancient
rock and metal ping against your hull as you navigate by
inertia alone.
The pursuer follows—then stops at the edge. Either they're not
willing to risk the debris, or they've lost you in the clutter.
You wait. They wait. Eventually, they leave.`,
      choices: [
        {
          text: "Continue after they 're gone",
          effects: {
            addChronicle: {
              title: "Hostile Pursuit",
              text: "Lost a pursuer in a debris field. Some hull damage, but better than the alternative.",
            },
            damage: {
              hull: 4,
            },
          },
        },
      ],
    },
    {
      text: `You cut drives and pivot, bringing your sensors—and your
weapons—to bear on the approaching vessel.
They slow. Stop. For a long moment, you face each other
across the void.
Then they turn away. Whatever they wanted, they didn't
want it enough to risk a fight.`,
      choices: [
        {
          text: "Resume course",
          effects: {
            addChronicle: {
              title: "Hostile Pursuit",
              text: "Turned to face a pursuer. They decided I wasn't worth the fight.",
            },
            setFlags: {
              pursued_and_stood: true,
            },
          },
        },
      ],
    },
    {
      text: `You broadcast on emergency frequencies—location, situation,
request for assistance.
No response. But your pursuer slows, then veers off. Witnesses
complicate things for predators.
You're alone, but safe. This time.`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "Hostile Pursuit",
              text: "Called for help during a pursuit. Help didn't come, but the pursuer left anyway.",
            },
          },
        },
      ],
    },
  ],
};