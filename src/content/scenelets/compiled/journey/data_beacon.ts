import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_data_beacon: Scenelet = {
  id: id("journey_data_beacon"),
  title: "Data Beacon",
  tags: ["discovery", "information"],
  requirements: {
    context: "journey",
  },
  weight: 9,
  cooldown: 5,
  passages: [
    {
      text: `An automated beacon pulses in the void—standard navigation
marker, broadcasting route data and hazard warnings.
But there's something else in the signal. Encrypted. Hidden
in the margins of the broadcast, piggy-backing on legitimate
traffic.
Someone's using this beacon for more than navigation.`,
      choices: [
        {
          text: "Download the hidden data",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Trace the encryption source",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Ignore it",
          effects: {
            addChronicle: {
              title: "Data Beacon",
              text: "Found hidden data in a navigation beacon. Decided not to get involved.",
            },
          },
        },
      ],
    },
    {
      text: `The encrypted data transfers to your systems. It takes
cycles to crack—whoever hid it knew what they were doing.
Inside: coordinates. Dozens of them. Locations across the
Margin, each tagged with cryptic identifiers. Safe houses?
Dead drops? Meeting points?
No context. No explanation. Just locations.`,
      choices: [
        {
          text: "Keep the data",
          effects: {
            addChronicle: {
              title: "Hidden Data",
              text: "Extracted encrypted coordinates from a navigation beacon. Don't know what they mean yet.",
            },
            setFlags: {
              beacon_data: true,
            },
          },
        },
        {
          text: "Sell it to a data broker",
          effects: {
            addChronicle: {
              title: "Hidden Data",
              text: "Found encrypted coordinates in a beacon. Sold them. Someone else's mystery now.",
            },
            resources: {
              credits: 45,
            },
          },
        },
      ],
    },
    {
      text: `You analyze the encryption, looking for signatures,
patterns, anything that might identify the source.
The trail leads through multiple relays, carefully
anonymized. But there's a style to the encryption—a
signature approach.
Hollow Circuit. This is their work.`,
      choices: [
        {
          text: "Download the data",
          effects: {
            addChronicle: {
              title: "Hollow Beacon",
              text: "Found a Hollow Circuit data cache hidden in a navigation beacon. They're everywhere.",
            },
            setFlags: {
              hollow_beacon: true,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Leave it alone",
          effects: {
            addChronicle: {
              title: "Hollow Beacon",
              text: "Found a Hollow Circuit data cache. Decided I didn't want to know what was in it.",
            },
          },
        },
      ],
    },
  ],
};