import type { Scenelet, SceneletId } from '../../core/types.js';

export const branching_test: Scenelet = {
  id: "branching_test",
  title: "Branching Test Scene",
  tags: ["test", "branching"],
  requirements: {
    context: "port",
    shipTags: ["sensor"],
    minResources: {
      credits: 50,
    },
  },
  weight: 12,
  cooldown: 3,
  passages: [
    {
      text: `A branching scene with multiple passages and complex effects.
You stand at a crossroads.`,
      choices: [
        {
          text: "Go left",
          requirements: {
            crewTags: ["engineering"],
          },
          effects: {
            resources: {
              fuel: -5,
            },
            setFlags: {
              went_left: true,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Go right",
          effects: {
            setFlags: {
              went_right: true,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Stay put",
          effects: {
            addChronicle: {
              title: "Indecision",
              text: "Couldn't make up our minds. Time wasted.",
            },
            resources: {
              morale: -3,
            },
          },
        },
      ],
    },
    {
      text: "The left path leads to a workshop.",
      choices: [
        {
          text: "Search the workshop",
          effects: {
            addChronicle: {
              title: "Workshop Discovery",
              text: "Found some useful salvage in an abandoned workshop.",
            },
            resources: {
              credits: 25,
            },
            addCards: ["cargo_salvage"],
          },
        },
        {
          text: "Leave quickly",
          effects: {},
        },
      ],
    },
    {
      text: "The right path leads to a marketplace.",
      choices: [
        {
          text: "Trade goods",
          effects: {
            resources: {
              credits: 50,
              supplies: -5,
            },
          },
        },
        {
          text: "Ask for information",
          effects: {
            resources: {
              credits: -10,
            },
            setFlags: {
              got_info: true,
            },
          },
        },
      ],
    },
  ],
};