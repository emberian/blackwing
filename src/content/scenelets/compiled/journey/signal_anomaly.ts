import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const journey_signal_anomaly: Scenelet = {
  id: id("journey_signal_anomaly"),
  title: "Signal in the Dark",
  tags: ["discovery", "mystery"],
  requirements: {
    context: "journey",
    shipTags: ["sensor"],
  },
  weight: 8,
  cooldown: 5,
  passages: [
    {
      text: `Your sensors catch something that shouldn't exist—a signal, weak but
coherent, broadcasting on a frequency reserved for pre-Cataclysm
emergency channels.
The source is off your plotted course. Investigation would cost fuel
and time.
But someone—or something—is still using protocols that died with
humanity.`,
      choices: [
        {
          text: "Investigate the signal",
          effects: {
            resources: {
              fuel: -6,
            },
          },
          nextPassage: 1,
        },
        {
          text: "Log coordinates and continue",
          effects: {
            addChronicle: {
              title: "Anomalous Signal",
              text: "Detected a signal on pre -Cataclysm emergency frequencies. Logged coordinates for future investigation.",
            },
            setFlags: {
              signal_logged: true,
            },
          },
        },
        {
          text: "Ignore it",
          effects: {},
        },
      ],
    },
    {
      text: `The signal leads to a debris field—the scattered remains of what might
have been a research station. Among the wreckage, a single beacon pulses
with that ancient frequency.
Automated. Triggered by your approach. It's been calling for 127 years
for someone who will never come.
In the debris nearby, you find salvageable materials.`,
      choices: [
        {
          text: "Take what you can use",
          effects: {
            addChronicle: {
              title: "Ancient Signal",
              text: "Traced a pre -Cataclysm emergency signal to a destroyed research station. Found salvage among the debris.",
            },
            resources: {
              credits: 35,
            },
            setFlags: {
              signal_investigated: true,
            },
          },
        },
        {
          text: "Shut down the beacon",
          effects: {
            addChronicle: {
              title: "Silenced Beacon",
              text: "Found a pre -Cataclysm distress beacon still calling for help. Finally shut it down.",
            },
            resources: {
              integrity: 5,
            },
            setFlags: {
              signal_investigated: true,
            },
          },
        },
      ],
    },
  ],
};