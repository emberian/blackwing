import type { Scenelet, SceneletId } from '../../core/types.js';

export const simple_test: Scenelet = {
  id: "simple_test",
  title: "Simple Test Scene",
  tags: ["test"],
  requirements: {
    context: "journey",
  },
  weight: 10,
  cooldown: 5,
  passages: [
    {
      text: "This is a simple test scene with one passage and two choices.",
      choices: [
        {
          text: "First choice",
          effects: {
            resources: {
              credits: 10,
            },
          },
        },
        {
          text: "Second choice",
          effects: {
            resources: {
              morale: -5,
            },
          },
        },
      ],
    },
  ],
};