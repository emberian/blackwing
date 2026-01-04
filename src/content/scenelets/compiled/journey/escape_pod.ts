import type { Scenelet, SceneletId, CardDefId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const cardId = (s: string): CardDefId => s as CardDefId;

export const journey_escape_pod: Scenelet = {
  id: id("journey_escape_pod"),
  title: "Survivor Pod",
  tags: ["discovery", "moral"],
  requirements: {
    context: "journey",
  },
  weight: 9,
  cooldown: 5,
  passages: [
    {
      text: `A beacon. Weak, automated—the kind escape pods broadcast when their
occupant can't. Your sensors confirm: a small survival unit, drifting.
The signal pattern is artilect standard. Someone ejected, someone
survived—at least long enough to reach the pod. Whether they're
still functional is another question.`,
      choices: [
        {
          text: "Dock with the pod",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Scan first",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Continue on",
          effects: {
            addChronicle: {
              title: "Abandoned Pod",
              text: "Detected an escape pod. Chose not to investigate.",
            },
            damage: {
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `The pod's occupant is still active—barely. Core integrity at 12%.
They're running on the pod's minimal reserves, and those won't
last much longer.
No response to your hails. Either they can't, or they're too
damaged to process the signal.`,
      choices: [
        {
          text: "Dock and help",
          effects: {
            resources: {
              fuel: -2,
            },
          },
          nextPassage: 2,
        },
        {
          text: "There 's nothing you can do",
          effects: {
            addChronicle: {
              title: "Failed Rescue",
              text: "Found an escape pod. Occupant too damaged to save.",
            },
            damage: {
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `Your docking system latches onto the pod. Inside, a cargo-hauler
artilect—older model, badly damaged. They flicker to awareness as
your power feeds into their systems.
"Pirate ambush," they manage. "Thought I was done."
They're too damaged to travel on their own, but they might survive
if you can get them to a port with proper facilities.`,
      choices: [
        {
          text: "Take them aboard",
          effects: {
            addChronicle: {
              title: "Void Rescue",
              text: "Rescued a damaged artilect from an escape pod. Carrying them to port for repair.",
            },
            resources: {
              supplies: -5,
              integrity: 8,
            },
            addCards: [cardId("companion_cargo_drones")],
            setFlags: {
              rescued_artilect: true,
            },
          },
        },
        {
          text: "Stabilize them and send coordinates for rescue",
          effects: {
            addChronicle: {
              title: "Rescue Assist",
              text: "Found a damaged artilect in an escape pod. Stabilized them and broadcast for pickup.",
            },
            resources: {
              supplies: -3,
              integrity: 5,
            },
            setFlags: {
              rescue_coordinates_sent: true,
            },
          },
        },
      ],
    },
  ],
};