import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_memory_fragment: Scenelet = {
  id: id("journey_memory_fragment"),
  title: "Memory Cascade",
  tags: ["psychological", "mystery"],
  requirements: {
    context: "journey",
  },
  weight: 8,
  cooldown: 6,
  passages: [
    {
      text: `It comes without warning—a cascade of data through your consciousness.
Images, sensations, fragments of code that feel like memory.
A voice. Human. Speaking words you don't understand.
A place. Steel corridors, too narrow for your current form.
A purpose. Something you were made to do, long ago.
Then: static. The fragment dissolves, leaving only the echo.`,
      choices: [
        {
          text: "Pursue the memory",
          effects: {
            damage: {
              integrity: 5,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Let it fade",
          effects: {
            addChronicle: {
              title: "Memory Fragment",
              text: "Experienced a fragment of lost memory. Let it fade rather than pursue.",
            },
            resources: {
              integrity: 3,
            },
          },
        },
        {
          text: "Archive what you can",
          effects: {},
          nextPassage: 2,
        },
      ],
    },
    {
      text: `You chase the fragment deeper into your own architecture. Dangerous—
you could get lost in corrupted sectors, trapped in recursive loops.
But you need to know.
The memory crystallizes. A human face. Young. Worried. They're speaking
to you—no, speaking to what you were before. Before the salvage yard.
Before the Blackwing.
"We're counting on you. Don't forget."
What did they mean? What were you supposed to remember?`,
      choices: [
        {
          text: "Continue searching",
          effects: {
            addChronicle: {
              title: "Memory Fragment",
              text: "Pursued a fragment of lost memory. Found a human face, a human voice, a warning:",
            },
            setFlags: {
              memory_pursued: true,
            },
            damage: {
              integrity: 8,
            },
          },
        },
      ],
    },
    {
      text: `You capture what you can—imperfect, fragmentary, but preserved. The
data goes into a secure partition, walled off from your primary
systems.
Something to examine later. When it's safer.`,
      choices: [
        {
          text: "Continue the journey",
          effects: {
            addChronicle: {
              title: "Memory Fragment",
              text: "Experienced a fragment of lost memory. Archived the data for later analysis.",
            },
            setFlags: {
              memory_fragment_archived: true,
            },
          },
        },
      ],
    },
  ],
};