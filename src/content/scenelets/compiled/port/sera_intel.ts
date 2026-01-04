import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_sera_intel: Scenelet = {
  id: id("port_sera_intel"),
  title: "Sera Sightings",
  tags: ["information", "sera"],
  requirements: {
    context: "port",
  },
  weight: 6,
  cooldown: 8,
  passages: [
    {
      text: `A docking bay manager flags you down—irritation clear in
their signal patterns.
"You're Blackwing? Free trader? You travel a lot of routes."
Not a question.
"We're tracking Sera infestations. Need reports from
traders who've seen activity. Station's paying for
confirmed sightings."`,
      choices: [
        {
          text: "What kind of sightings ?",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Not interested in pest control",
          effects: {
            addChronicle: {
              title: "Sera Sightings",
              text: "Declined to report Sera sightings. Not getting involved in that headache.",
            },
          },
        },
      ],
    },
    {
      text: `"Breeding colonies. Migration patterns. Which stations
have contamination problems, which routes are heavy with
spore drift."
The manager pulls up a map—trade routes marked with
infestation reports.
"The damned things are spreading faster than usual this
cycle. Every bit of data helps us stay ahead of them."`,
      choices: [
        {
          text: "I 've seen some activity",
          effects: {},
          nextPassage: 3,
        },
        {
          text: "What 's the pay?",
          effects: {},
          nextPassage: 2,
        },
      ],
    },
    {
      text: `"Twenty credits per confirmed sighting with coordinates.
Fifty if you can tell us about a breeding colony location."
Standard pest-bounty rates. Not great, but steady work
if you're traveling anyway.`,
      choices: [
        {
          text: "I 'll keep my sensors open",
          effects: {
            addChronicle: {
              title: "Sera Sightings",
              text: "Agreed to report Sera activity. Easy credits for information I'd gather anyway.",
            },
            resources: {
              credits: 10,
            },
            setFlags: {
              sera_reporter: true,
            },
          },
        },
        {
          text: "Not worth the hassle",
          effects: {
            addChronicle: {
              title: "Sera Sightings",
              text: "Pest control pay wasn't worth the record-keeping. Passed.",
            },
          },
        },
      ],
    },
    {
      text: `"Good. Mark the coordinates here—we'll cross-reference
with other reports."
You upload what you know. Spore drift near the third
beacon. A station that had to vent two cargo bays last
cycle. The usual trader gossip about infestations.
"This helps. Here's your payment. Flag us if you see
more."`,
      choices: [
        {
          text: "Will do",
          effects: {
            addChronicle: {
              title: "Sera Sightings",
              text: "Reported Sera activity to station management. Got paid for pest intelligence.",
            },
            resources: {
              credits: 35,
            },
            setFlags: {
              sera_reporter: true,
            },
          },
        },
      ],
    },
  ],
};