import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const journey_derelict: Scenelet = {
  id: id("journey_derelict"),
  title: "Ghost Ship",
  tags: ["discovery", "salvage"],
  requirements: {
    context: "journey",
  },
  weight: 9,
  cooldown: 5,
  passages: [
    {
      text: `Your sensors paint a vessel drifting in the void. No drive signature. No
running lights. No response to standard hails.
The hull configuration is old—pre-Cataclysm old. A human-era design,
abandoned here for over a century.
Ghost ships happen. But they never stop being unsettling.`,
      choices: [
        {
          text: "Board and investigate",
          effects: {
            resources: {
              fuel: -3,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Scan remotely and move on",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Leave it alone",
          effects: {
            addChronicle: {
              title: "Ghost Ship",
              text: "Detected a pre -Cataclysm derelict. Chose not to investigate.",
            },
          },
        },
      ],
    },
    {
      text: `Your docking systems interface with the ancient vessel. The seals are
corroded but hold. Inside: darkness, silence, the smell of long-dead
atmosphere.
The crew quarters are empty. Whatever happened here, it happened fast.
In the engineering section, you find a functional memory core. Old
data—personnel records, navigation logs, final communications.`,
      choices: [
        {
          text: "Take the memory core",
          effects: {
            addChronicle: {
              title: "Derelict Salvage",
              text: "Boarded a Cataclysm -era derelict. Salvaged a memory core from engineering.",
            },
            addCards: [cardId("cargo_memory_crystal")],
            setFlags: {
              derelict_salvaged: true,
            },
          },
        },
        {
          text: "Leave everything as it was",
          effects: {
            addChronicle: {
              title: "Ghost Ship",
              text: "Boarded a Cataclysm -era derelict. Left the remains undisturbed.",
            },
            resources: {
              integrity: 3,
            },
          },
        },
      ],
    },
    {
      text: `Your sensors sweep the derelict. Hull integrity: 23%. Power reserves: depleted.
Life signs: none—but that's expected.
Wait. There's something in the cargo bay. Low-level EM signature. Could be
salvageable equipment. Could be something else.`,
      choices: [
        {
          text: "Closer inspection isn 't worth the risk",
          effects: {
            addChronicle: {
              title: "Ghost Ship",
              text: "Scanned a pre -Cataclysm derelict. Something was active in the cargo bay. Didn't investigate.",
            },
          },
        },
        {
          text: "Board to investigate the signature",
          effects: {
            resources: {
              fuel: -3,
            },
          },
          nextPassage: 1,
        },
      ],
    },
  ],
};