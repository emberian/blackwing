import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_debris_field: Scenelet = {
  id: id("journey_debris_field"),
  title: "Navigation Hazard",
  tags: ["danger", "environment"],
  requirements: {
    context: "journey",
  },
  weight: 12,
  cooldown: 4,
  passages: [
    {
      text: `Proximity warnings cascade through your sensors. A debris field—dense, chaotic,
spanning the approach vector. Fragments of rock, metal, and ice tumbling in slow
orbit around nothing at all.
Going through will be faster. Going around will be safer.`,
      choices: [
        {
          text: "Navigate carefully through the field",
          effects: {
            resources: {
              fuel: -3,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Push through at speed",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Plot a course around",
          effects: {
            addChronicle: {
              title: "Debris Field",
              text: "Detected dense debris field on approach vector. Chose the long way around.",
            },
            resources: {
              fuel: -8,
            },
          },
        },
      ],
    },
    {
      text: `You throttle back and let your sensors lead. The Blackwing threads between tumbling
fragments, adjusting vector by millimeters. It's delicate work. Time-consuming.
But you emerge intact.`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "Debris Field",
              text: "Navigated through a dense debris field. Careful work, but successful.",
            },
          },
        },
      ],
    },
    {
      text: `Full burn. The debris becomes a blur of proximity warnings. Something strikes your
hull—a glancing blow, then another. You're through before you can fully assess
the damage.`,
      choices: [
        {
          text: "Assess the damage",
          effects: {
            addChronicle: {
              title: "Debris Field",
              text: "Pushed through a debris field at speed. The hull took hits.",
            },
            damage: {
              hull: 12,
            },
          },
        },
      ],
    },
  ],
};