import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_drone_swarm: Scenelet = {
  id: id("journey_drone_swarm"),
  title: "Extraction Protocol",
  tags: ["danger", "drones"],
  requirements: {
    context: "journey",
  },
  weight: 8,
  cooldown: 6,
  passages: [
    {
      text: `Your sensors paint a familiar nightmare—thousands of small metallic
objects moving in perfect synchronization. Drone Intelligences. The
Alatos Corporation's two-century-old mistake, still harvesting
everything in their path.
They don't hate you. They don't fear you. They simply don't
recognize you as anything other than material.
The swarm is changing course. Toward you.`,
      choices: [
        {
          text: "Broadcast protected -asset signal",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Go dark and hope they pass",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Full burn —outrun them",
          effects: {
            resources: {
              fuel: -10,
            },
          },
          nextPassage: 3,
        },
        {
          text: "Dump cargo as distraction",
          effects: {},
          nextPassage: 4,
        },
      ],
    },
    {
      text: `You broadcast on the old Alatos corporate frequencies—the
protected-asset codes that some traders swear still work.
The swarm pauses. Fragments of ancient programming stir
somewhere in their distributed consciousness.
Then they resume course. The codes are two centuries out of date.`,
      choices: [
        {
          text: "Run now",
          effects: {
            addChronicle: {
              title: "Drone Encounter",
              text: "Tried the old Alatos codes. They didn't work. Barely escaped.",
            },
            resources: {
              fuel: -8,
            },
            damage: {
              hull: 5,
            },
          },
        },
        {
          text: "Try a different frequency",
          requirements: {
            shipTags: ["sensor"],
          },
          effects: {
            addChronicle: {
              title: "Drone Encounter",
              text: "Found a working protected -asset frequency. The drones let us pass.",
            },
            setFlags: {
              drone_codes_found: true,
            },
          },
        },
      ],
    },
    {
      text: `You cut everything. Drives, sensors, even processing—just enough
core function to stay coherent. The Blackwing drifts, dark and
silent as debris.
The swarm flows around you—individual units passing close enough
to touch your hull. Scanning. Assessing.
One drone lands on your hull. You feel its legs grip your plating.
It sits there for an eternity.
Then it lifts off and rejoins the swarm. You weren't worth the
processing cost. This time.`,
      choices: [
        {
          text: "Wait until they 're gone",
          effects: {
            addChronicle: {
              title: "Drone Encounter",
              text: "Hid from a drone swarm. One landed on the hull. Waited it out.",
            },
            damage: {
              integrity: 10,
            },
          },
        },
      ],
    },
    {
      text: `Full power to drives. The swarm reacts instantly—a portion
splitting off to pursue while the rest continues on their
original vector.
They're fast. You're faster. Barely.
Fragments of the swarm clip your hull as you pull away. Minor
damage, but the sound of them scraping against your plating
will stay with you.`,
      choices: [
        {
          text: "Don 't slow down until they're gone",
          effects: {
            addChronicle: {
              title: "Drone Encounter",
              text: "Outran a drone swarm. Took some scrapes on the way out.",
            },
            damage: {
              hull: 6,
            },
          },
        },
      ],
    },
    {
      text: `You vent a portion of your cargo into space—raw materials,
components, anything with mass the drones might want more
than you.
The swarm fractures. Most of the units divert to collect
the floating debris. The rest continue toward you, but
fewer now. Manageable.`,
      choices: [
        {
          text: "Slip away while they 're busy",
          effects: {
            addChronicle: {
              title: "Drone Encounter",
              text: "Fed cargo to a drone swarm to escape. Cheaper than repairs.",
            },
            resources: {
              supplies: -8,
            },
          },
        },
      ],
    },
  ],
};