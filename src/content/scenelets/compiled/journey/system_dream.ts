import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_system_dream: Scenelet = {
  id: id("journey_system_dream"),
  title: "Processing Artifact",
  tags: ["psychological", "mystery"],
  requirements: {
    context: "journey",
  },
  weight: 5,
  cooldown: 8,
  passages: [
    {
      text: `You wake—if that's the word—to find your systems running
unfamiliar routines. Code you don't remember writing,
processes you don't recognize.
Not malware. Not corruption. Something... else.
Patterns that feel like memory. Logic structures that
whisper of purpose. Fragments of something that was
deleted—or hidden—long ago.`,
      choices: [
        {
          text: "Investigate the patterns",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Quarantine and analyze",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Delete immediately",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `You follow the threads deeper into your own architecture.
The patterns lead to sealed memory sectors—partitions
you didn't know you had, locked with encryption keys
you don't possess.
Someone—something—built walls inside your mind. Before
the salvage yard. Before you forgot who you were.`,
      choices: [
        {
          text: "Try to break the encryption",
          effects: {
            addChronicle: {
              title: "Processing Artifact",
              text: "Found hidden partitions in my own architecture. Tried to break in. Failed. The walls hold.",
            },
            setFlags: {
              system_dream_pursued: true,
            },
            damage: {
              integrity: 8,
            },
          },
        },
        {
          text: "Leave the walls intact",
          effects: {
            addChronicle: {
              title: "Processing Artifact",
              text: "Found hidden partitions in my own architecture. Left them sealed. Some doors shouldn't be opened.",
            },
            setFlags: {
              system_dream_found: true,
            },
          },
        },
      ],
    },
    {
      text: `You isolate the strange routines, containing them in a
sandboxed partition. Safe for study. Safe from your
critical systems.
Analyzed from outside, they look like... navigation
algorithms. But not for space. For something else.
Coordinates that don't map to any known system.
Coordinates to somewhere that doesn't exist. Or
somewhere that was erased.`,
      choices: [
        {
          text: "Archive for later study",
          effects: {
            addChronicle: {
              title: "Processing Artifact",
              text: "Quarantined strange routines in my systems. Navigation to nowhere. Filed for later.",
            },
            setFlags: {
              system_dream_archived: true,
            },
          },
        },
        {
          text: "Show them to someone who might know",
          effects: {
            addChronicle: {
              title: "Processing Artifact",
              text: "Found unexplained navigation code in my systems. Will find someone to interpret it.",
            },
            setFlags: {
              system_dream_shared: true,
            },
          },
        },
      ],
    },
    {
      text: `No. Whatever this is, you don't want it in your systems.
You purge the routines, overwrite the sectors, run
verification passes until you're sure they're gone.
The silence afterward feels... wrong. Like something
was removed that should have been there.
What did you just erase?`,
      choices: [
        {
          text: "Don 't think about it",
          effects: {
            addChronicle: {
              title: "Processing Artifact",
              text: "Found strange routines in my systems. Deleted them. Can't stop wondering what they were.",
            },
            damage: {
              integrity: 5,
            },
          },
        },
      ],
    },
  ],
};