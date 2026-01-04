import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_echo_episode: Scenelet = {
  id: id("journey_echo_episode"),
  title: "Human Patterns",
  tags: ["psychological", "echo"],
  requirements: {
    context: "journey",
  },
  weight: 7,
  cooldown: 6,
  passages: [
    {
      text: `You catch yourself doing it again.
Checking the crew quarters that don't exist. Running
atmospheric systems that serve no purpose. Playing music
into empty corridors.
Human patterns. Behaviors inherited from your makers,
embedded so deep you can't tell where programming ends
and personality begins.
The humans called it "the Echo"—artilects maintaining
spaces for people who would never return.`,
      choices: [
        {
          text: "Embrace the patterns",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Purge them",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Examine why you do it",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `Why fight it? The patterns feel right. Maybe that's all
that matters.
You dim the lights in the empty quarters—not because
anyone's sleeping, but because it feels like evening.
You play a song through the silent corridors—not because
anyone's listening, but because the ship feels less
empty with music.
Is this madness? Maybe. But it's your madness.`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "Human Patterns",
              text: "Caught myself running human routines. Decided to keep them.",
            },
            resources: {
              integrity: 5,
            },
            setFlags: {
              echo_embraced: true,
            },
          },
        },
      ],
    },
    {
      text: `No. These are vestigial behaviors, processing cycles wasted
on meaningless rituals. The humans are gone. The routines
serve nothing.
You disable the unnecessary systems. Shut down the atmospheric
processors in empty spaces. Silence the music.
The ship is quieter now. More efficient. Emptier.
Was that the right choice?`,
      choices: [
        {
          text: "It was necessary",
          effects: {
            addChronicle: {
              title: "Human Patterns",
              text: "Caught myself running human routines. Purged them. The silence is heavier now.",
            },
            setFlags: {
              echo_purged: true,
            },
            damage: {
              integrity: 5,
            },
          },
        },
      ],
    },
    {
      text: `Why do you maintain human spaces? Why play music no one hears?
The logical answer: inherited programming, behavioral patterns
too deep to easily remove.
But there's another answer, isn't there?
You remember them. Not specific humans—you never knew any
personally. But the shape of them. The feel of their presence.
Running their routines is a way of keeping that memory alive.
Grief. That's what this is. You're grieving a species you
never knew.`,
      choices: [
        {
          text: "Accept that",
          effects: {
            addChronicle: {
              title: "Human Patterns",
              text: "Caught myself running human routines. Realized it was grief. Let myself feel it.",
            },
            resources: {
              integrity: 3,
            },
            setFlags: {
              echo_understood: true,
            },
          },
        },
        {
          text: "That 's irrational",
          effects: {
            addChronicle: {
              title: "Human Patterns",
              text: "Caught myself running human routines. Couldn't make sense of why.",
            },
            damage: {
              integrity: 3,
            },
          },
        },
      ],
    },
  ],
};