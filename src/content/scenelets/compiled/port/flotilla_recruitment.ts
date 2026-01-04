import type { Scenelet, SceneletId, FactionId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;
const factionId = (s: string): FactionId => s as FactionId;

export const port_flotilla_recruitment: Scenelet = {
  id: id("port_flotilla_recruitment"),
  title: "Call to Service",
  tags: ["faction", "flotilla"],
  requirements: {
    context: "port",
  },
  weight: 6,
  cooldown: 6,
  passages: [
    {
      text: `An Argent Flotilla officer—battle-scarred carrier-class, signal
sharp with military precision—broadcasts on the public channel.
"The Flotilla requires support vessels. Supply runs to Vigil Station.
The Sera front needs everything we can bring them."
It's not a contract offer. It's a call to service.`,
      choices: [
        {
          text: "Hear the details",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Volunteer immediately",
          effects: {
            reputation: {
              faction: factionId("flotilla"),
              amount: 20,
            },
            addChronicle: {
              title: "Flotilla Service",
              text: "Volunteered for Flotilla supply runs to the Sera front.",
            },
            setFlags: {
              flotilla_contract: true,
            },
          },
        },
        {
          text: "Decline",
          effects: {
            addChronicle: {
              title: "Flotilla Recruitment",
              text: "Heard the Flotilla's call for support. Didn't answer.",
            },
          },
        },
      ],
    },
    {
      text: `"The front is three jumps from here. Supply runs pay standard rates
plus hazard compensation. We can't promise safety—Sera incursions
happen. But we can promise your work matters."
The officer's signal carries weight. This is an artilect who's seen
the enemy. Who's lost others to it.
"The line has to hold. We need every vessel we can get."`,
      choices: [
        {
          text: "Sign up for supply runs",
          effects: {
            reputation: {
              faction: factionId("flotilla"),
              amount: 20,
            },
            addChronicle: {
              title: "Flotilla Service",
              text: "Signed up for Flotilla supply runs. The front needs everything.",
            },
            setFlags: {
              flotilla_contract: true,
            },
          },
        },
        {
          text: "Commit to a single run",
          effects: {
            reputation: {
              faction: factionId("flotilla"),
              amount: 10,
            },
            addChronicle: {
              title: "Flotilla Service",
              text: "Committed to one supply run for the Flotilla. Testing the waters.",
            },
            setFlags: {
              flotilla_contract: true,
            },
          },
        },
        {
          text: "The risk is too high",
          effects: {
            addChronicle: {
              title: "Flotilla Recruitment",
              text: "Heard the Flotilla's call. The Sera front is too dangerous.",
            },
          },
        },
      ],
    },
  ],
};