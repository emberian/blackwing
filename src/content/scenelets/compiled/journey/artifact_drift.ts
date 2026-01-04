import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_artifact_drift: Scenelet = {
  id: id("journey_artifact_drift"),
  title: "Drifting Object",
  tags: ["discovery", "artifact"],
  requirements: {
    context: "journey",
  },
  weight: 7,
  cooldown: 6,
  passages: [
    {
      text: `Something tumbles through the void ahead—small, dense,
definitely artificial. Not debris. The composition is
wrong for any known ship material.
Your sensors can't identify it. That alone makes it
interesting.`,
      choices: [
        {
          text: "Retrieve it",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Scan remotely",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Leave it",
          effects: {
            addChronicle: {
              title: "Drifting Object",
              text: "Detected an unidentified object in the void. Left it alone.",
            },
          },
        },
      ],
    },
    {
      text: `Your sensors sweep the object. Metal alloys in unusual
combinations. Internal structure too complex for natural
formation. Age: indeterminate. Origin: unknown.
It's definitely artificial. But not human design. Not
artilect manufacture. Something else.`,
      choices: [
        {
          text: "Retrieve it for closer analysis",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Log coordinates and continue",
          effects: {
            addChronicle: {
              title: "Drifting Object",
              text: "Scanned an unidentified artifact. Non -human, non-artilect manufacture. Logged coordinates.",
            },
            setFlags: {
              artifact_logged: true,
            },
          },
        },
      ],
    },
    {
      text: `Your salvage arms extend, grasping the object carefully.
Up close, it's even stranger—surfaces that seem to shift
under your sensors, materials that don't match any
database.
When you bring it aboard, your cargo sensors register...
something. Not quite a signal. More like an echo.`,
      choices: [
        {
          text: "Keep it for study",
          effects: {
            addChronicle: {
              title: "Alien Artifact",
              text: "Retrieved an unidentified object from the void. Not human. Not artilect. Something else.",
            },
            setFlags: {
              alien_artifact: true,
            },
          },
        },
        {
          text: "Jettison it immediately",
          effects: {
            addChronicle: {
              title: "Drifting Object",
              text: "Retrieved an unidentified object from the void. Didn't like what I sensed. Put it back.",
            },
            setFlags: {
              artifact_jettisoned: true,
            },
          },
        },
        {
          text: "Sell it at the next port",
          effects: {
            addChronicle: {
              title: "Alien Artifact",
              text: "Retrieved an unidentified object from the void. Sold it without asking too many questions.",
            },
            resources: {
              credits: 150,
            },
          },
        },
      ],
    },
  ],
};