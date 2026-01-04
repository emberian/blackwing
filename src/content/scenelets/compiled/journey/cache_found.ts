import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_cache_found: Scenelet = {
  id: id("journey_cache_found"),
  title: "Supply Cache",
  tags: ["discovery", "opportunity"],
  requirements: {
    context: "journey",
  },
  weight: 10,
  cooldown: 5,
  passages: [
    {
      text: `Your sensors flag an anomaly—a mass signature that doesn't match
natural formations. Investigation reveals a hidden cache: containers
secured to an asteroid fragment, beacon dark but intact.
Smuggler drop? Emergency stash? The containers are sealed, unmarked.
Whatever's inside, someone went to trouble to hide it.`,
      choices: [
        {
          text: "Retrieve the cache",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Leave it alone",
          effects: {
            addChronicle: {
              title: "Supply Cache",
              text: "Found a hidden cache in the void. Left it undisturbed.",
            },
          },
        },
        {
          text: "Check for traps first",
          effects: {},
          nextPassage: 1,
        },
      ],
    },
    {
      text: `Careful scans reveal no active threats—no trip sensors, no explosives,
no automated defenses. Either the cache's owner trusted isolation as
security, or they're long gone.`,
      choices: [
        {
          text: "Retrieve the cache",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Still too risky",
          effects: {
            addChronicle: {
              title: "Supply Cache",
              text: "Found a hidden cache. Scans were clean, but still chose not to risk it.",
            },
          },
        },
      ],
    },
    {
      text: `You match velocity with the asteroid fragment and extend your
manipulation arms. The containers come free with minimal resistance.
Inside: supplies, antimatter cells, and a small pouch of credit
chips. Not a fortune, but a solid find.
No one comes to stop you. The cache's owner is either dead,
gone, or doesn't care.`,
      choices: [
        {
          text: "Continue with your prize",
          effects: {
            addChronicle: {
              title: "Cache Found",
              text: "Retrieved a hidden supply cache from an asteroid. Credits, fuel, and supplies gained.",
            },
            resources: {
              supplies: 10,
              fuel: 8,
              credits: 45,
            },
          },
        },
      ],
    },
  ],
};