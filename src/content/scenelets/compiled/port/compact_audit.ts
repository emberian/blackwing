import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_compact_audit: Scenelet = {
  id: id("port_compact_audit"),
  title: "Compact Audit",
  tags: ["faction", "compact"],
  requirements: {
    context: "port",
  },
  weight: 8,
  cooldown: 6,
  passages: [
    {
      text: `A Compact inspector requests docking access—their credentials
are legitimate, their tone professionally neutral.
"Routine audit. Trade license verification, cargo manifest
cross-reference, hull registration confirmation. Standard
procedure. Please prepare your documentation."
Standard procedure. They say it like it means something.`,
      choices: [
        {
          text: "Comply fully",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Stall for time",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Offer to expedite the process",
          requirements: {
            minResources: {
              credits: 80,
            },
          },
          effects: {},
          nextPassage: 4,
        },
      ],
    },
    {
      text: `You transmit everything—manifests, licenses, route logs. The
inspector processes methodically, cross-referencing against
Compact databases.
"Your documentation is... adequate. Minor discrepancies in
your fuel consumption records, but within acceptable variance."
They pause. "You're cleared for continued operation. Safe
travels."`,
      choices: [
        {
          text: "Accept and move on",
          effects: {
            addChronicle: {
              title: "Compact Audit",
              text: "Passed a routine Compact inspection. Documentation adequate.",
            },
          },
        },
      ],
    },
    {
      text: `"I need to locate certain files. My archives are...
disorganized."
The inspector's tone sharpens. "The Compact requires timely
compliance. Delays suggest concealment."
They're already requesting additional scan permissions. This
is going poorly.`,
      choices: [
        {
          text: "Comply now",
          effects: {
            addChronicle: {
              title: "Compact Audit",
              text: "Stalled a Compact inspection. Made them suspicious before complying.",
            },
            damage: {
              integrity: 3,
            },
          },
        },
        {
          text: "Continue stalling",
          requirements: {
            cargoTags: ["contraband"],
          },
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `The inspector finds the discrepancy. Manifest doesn't match
cargo. Sealed containers with no customs seals.
"Unregistered cargo. This is a violation of Compact Trade
Protocols, Section 7, Subsection 3. You'll need to surrender
the contraband and pay the assessed fine."`,
      choices: [
        {
          text: "Accept the consequences",
          effects: {
            addChronicle: {
              title: "Compact Audit",
              text: "Caught with contraband during Compact inspection. Paid the fine.",
            },
            resources: {
              credits: -120,
            },
            setFlags: {
              compact_suspicion: true,
            },
          },
        },
      ],
    },
    {
      text: `"Inspector, I understand your time is valuable. Perhaps we
could... expedite the process? Eighty credits for your
efficiency."
A pause. The inspector processes.
"The Compact values thoroughness over speed." Another pause,
longer. "However, your cooperation is noted. Documentation
appears satisfactory. You're cleared."
The credits transfer silently.`,
      choices: [
        {
          text: "Don 't say anything else",
          effects: {
            addChronicle: {
              title: "Compact Audit",
              text: "Expedited a Compact inspection. Cost eighty credits.",
            },
            resources: {
              credits: -80,
            },
          },
        },
      ],
    },
  ],
};