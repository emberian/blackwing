import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_radiation_storm: Scenelet = {
  id: id("journey_radiation_storm"),
  title: "Solar Event",
  tags: ["danger", "environment"],
  requirements: {
    context: "journey",
  },
  weight: 10,
  cooldown: 4,
  passages: [
    {
      text: `Warning: solar radiation spike detected. A nearby star is
having a bad day—coronal mass ejection, headed your way.
The wavefront will hit in minutes. You can see it on sensors:
a wall of charged particles that will scour unshielded
systems clean.`,
      choices: [
        {
          text: "Find shelter",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Ride it out with reinforced shielding",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Try to outrun it",
          effects: {
            resources: {
              fuel: -12,
            },
          },
          nextPassage: 3,
        },
      ],
    },
    {
      text: `You scan for cover—asteroids, debris, anything to put between
you and the storm. There: a metallic asteroid, dense enough to
block the worst of it.
You tuck into its shadow just as the wavefront hits. Radiation
screams past, but the rock takes the brunt.
When it passes, you emerge. Systems nominal. A little shaken,
but intact.`,
      choices: [
        {
          text: "Resume course",
          effects: {
            addChronicle: {
              title: "Solar Storm",
              text: "Sheltered behind an asteroid during a radiation storm. Emerged intact.",
            },
            resources: {
              fuel: -3,
            },
          },
        },
      ],
    },
    {
      text: `You divert power to hull shielding and ride the wave. The
radiation hammers your systems—sensor static, processing
stutters, the unpleasant sensation of your outer layers
absorbing energy they weren't designed for.
But you hold.
When the storm passes, you run diagnostics. Some minor
degradation, nothing critical. You've weathered worse.`,
      choices: [
        {
          text: "Continue",
          effects: {
            addChronicle: {
              title: "Solar Storm",
              text: "Rode out a radiation storm. Hull absorbed most of the damage.",
            },
            damage: {
              hull: 8,
            },
          },
        },
      ],
    },
    {
      text: `Full burn perpendicular to the wavefront. If you can get
outside the cone of effect before it reaches you...
The math is close. Too close.
The edge of the storm catches you—just a glancing blow,
but enough to scramble your systems for a terrifying moment.
When you stabilize, you're clear. But that was closer than
you'd like.`,
      choices: [
        {
          text: "Don 't do that again",
          effects: {
            addChronicle: {
              title: "Solar Storm",
              text: "Tried to outrun a radiation storm. Almost made it.",
            },
            damage: {
              hull: 4,
              integrity: 4,
            },
          },
        },
      ],
    },
  ],
};