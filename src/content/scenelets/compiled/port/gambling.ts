import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_gambling: Scenelet = {
  id: id("port_gambling"),
  title: "The Pit",
  tags: ["opportunity", "risk"],
  requirements: {
    context: "port",
  },
  weight: 7,
  cooldown: 4,
  passages: [
    {
      text: `The Pit: a combat simulation arena where artilects wager on
themselves and each other. Not real combat—simulated, consequence-
free except for credits won or lost.
A match is forming. The stakes are modest. The crowd is
enthusiastic.`,
      choices: [
        {
          text: "Watch for a while",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "Enter the competition",
          requirements: {
            minResources: {
              credits: 50,
            },
          },
          effects: {},
          nextPassage: 4,
        },
        {
          text: "Not interested",
          effects: {
            addChronicle: {
              title: "The Pit",
              text: "Visited The Pit. Left without participating.",
            },
          },
        },
      ],
    },
    {
      text: `The combatants circle each other in the simulation space—two
hauler-class artilects, not built for fighting, but the
competition isn't about winning. It's about entertainment.
Community. Something to break the routine.
The crowd cheers. Credits change hands. Someone offers you
a bet.`,
      choices: [
        {
          text: "Place a bet on the underdog",
          effects: {
            resources: {
              credits: -30,
            },
          },
          nextPassage: 2,
        },
        {
          text: "Place a bet on the favorite",
          effects: {
            resources: {
              credits: -30,
            },
          },
          nextPassage: 3,
        },
        {
          text: "Leave",
          effects: {
            addChronicle: {
              title: "The Pit",
              text: "Watched a match at The Pit. Didn't wager.",
            },
          },
        },
      ],
    },
    {
      text: `The underdog fights dirty—unexpected maneuvers, unconventional
tactics. It's not enough to win, but it's close. Closer than
anyone expected.
You lose your bet. The crowd roars anyway.`,
      choices: [
        {
          text: "Accept the loss",
          effects: {
            addChronicle: {
              title: "The Pit",
              text: "Lost a bet at The Pit. The underdog put up a good fight.",
            },
          },
        },
      ],
    },
    {
      text: `The favorite wins, but not easily. Your bet pays out—modest
returns, but satisfying.`,
      choices: [
        {
          text: "Collect your winnings",
          effects: {
            addChronicle: {
              title: "The Pit",
              text: "Won a bet at The Pit. Modest returns.",
            },
            resources: {
              credits: 45,
            },
          },
        },
      ],
    },
    {
      text: `You upload to the simulation space—a simplified combat
environment, you versus an opponent selected at random.
Your opponent is a courier-class, fast and light. You're
bigger, slower, but tougher.`,
      choices: [
        {
          text: "Fight defensively",
          effects: {},
          nextPassage: 5,
        },
        {
          text: "Fight aggressively",
          effects: {},
          nextPassage: 6,
        },
      ],
    },
    {
      text: `You weather the courier's attacks, absorb their speed
advantage, wait for an opening. When it comes, you take it.
Not a dominant victory, but a victory. The crowd cheers.`,
      choices: [
        {
          text: "Collect your winnings",
          effects: {
            addChronicle: {
              title: "The Pit",
              text: "Fought in The Pit. Defensive strategy paid off.",
            },
            resources: {
              credits: 80,
            },
          },
        },
      ],
    },
    {
      text: `You push the offensive, trying to overwhelm the courier
before their speed becomes decisive. It almost works.
Almost.`,
      choices: [
        {
          text: "Accept defeat",
          effects: {
            addChronicle: {
              title: "The Pit",
              text: "Fought in The Pit. Aggressive strategy failed.",
            },
          },
        },
      ],
    },
  ],
};