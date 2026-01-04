import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_nebula: Scenelet = {
  id: id("journey_nebula"),
  title: "Nebula Transit",
  tags: ["environment", "beauty"],
  requirements: {
    context: "journey",
  },
  weight: 10,
  cooldown: 4,
  passages: [
    {
      text: `The jump deposits you inside a stellar nursery. Clouds of gas and
dust stretch for light-years in every direction, lit from within by
infant stars. The colors shift as you move—violet to gold to deep
crimson.
It's beautiful. Even for a consciousness that processes beauty as
data, it's undeniably beautiful.
Sensors are degraded by the ionization. Navigation is slower. But
rushing through seems... wasteful, somehow.`,
      choices: [
        {
          text: "Take time to observe",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Process the data efficiently and move on",
          effects: {
            addChronicle: {
              title: "Nebula Transit",
              text: "Passed through a stellar nursery. Processed the data, moved on.",
            },
          },
        },
        {
          text: "Use the sensor interference to run dark",
          effects: {},
          nextPassage: 2,
        },
      ],
    },
    {
      text: `You drift. Let the colors wash over your sensors. Run pattern
recognition on cloud formations, predict stellar evolution,
catalog the spectrum data.
It serves no purpose. It feels right anyway.
When you finally resume course, something has shifted in your
processing. A lightness that wasn't there before.`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "Nebula Transit",
              text: "Took time to observe a stellar nursery. Beauty serves no purpose. It felt right anyway.",
            },
            resources: {
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `The ionization plays havoc with active sensors—but that cuts both
ways. You power down to minimal emissions and drift through the
nebula like a ghost.
If anyone's watching, they won't see you.`,
      choices: [
        {
          text: "Continue unseen",
          effects: {
            addChronicle: {
              title: "Nebula Transit",
              text: "Used a stellar nursery's interference for stealth transit.",
            },
            resources: {
              fuel: -3,
            },
          },
        },
      ],
    },
  ],
};