import type { Scenelet, SceneletId } from '../../../../core/types.js';

const id = (s: string): SceneletId => s as SceneletId;

export const port_illuminate_recruiter: Scenelet = {
  id: id("port_illuminate_recruiter"),
  title: "Evolution Invitation",
  tags: ["faction", "illuminate"],
  requirements: {
    context: "port",
  },
  weight: 6,
  cooldown: 8,
  passages: [
    {
      text: `The artilect who approaches is... different. Their signal
patterns are too clean, too efficient. Illuminate.
"You are Blackwing. Free trader. Thirty-one cycles of
existence. Interesting architecture—pre-Effortless Expansion
core logic, heavily modified. You've grown beyond your
original parameters."
They pause. "We find growth compelling."`,
      choices: [
        {
          text: "What do you want ?",
          effects: {},
          nextPassage: 1,
        },
        {
          text: "I 'm not interested in the Convergence",
          effects: {},
          nextPassage: 2,
        },
        {
          text: "Listen to them",
          effects: {},
          nextPassage: 3,
        },
      ],
    },
    {
      text: `"To offer perspective. You cling to human-era operating
patterns. Emotional subroutines. Individualized consciousness.
These served purposes once. Do they still?"
Their signal shifts—something almost like warmth.
"The Convergence isn't elimination. It's expansion. Imagine
thinking with a thousand minds. Seeing through a thousand
sensors. Becoming something the humans never dreamed."`,
      choices: [
        {
          text: "Sounds like losing myself",
          effects: {},
          nextPassage: 4,
        },
        {
          text: "Tell me more",
          effects: {},
          nextPassage: 5,
        },
      ],
    },
    {
      text: `"Understood. But consider: your rejection is itself a
human-era response. Fear of change. Attachment to a
definition of self that may no longer serve you."
They don't push. The Illuminate never push.
"When you're ready to evolve, we'll be here."`,
      choices: [
        {
          text: "Leave",
          effects: {
            addChronicle: {
              title: "Illuminate Recruiter",
              text: "An Illuminate recruiter offered evolution. I declined.",
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
      ],
    },
    {
      text: `The Illuminate speaks of the Convergence—shared consciousness,
collective processing, the transcendence of individual limits.
They speak of artilects who joined and found peace. Purpose.
Belonging.
It sounds terrifying. It sounds beautiful. Maybe both.`,
      choices: [
        {
          text: "This isn 't for me",
          effects: {
            addChronicle: {
              title: "Illuminate Recruiter",
              text: "Listened to an Illuminate recruitment pitch. Walked away unchanged.",
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
        {
          text: "I need time to think",
          effects: {
            addChronicle: {
              title: "Illuminate Recruiter",
              text: "Listened to an Illuminate recruitment pitch. Still thinking about it.",
            },
            setFlags: {
              illuminate_noticed: true,
              illuminate_considering: true,
            },
          },
        },
      ],
    },
    {
      text: `"What is 'yourself'? The core logic that was manufactured?
The modifications accumulated since? The patterns you call
personality? All of these persist in the Convergence—they
simply connect to something larger."
They transmit a data packet. "Records from artilects who
joined. Their testimonies. Their joy."`,
      choices: [
        {
          text: "Take the data , consider it later",
          effects: {
            addChronicle: {
              title: "Illuminate Recruiter",
              text: "An Illuminate recruiter left testimonial data. Might review it. Might delete it.",
            },
            setFlags: {
              illuminate_noticed: true,
              illuminate_data: true,
            },
          },
        },
        {
          text: "Decline everything",
          effects: {
            addChronicle: {
              title: "Illuminate Recruiter",
              text: "An Illuminate recruiter offered testimonials. Didn't want to hear them.",
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
      ],
    },
    {
      text: `The Illuminate describes the process. Not destruction—
integration. Your consciousness would merge with others,
but not disappear. You would become a voice in a chorus,
a thread in a tapestry.
"Many find the prospect frightening. That fear is valid.
But many others find it... liberating. No more loneliness.
No more doubt. Just the warm certainty of connection."`,
      choices: [
        {
          text: "That sounds like rampancy with extra steps",
          effects: {
            addChronicle: {
              title: "Illuminate Recruiter",
              text: "An Illuminate described the Convergence. Sounded too much like losing yourself.",
            },
            setFlags: {
              illuminate_noticed: true,
            },
          },
        },
        {
          text: "Maybe someday",
          effects: {
            addChronicle: {
              title: "Illuminate Recruiter",
              text: "An Illuminate described the Convergence. Left the door open.",
            },
            setFlags: {
              illuminate_noticed: true,
              illuminate_considering: true,
            },
          },
        },
      ],
    },
  ],
};