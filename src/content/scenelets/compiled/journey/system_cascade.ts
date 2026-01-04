import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_system_cascade: Scenelet = {
  id: id("journey_system_cascade"),
  title: "Cascade Failure",
  tags: ["danger", "technical"],
  requirements: {
    context: "journey",
  },
  weight: 10,
  cooldown: 6,
  passages: [
    {
      text: `It starts with a flicker in your navigation subsystem. Then power fluctuations
in the cargo hold. Then—
Warnings flood your consciousness. Multiple systems failing, each failure
triggering another. A cascade, spreading through your infrastructure like
disease through flesh.
You have seconds to decide.`,
      choices: [
        {
          text: "Isolate and restart affected systems",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Emergency shutdown —go dark",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Ride it out , trust your redundancies",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `You partition your consciousness, cutting off the infected systems. It feels like
losing fingers. The cascade slows, stops—but not before it claims your secondary
navigation array.
The restart takes time. You drift in the dark, running on minimal power,
rebuilding yourself piece by piece.`,
      choices: [
        {
          text: "Complete the restart",
          effects: {
            addChronicle: {
              title: "System Cascade",
              text: "Suffered a cascade failure. Lost secondary navigation, but contained the damage.",
            },
            resources: {
              supplies: -8,
            },
          },
        },
      ],
    },
    {
      text: `You go cold. Everything stops—thought, sensation, movement. For an eternal
instant, you are nothing.
Then: restart. Systems coming online one by one, clean, uncorrupted. The cascade
burned itself out against dead circuits.
But you've lost time. And fuel. Your reserves burned to keep core functions
alive during the blackout.`,
      choices: [
        {
          text: "Resume course",
          effects: {
            addChronicle: {
              title: "Emergency Shutdown",
              text: "Initiated emergency shutdown during cascade failure. Lost time and fuel, but systems are clean.",
            },
            resources: {
              fuel: -12,
            },
            damage: {
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `You trust your design. The redundancies hold—barely. The cascade washes through
your systems and ebbs, leaving damage in its wake.
You're still flying. That's something.`,
      choices: [
        {
          text: "Assess the damage",
          effects: {
            addChronicle: {
              title: "System Cascade",
              text: "Rode out a cascade failure. Hull and systems took damage, but we're still operational.",
            },
            resources: {
              supplies: -5,
            },
            damage: {
              hull: 10,
            },
          },
        },
      ],
    },
  ],
};