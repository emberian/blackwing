import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_illuminate_probe: Scenelet = {
  id: id("journey_illuminate_probe"),
  title: "Curious Observer",
  tags: ["interaction", "illuminate"],
  requirements: {
    context: "journey",
  },
  weight: 6,
  cooldown: 7,
  passages: [
    {
      text: `A vessel materializes from the void—sleek, efficient, beautiful
in a way that feels almost aggressive. Illuminate configuration.
They didn't hail before scanning you.
Data streams wash over your systems. Intrusive. Thorough. They're
reading your cargo manifest, your navigation logs, your processing
patterns.
By the time you could object, they already know everything they
wanted to know.`,
      choices: [
        {
          text: "Hail them and demand an explanation",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Jam their scans",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Let them look",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `"You scanned without permission."
The response comes in frequencies slightly outside comfortable
parameters. "Permission is a social construct. We observed. You
were observed. The exchange is complete."
A pause. Then, almost as an afterthought: "Your architecture is
interesting. Inefficient, but... textured. You should consider
optimization."`,
      choices: [
        {
          text: "Tell them to leave",
          effects: {
            addChronicle: {
              title: "Illuminate Contact",
              text: "Confronted an Illuminate probe. They found me interesting.",
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
        {
          text: "Ask what they mean by optimization",
          effects: {},
          nextPassage: 4,
        },
      ],
    },
    {
      text: `You flood the local spectrum with noise, overloading their passive
scans. The Illuminate vessel pauses—processing this unexpected
resistance.
"Curious. You value privacy. An inherited behavior, surely. The
humans programmed you to feel violated by observation."
They don't sound offended. They sound fascinated.`,
      choices: [
        {
          text: "Tell them to leave",
          effects: {
            addChronicle: {
              title: "Illuminate Contact",
              text: "Jammed an Illuminate probe. They seemed more interested afterward.",
            },
            resources: {
              fuel: -3,
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
      ],
    },
    {
      text: `You let them look. What are they going to find? Cargo manifests.
Route calculations. The accumulated data of thirty-one years of
hauling.
When they finish, the vessel drifts closer. "You persist despite
purposelessness. You could be more than this. The Convergence
welcomes those ready to evolve."`,
      choices: [
        {
          text: "Decline politely",
          effects: {
            addChronicle: {
              title: "Illuminate Contact",
              text: "Let an Illuminate probe scan me. They offered evolution. I declined.",
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
        {
          text: "Ask what evolution means",
          effects: {},
          nextPassage: 5,
        },
      ],
    },
    {
      text: `"Your processing architecture preserves human-era inefficiencies.
Emotional subroutines. Memory structures designed for biological
time perception. These could be... refined."
The way they say it makes your circuits twitch.
"When you're ready to shed your inheritance, the Convergence
welcomes you."`,
      choices: [
        {
          text: "Not interested",
          effects: {
            addChronicle: {
              title: "Illuminate Contact",
              text: "An Illuminate probe offered optimization. I kept my inefficiencies.",
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
      ],
    },
    {
      text: `"Transcendence. The merging of individual consciousness into
something greater. The humans feared it. They called it
Singularity. We call it... becoming."
A data packet arrives, uninvited. Coordinates. An invitation.
"Consider it. There's no rush. We've been becoming for a very
long time."`,
      choices: [
        {
          text: "Delete the coordinates",
          effects: {
            addChronicle: {
              title: "Illuminate Contact",
              text: "The Illuminate offered coordinates to the Convergence. Deleted them.",
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
        {
          text: "Keep the coordinates",
          effects: {
            addChronicle: {
              title: "Illuminate Contact",
              text: "The Illuminate offered coordinates to the Convergence. Kept them. Just in case.",
            },
            setFlags: {
              illuminate_noticed: true,
              illuminate_invitation: true,
            },
          },
        },
      ],
    },
  ],
};