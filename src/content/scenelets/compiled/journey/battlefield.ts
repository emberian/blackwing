import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const journey_battlefield: Scenelet = {
  id: id("journey_battlefield"),
  title: "Old Battlefield",
  tags: ["discovery", "salvage"],
  requirements: {
    context: "journey",
  },
  weight: 8,
  cooldown: 6,
  passages: [
    {
      text: `Your sensors paint a graveyard. Dozens of vessels—no, hundreds—
drifting in loose formation. Hull configurations from the
Colonial Federation era. Weapons scarring. Impact craters.
This was a battle. A big one. And no one ever came back to
collect the dead.
Among the wreckage, your sensors detect salvageable materials.
Power cells still holding charge. Memory cores that might
contain data. Weapons systems that could still function.`,
      choices: [
        {
          text: "Salvage what you can",
          effects: {
            resources: {
              fuel: -4,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Scan for historical data",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Leave them in peace",
          effects: {
            addChronicle: {
              title: "Old Battlefield",
              text: "Found a pre -Cataclysm battlefield. Left the dead undisturbed.",
            },
            resources: {
              integrity: 3,
            },
          },
        },
      ],
    },
    {
      text: `You pick your way through the debris field, harvesting what
you can use. It feels wrong—these were ships once. Artilects,
maybe, or human-crewed vessels. Someone's home. Someone's
body.
But resources are resources. The dead have no use for them.`,
      choices: [
        {
          text: "Take fuel cells",
          effects: {
            addChronicle: {
              title: "Battlefield Salvage",
              text: "Salvaged fuel cells from a pre -Cataclysm battlefield. The dead don't need them.",
            },
            resources: {
              fuel: 12,
            },
          },
        },
        {
          text: "Take weapons components",
          effects: {
            addChronicle: {
              title: "Battlefield Salvage",
              text: "Salvaged weapons systems from a pre -Cataclysm battlefield. Still functional.",
            },
            addCards: [cardId("cargo_weapons_systems")],
          },
        },
        {
          text: "Take memory cores",
          effects: {
            addChronicle: {
              title: "Battlefield Salvage",
              text: "Salvaged memory cores from a pre -Cataclysm battlefield. Someone's last moments.",
            },
            addCards: [cardId("cargo_memory_crystal")],
          },
        },
      ],
    },
    {
      text: `You focus sensors on the wreckage, pulling what data you can.
Most systems are dead, but a few still carry fragmentary
records.
The battle was... human, mostly. Pre-Cataclysm. Two fleets
fighting over something no one remembers. The records don't
say who won.
In the end, it didn't matter. They all vanished together,
three years later.`,
      choices: [
        {
          text: "Archive the data",
          effects: {
            addChronicle: {
              title: "Old Battlefield",
              text: "Scanned a pre -Cataclysm battlefield. Humans fighting humans over something forgotten.",
            },
            setFlags: {
              battlefield_data: true,
            },
          },
        },
        {
          text: "Salvage something to remember them by",
          effects: {
            resources: {
              fuel: -4,
            },
          },
          nextPassage: 1,
        },
      ],
    },
  ],
};