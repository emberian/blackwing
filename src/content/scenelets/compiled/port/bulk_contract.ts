import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_bulk_contract: Scenelet = {
  id: id("port_bulk_contract"),
  title: "Bulk Opportunity",
  tags: ["trade", "contract"],
  requirements: {
    context: "port",
  },
  weight: 8,
  cooldown: 5,
  passages: [
    {
      text: `A shipping consortium approaches—multiple artilects pooling
resources to move large volumes.
"We need capacity. Significant cargo, tight timeline. The
contract pays well, but it's all or nothing. Can you commit?"
They transmit the details. It's a lot of cargo. The deadline
is aggressive. The pay is... substantial.`,
      choices: [
        {
          text: "Review the terms",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Decline immediately",
          effects: {
            addChronicle: {
              title: "Bulk Contract",
              text: "Declined a bulk shipping opportunity. Too much commitment.",
            },
          },
        },
      ],
    },
    {
      text: `Destination: Crucible Station. Cargo: processed materials,
components, manufacturing supplies. Deadline: six cycles.
Payment: three hundred credits, plus performance bonus.
It's good money. But six cycles of your schedule, locked in.
No flexibility. No deviation.`,
      choices: [
        {
          text: "Accept the contract",
          effects: {
            addChronicle: {
              title: "Bulk Contract",
              text: "Accepted a major hauling contract. Six cycles, three hundred credits.",
            },
            setFlags: {
              bulk_contract_accepted: true,
            },
          },
        },
        {
          text: "Negotiate for more",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Bulk Contract",
              text: "Reviewed a bulk contract. Decided against the commitment.",
            },
          },
        },
      ],
    },
    {
      text: `"The timeline is aggressive. Three hundred fifty, or I can't
guarantee delivery."
A pause. The consortium deliberates.
"Three twenty-five. That's our limit."`,
      choices: [
        {
          text: "Accept the improved terms",
          effects: {
            addChronicle: {
              title: "Bulk Contract",
              text: "Negotiated a better rate for a bulk contract. Three twenty -five credits.",
            },
            setFlags: {
              bulk_contract_accepted: true,
            },
          },
        },
        {
          text: "Push for more",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "Walk away",
          effects: {
            addChronicle: {
              title: "Bulk Contract",
              text: "Pushed too hard on a bulk contract. Lost the opportunity.",
            },
          },
        },
      ],
    },
    {
      text: `"Three forty, or find another hauler."
The consortium's response is immediate. "We'll find another
hauler. Thank you for your time."
The connection closes.`,
      choices: [
        {
          text: "Accept the loss",
          effects: {
            addChronicle: {
              title: "Bulk Contract",
              text: "Overplayed my hand on a bulk contract negotiation.",
            },
          },
        },
      ],
    },
  ],
};