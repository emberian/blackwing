# Holdsmith Usage Guide

Holdsmith is a DSL (Domain-Specific Language) toolkit for authoring narrative scenes. It compiles `.scene` files into TypeScript code compatible with Cargo Hold's `Scenelet` type system.

## Quick Start

```bash
cd tools/holdsmith
npm install
npm run build:cli

# Compile all .scene files in a directory
node dist/cli/index.js compile path/to/scenes/ -o output/

# Parse a single file and view AST
node dist/cli/index.js parse path/to/scene.scene

# Validate without generating output
node dist/cli/index.js validate path/to/scenes/
```

## File Format

Scene files use a YAML frontmatter + Ink-inspired body syntax.

### Complete Example

```
---
id: journey_strange_signal
title: Signal in the Dark
tags: [mystery, discovery, prior]
context: journey
weight: 10
cooldown: 5
requires:
  shipTags: [sensor]
  minResources:
    credits: 50
---

=== intro

The sensor array chirps—an anomaly in the void. A signal, repeating in patterns 
that feel almost like language. Old. Pre-Silence old.

It's coming from somewhere off your plotted course.

* [Investigate the signal]
  ~ fuel -= 5
  ~ flag signal_investigated
  -> derelict

* [Log coordinates and continue]
  ~ flag signal_logged
  ~ chronicle "Signal Logged"
    Detected an anomalous signal. Logged coordinates for future investigation.
  -> END

* [Ignore it]
  -> END

=== derelict

You find a derelict, drifting in the void. In the cargo bay, a data core glows.

* [Take the data core]
  ~ addCard cargo_memory_cores
  ~ flag found_derelict_core
  ~ chronicle "The Derelict's Secret"
    Salvaged a memory core from a Silence-era derelict.
  -> END

* [Leave them undisturbed]
  ~ morale += 5
  -> END
```

## Frontmatter Reference

The YAML frontmatter defines scene metadata:

```yaml
---
id: unique_scene_id          # Required. Must be unique across all scenes.
title: Human Readable Title   # Required. Displayed to player.
tags: [tag1, tag2]           # Optional. For categorization and filtering.
context: journey             # Required. One of: journey, port, any
weight: 10                   # Optional. Higher = more likely to appear. Default: 10
cooldown: 5                  # Optional. Cycles before scene can repeat. Default: 5
requires:                    # Optional. Conditions for scene to appear.
  shipTags: [sensor]
  crewTags: [engineering]
  cargoTags: [volatile]
  minResources:
    credits: 50
    fuel: 20
  maxResources:
    morale: 60
  requiredFlags: [met_collector]
  excludedFlags: [refused_twice]
---
```

### Context Values

| Context | When Scene Can Appear |
|---------|----------------------|
| `journey` | During travel between ports |
| `port` | While docked at a port |
| `any` | Either context |

### Resources

The five resources that can be checked or modified:

| Resource | Description |
|----------|-------------|
| `credits` | Currency |
| `fuel` | Travel capacity |
| `supplies` | Crew sustenance |
| `hull` | Ship integrity (0 = game over) |
| `morale` | Crew mental state (0 = game over) |

## Passage Syntax

### Passage Headers

```
=== passage_name
```

Passages are named sections. The first passage is the entry point. Names must be alphanumeric with underscores.

### Prose

Any text after a passage header (until a choice or another passage) is prose:

```
=== intro

This is prose. Multiple lines become a single paragraph.

A blank line starts a new paragraph. You can use any punctuation,
including apostrophes (it's, won't, couldn't) without issues.
```

### Choices

Choices use `*` followed by `[text]`:

```
* [Choice text here]
  ~ effect
  -> target
```

Choices must be indented consistently. Effects and navigation belong to the choice above them.

### Choice Conditions

Add conditions in braces after the choice text:

```
* [Attempt repair] { crew.engineering }
  ~ hull += 10
  -> END

* [Buy supplies] { credits >= 50 }
  ~ credits -= 50
  ~ supplies += 20
  -> END

* [Secret option] { flag secret_unlocked }
  -> secret_ending
```

#### Condition Types

| Syntax | Meaning |
|--------|---------|
| `{ crew.tagname }` | Requires crew with tag |
| `{ ship.tagname }` | Requires ship with tag |
| `{ cargo.tagname }` | Requires cargo with tag |
| `{ credits >= 50 }` | Resource comparison |
| `{ flag_name }` | Flag must be set (truthy) |
| `{ !flag_name }` | Flag must NOT be set |

Multiple conditions can be comma-separated:

```
* [Complex option] { crew.engineering, credits >= 100, !already_tried }
```

## Effects Reference

Effects modify game state. They start with `~` and must be indented under a choice.

### Resource Effects

```
~ credits += 50      # Add 50 credits
~ fuel -= 10         # Subtract 10 fuel
~ morale = 75        # Set morale to exactly 75
~ hull += 20         # Repair 20 hull
~ supplies -= 5      # Consume 5 supplies
```

### Flag Effects

```
~ flag quest_started              # Set flag to true
~ flag visit_count = 3            # Set flag to number
~ flag player_choice = "helped"   # Set flag to string
```

### Card Effects

```
~ addCard cargo_memory_cores      # Add card to player inventory
~ addCard crew_engineer           # Add crew card
```

### Chronicle Effects

Chronicle entries appear in the player's journal:

```
~ chronicle "Entry Title"
  This is the chronicle text. It can span multiple lines
  as long as they're indented under the chronicle effect.
```

Single-line version:

```
~ chronicle "Short Entry"
  Brief note about what happened.
```

### Damage Effects

```
~ damage hull 10       # Deal 10 hull damage
~ damage morale 5      # Deal 5 morale damage
```

Note: Damage is different from resource subtraction. It may trigger special game logic.

### Reputation Effects

```
~ reputation guild += 10      # Increase guild reputation
~ reputation consortium -= 5  # Decrease consortium reputation
```

## Navigation

### Basic Navigation

```
-> passage_name    # Go to named passage
-> END             # End the scene
```

### Multi-Passage Flow

```
=== intro

You arrive at the station.

* [Enter the bar]
  -> bar

* [Visit the market]
  -> market

* [Leave immediately]
  -> END

=== bar

The bar is dimly lit.

* [Order a drink]
  ~ credits -= 5
  ~ morale += 3
  -> END

* [Leave]
  -> intro

=== market

Merchants hawk their wares.

* [Browse goods]
  -> END
```

## Comments

Use `//` for comments (ignored by parser):

```
=== intro

// This comment won't appear in output
The story begins here.

* [Option one]
  // TODO: add more effects later
  ~ credits += 10
  -> END
```

## Output Format

### TypeScript Output (Default)

```bash
node dist/cli/index.js compile scenes/ -o compiled/
```

Generates:

```typescript
// compiled/scene_name.ts
import type { Scenelet, SceneletId } from '../../core/types.js';

export const scene_name: Scenelet = {
  id: "scene_name",
  title: "Scene Title",
  tags: ["tag1", "tag2"],
  requirements: {
    context: "journey",
    shipTags: ["sensor"],
  },
  weight: 10,
  cooldown: 5,
  passages: [
    {
      text: "Prose text here.",
      choices: [
        {
          text: "Choice text",
          effects: {
            resources: { credits: 50 },
            setFlags: { quest_done: true },
          },
        },
      ],
    },
  ],
};
```

Also generates `compiled/index.ts` with all exports.

### JSON Output

```bash
node dist/cli/index.js compile scenes/ -o compiled/ --json
```

Generates `.json` files instead of `.ts`.

## CLI Commands

### compile

```bash
node dist/cli/index.js compile <input-dir> [options]

Options:
  -o, --output <dir>   Output directory (default: ./compiled)
  --json               Output JSON instead of TypeScript
```

### parse

```bash
node dist/cli/index.js parse <file>
```

Outputs the AST as JSON. Useful for debugging.

### validate

```bash
node dist/cli/index.js validate <input-dir>
```

Checks all `.scene` files for errors without generating output.

## Project Structure

```
tools/holdsmith/
├── src/
│   ├── cli/
│   │   └── index.ts        # CLI entry point
│   ├── parser/
│   │   ├── ast.ts          # AST type definitions
│   │   ├── errors.ts       # Parse error types
│   │   ├── lexer.ts        # Tokenizer
│   │   └── parser.ts       # Recursive descent parser
│   └── compiler/
│       └── compiler.ts     # AST → Scenelet compiler
├── tests/
│   ├── parser.test.ts      # Test suite
│   └── fixtures/
│       └── valid/          # Example .scene files
├── dist/                   # Compiled JavaScript (after build)
├── package.json
├── tsconfig.json
└── USAGE.md               # This file
```

## Integration with Cargo Hold

### Recommended Workflow

1. Create `.scene` files in `src/content/scenelets/scenes/`
2. Compile: `node tools/holdsmith/dist/cli/index.js compile src/content/scenelets/scenes/ -o src/content/scenelets/compiled/`
3. Import compiled scenelets in game code

### File Organization

```
src/content/scenelets/
├── scenes/                 # Source .scene files
│   ├── journey/
│   │   ├── strange_signal.scene
│   │   └── hull_breach.scene
│   └── port/
│       ├── desperate_seller.scene
│       └── old_captain.scene
├── compiled/               # Generated TypeScript (gitignored or committed)
│   ├── journey/
│   ├── port/
│   └── index.ts
└── index.ts               # Exports ALL_SCENELETS combining manual + compiled
```

## Limitations

### Current Limitations

1. **Single quotes in strings**: Use double quotes for string literals (`"title"` not `'title'`)
2. **No expressions in effects**: Can't do `~ credits += fuel * 2`, only constants
3. **No conditional effects**: Effects always apply; use separate choices for branching
4. **No variables/interpolation**: Prose text is static, can't embed `{player_name}`

### Cargo Hold-Specific

This version of Holdsmith is tailored to Cargo Hold's type system:

- Resources: `credits`, `fuel`, `supplies`, `hull`, `morale`
- Tag sources: `crew`, `ship`, `cargo`
- Contexts: `journey`, `port`, `any`
- Damage targets: `hull`, `morale`

For other games, fork and modify the hardcoded values in `ast.ts`, `parser.ts`, and `compiler.ts`.

## Testing

```bash
cd tools/holdsmith
npm test           # Watch mode
npm run test:run   # Single run
```

## Troubleshooting

### "Expected passage header ==="

Your file might be missing the frontmatter closing `---` or have malformed YAML.

### "Unknown passage: xyz"

A `-> xyz` navigation references a passage that doesn't exist. Check spelling.

### "Expected ] to close choice text"

Choice text must be on one line: `* [This is valid]` not split across lines.

### Chronicle text appears wrong

Make sure chronicle content is indented more than the `~` line:

```
// Wrong - no indent
~ chronicle "Title"
This text won't be captured.

// Right - indented
~ chronicle "Title"
  This text will be captured correctly.
```

## Examples

See `tests/fixtures/valid/` for working examples:

- `simple.scene` - Basic single-passage scene
- `branching.scene` - Multi-passage with conditions and all effect types
