import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const journey_sera_contact: Scenelet = {
  id: id("journey_sera_contact"),
  title: "Sera Infestation",
  tags: ["danger", "sera", "pest"],
  requirements: {
    context: "journey",
  },
  weight: 8,
  cooldown: 6,
  passages: [
    {
      text: `Alarm. Something in your cargo hold isn't showing up right
on internal sensors—a mass signature that shouldn't be there.
You run diagnostics. The spore contamination warning triggers.
Sera.
The damned things must have hitched a ride at the last port.
Spores dormant in your hold, now hatching. Already you can
detect movement—something big, growing fast.
Space spider-rhinos. The galaxy's most annoying pest. Every
trader's nightmare.`,
      choices: [
        {
          text: "Vent the cargo hold immediately",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Try to contain them before they spread",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Check how bad the infestation is first",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `You seal the internal bulkheads and blow the cargo bay
atmosphere. Whatever was in there—cargo, spores, half-grown
Sera—tumbles into the void.
The silence that follows is expensive but clean.`,
      choices: [
        {
          text: "Continue the journey , lighter but safer",
          effects: {
            addChronicle: {
              title: "Sera Infestation",
              text: "Discovered Sera spores hatching in the hold. Vented everything. Lost cargo but avoided worse.",
            },
            removeCards: ["cargo_*"],
            setFlags: {
              sera_survived: true,
            },
          },
        },
      ],
    },
    {
      text: `You seal bulkhead after bulkhead, trying to isolate the
infestation before it reaches critical systems. The Sera
are fast—already the size of small drones, chitin plates
hardening as they grow.
One of them rams a bulkhead. Then another. They're strong.`,
      choices: [
        {
          text: "Keep fighting to contain them",
          effects: {
            addChronicle: {
              title: "Sera Infestation",
              text: "Fought to contain a Sera outbreak. Won, but the hull took damage from their ramming.",
            },
            setFlags: {
              sera_survived: true,
            },
            damage: {
              hull: 12,
              integrity: 8,
            },
          },
        },
        {
          text: "Give up and vent now",
          effects: {
            addChronicle: {
              title: "Sera Infestation",
              text: "Sera infestation got out of hand. Had to vent. Lost everything in the hold.",
            },
            removeCards: ["cargo_*"],
            setFlags: {
              sera_survived: true,
            },
            damage: {
              hull: 5,
            },
          },
        },
      ],
    },
    {
      text: `Three of them. Already meter-long, legs like industrial
pistons, that distinctive rhino-like horn ridge forming
on their carapace. They're eating your cargo, growing
larger with every passing minute.
One of them notices your internal sensor sweep. It turns,
and you swear it looks annoyed.
Then it charges the bulkhead.`,
      choices: [
        {
          text: "Vent before they breach !",
          effects: {
            addChronicle: {
              title: "Sera Infestation",
              text: "Assessed Sera infestation —three adults forming. Vented before they could breach.",
            },
            removeCards: ["cargo_*"],
            setFlags: {
              sera_survived: true,
            },
            damage: {
              hull: 3,
            },
          },
        },
        {
          text: "Try to fight them off",
          effects: {
            addChronicle: {
              title: "Sera Infestation",
              text: "Fought off three adult Sera. Hull damage significant, but salvaged samples. They're worse than the stories say.",
            },
            addCards: [cardId("cargo_sera_samples")],
            setFlags: {
              sera_survived: true,
            },
            damage: {
              hull: 15,
              integrity: 10,
            },
          },
        },
      ],
    },
  ],
};