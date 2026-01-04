# Holdsmith: Scene DSL Toolkit

A standalone toolkit for authoring narrative scenelets for Cargo Hold and similar games.

## Vision

Enable non-coders to create rich, branching narrative content through:
- A clean, Ink-inspired DSL (`.scene` files)
- A rich web-based authoring tool with visual editing
- Real-time validation against game schema
- Live preview and testing

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      HOLDSMITH AUTHORING TOOL                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ Text Editor  │  │ Visual Editor│  │   Preview    │           │
│  │ (CodeMirror) │  │ (React Flow) │  │  (Game Sim)  │           │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┘           │
│         │                 │                                     │
│         ▼                 ▼                                     │
│  ┌────────────────────────────────────┐                         │
│  │         Scene AST (in-memory)      │                         │
│  └────────────────┬───────────────────┘                         │
└───────────────────┼─────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────┐
│           .scene files                │  ← Human-readable DSL
│     (source of truth)                 │    Editable in any editor
└───────────────────┬───────────────────┘
                    │
                    ▼ (holdsmith compile)
┌───────────────────────────────────────┐
│     Compiled TypeScript/JSON          │  ← What the game runtime
│     (type-safe, validated)            │    actually imports
└───────────────────────────────────────┘
```

---

## The `.scene` DSL

### File Structure

```
content/
  scenes/
    manifest.yaml          # Discovery index & schema refs
    journey/
      strange-signal.scene
      hull-breach.scene
    port/
      desperate-seller.scene
  schema/
    cards.yaml             # Valid card IDs
    ports.yaml             # Valid port IDs
    factions.yaml          # Valid faction IDs
    tags.yaml              # Valid tags
```

### Manifest File (`manifest.yaml`)

```yaml
version: 1
game: cargo-hold

schema:
  cards: ./schema/cards.yaml
  ports: ./schema/ports.yaml
  factions: ./schema/factions.yaml
  tags: ./schema/tags.yaml

sources:
  - journey/
  - port/

defaults:
  cooldown: 5
  weight: 10
```

### Scene File Syntax

```ink
---
id: strange_signal
title: Signal in the Dark
tags: [mystery, discovery, prior]
context: journey
weight: 10
cooldown: 5
requires:
  shipTags: [sensor]
---

=== intro

The sensor array chirps—an anomaly in the void. A signal, repeating 
in patterns that feel almost like language. Old. Pre-Silence old.

It's coming from somewhere off your plotted course. Investigating 
would cost fuel, but signals like this don't appear on any chart.

* [Investigate the signal]
  ~ fuel -= 5
  ~ flag signal_investigated
  -> derelict

* [Log coordinates and continue]
  ~ flag signal_logged
  ~ chronicle "Signal Logged"
    Detected an anomalous signal—pre-Silence origin, possibly Prior.
    Logged coordinates for future investigation.
  -> END

* [Ignore it—nothing good comes from the dark]
  -> END

=== derelict

The source is a derelict, drifting in the void. Her hull is pocked 
with three centuries of micrometeorite impacts.

* [Take the data core]
  ~ addCard cargo_memory_cores
  ~ flag found_derelict_core
  ~ chronicle "The Derelict's Secret"
    Salvaged a memory core from a Silence-era derelict.
  -> END

* [Leave them undisturbed]
  ~ morale += 5
  ~ chronicle "Respect for the Dead"
    Found a Silence-era derelict. Left her crew to their rest.
  -> END
```

### Syntax Reference

| Element | Syntax | Example |
|---------|--------|---------|
| Frontmatter | YAML between `---` | `id: my_scene` |
| Passage | `=== name` | `=== intro` |
| Prose | Plain paragraphs | Multi-line text |
| Choice | `* [text]` | `* [Fight back]` |
| Conditional choice | `* [text] { condition }` | `* [Repair] { crew.engineering }` |
| Effect | `~ effect` | `~ fuel -= 5` |
| Navigate | `-> passage` | `-> derelict` |
| End scene | `-> END` | Terminal |
| Comment | `// comment` | `// TODO: more options` |

### Effect Commands

```ink
// Resources (additive)
~ credits += 50
~ credits -= 20
~ fuel -= 5
~ supplies -= 3
~ hull -= 10
~ morale += 8

// Flags
~ flag my_flag                    // set true
~ flag my_flag = false            // set false  
~ flag counter = 3                // set number
~ flag name = "value"             // set string

// Cards
~ addCard cargo_memory_cores
~ addCard crew_stowaway

// Chronicle entries (multi-line)
~ chronicle "Entry Title"
  The text of the chronicle entry.
  Can span multiple lines.

// Damage
~ damage hull 10
~ damage morale 5

// Reputation
~ reputation free_traders += 10
~ reputation consortium -= 5
```

### Conditions

Used in choice requirements: `* [Choice text] { conditions }`

```ink
// Tag checks
{ crew.engineering }              // crew has engineering tag
{ ship.combat }                   // ship has combat tag
{ cargo.volatile }                // cargo has volatile tag

// Resource checks  
{ credits >= 50 }
{ fuel >= 10 }
{ morale <= 30 }
{ hull < 50 }

// Flag checks
{ flag visited_before }           // flag is truthy
{ !flag hostile }                 // flag is falsy
{ flag counter >= 3 }             // numeric comparison

// Combined (all must be true)
{ crew.engineering, hull >= 20 }
```

---

## Directory Structure

```
tools/
  holdsmith/
    package.json
    tsconfig.json
    vite.config.ts
    
    src/
      # Core DSL Engine
      parser/
        lexer.ts              # Tokenize .scene files
        parser.ts             # Build AST from tokens
        ast.ts                # AST node type definitions
        errors.ts             # Error types with locations
        
      compiler/
        compiler.ts           # AST -> Scenelet types
        validator.ts          # Validate against schema
        emitter.ts            # Output TypeScript/JSON
        
      schema/
        loader.ts             # Load YAML schema files
        types.ts              # Schema type definitions
        
      cli/
        index.ts              # CLI entry point
        commands/
          compile.ts          # holdsmith compile
          validate.ts         # holdsmith validate  
          watch.ts            # holdsmith watch
          init.ts             # holdsmith init
          serve.ts            # holdsmith serve (authoring tool)
      
      # Authoring Tool (Web App)
      app/
        main.tsx              # App entry
        App.tsx               # Root component
        store.ts              # Zustand store
        
        components/
          Layout/
            Header.tsx
            Sidebar.tsx
            MainPanel.tsx
            
          FileTree/
            FileTree.tsx
            FileItem.tsx
            
          Editor/
            TextEditor.tsx    # CodeMirror wrapper
            VisualEditor.tsx  # React Flow wrapper
            Preview.tsx       # Game simulation
            EditorTabs.tsx    # Text/Visual/Preview switcher
            
          Palette/
            EffectsPalette.tsx
            ConditionBuilder.tsx
            
          Validation/
            ErrorPanel.tsx
            ErrorItem.tsx
            
          PassageList/
            PassageList.tsx
            PassageItem.tsx
            
        codemirror/
          scene-language.ts   # Language definition
          highlighting.ts     # Syntax highlighting
          autocomplete.ts     # Completions
          linting.ts          # Live validation
          
        reactflow/
          nodes/
            PassageNode.tsx
            ChoiceEdge.tsx
          scene-to-graph.ts   # AST -> nodes/edges
          graph-to-scene.ts   # nodes/edges -> AST
          
        preview/
          GameSimulator.ts    # Minimal game state sim
          PreviewRenderer.tsx
          StateEditor.tsx     # Edit test state
          
    public/
      index.html
      
    tests/
      parser/
        lexer.test.ts
        parser.test.ts
      compiler/
        compiler.test.ts
        validator.test.ts
      fixtures/
        valid/
          simple.scene
          branching.scene
          conditions.scene
        invalid/
          bad-syntax.scene
          unknown-card.scene
```

---

## Implementation Phases

### Phase 1: Core Parser & Compiler
**Goal**: Convert `.scene` files to game-compatible TypeScript

- [ ] Define AST types (`ast.ts`)
- [ ] Implement lexer with position tracking
- [ ] Implement recursive descent parser
- [ ] Implement compiler to Scenelet types
- [ ] Basic error messages with line numbers
- [ ] CLI: `holdsmith compile <dir>`
- [ ] Test suite with valid/invalid fixtures

**Deliverable**: Can author scenes in `.scene` format and compile to TypeScript

### Phase 2: Schema & Validation  
**Goal**: Catch errors before they hit the game

- [ ] YAML schema file format
- [ ] Schema loader
- [ ] Validator: check all IDs exist
- [ ] Validator: check tag names
- [ ] Helpful error messages with suggestions
- [ ] CLI: `holdsmith validate <dir>`
- [ ] CLI: `holdsmith watch <dir>` (continuous)

**Deliverable**: Real-time feedback on content errors

### Phase 3: Text Editor Tool
**Goal**: Rich editing experience for scene files

- [ ] Vite + React app scaffold
- [ ] File tree sidebar (read directory)
- [ ] CodeMirror 6 integration
- [ ] Custom language mode for `.scene`
- [ ] Syntax highlighting
- [ ] Autocomplete (IDs, tags, effects)
- [ ] Inline error display
- [ ] Effects palette (click to insert)
- [ ] Passage list sidebar
- [ ] CLI: `holdsmith serve` (dev server)

**Deliverable**: Usable web editor with game-aware features

### Phase 4: Visual Editor
**Goal**: Non-coders can work without touching syntax

- [ ] React Flow integration
- [ ] Passage nodes with preview text
- [ ] Choice edges with labels
- [ ] AST → graph conversion
- [ ] Graph → AST conversion  
- [ ] Bidirectional sync (edit either mode)
- [ ] Drag to connect passages
- [ ] Node properties panel
- [ ] Add/delete passages visually

**Deliverable**: Full visual editing capability

### Phase 5: Preview & Testing
**Goal**: Test scenes without running the full game

- [ ] Game state simulator (minimal)
- [ ] Interactive preview panel
- [ ] Click through choices
- [ ] See effects applied
- [ ] Editable test state (set flags, resources)
- [ ] Condition evaluation display
- [ ] Path coverage visualization

**Deliverable**: Complete testing within the tool

### Phase 6: Polish & Documentation
**Goal**: Production-ready toolkit

- [ ] Export compiled bundle for game
- [ ] Import existing TS scenelets → DSL (migration)
- [ ] Scene templates (common patterns)
- [ ] Keyboard shortcuts
- [ ] Undo/redo
- [ ] User documentation
- [ ] Tutorial scenes

**Deliverable**: Ship-ready toolkit

---

## Tech Stack

| Component | Library | Why |
|-----------|---------|-----|
| Language | TypeScript | Type safety, same as game |
| Build | Vite | Fast, modern |
| CLI | Commander.js | Standard, lightweight |
| Web Framework | React 18 | Familiar, good ecosystem |
| State | Zustand | Simple, minimal boilerplate |
| Text Editor | CodeMirror 6 | Best custom language support |
| Visual Editor | React Flow | Mature node-based editor |
| Styling | Tailwind CSS | Fast iteration |
| Testing | Vitest | Matches game's test setup |
| YAML | yaml (npm) | Parse schema/manifest |

---

## CLI Interface

```bash
# Compile scenes to TypeScript
holdsmith compile ./content/scenes -o ./src/content/compiled

# Validate without compiling
holdsmith validate ./content/scenes

# Watch for changes and recompile
holdsmith watch ./content/scenes -o ./src/content/compiled

# Start authoring tool dev server
holdsmith serve ./content/scenes

# Initialize new content directory
holdsmith init ./content
```

---

## Compiled Output

Input (`strange-signal.scene`):
```ink
---
id: strange_signal
title: Signal in the Dark
...
---

=== intro
...
```

Output (`strange-signal.ts`):
```typescript
import type { Scenelet } from '../../core/types.js';

export const strange_signal: Scenelet = {
  id: 'strange_signal' as SceneletId,
  title: 'Signal in the Dark',
  tags: ['mystery', 'discovery', 'prior'],
  requirements: {
    context: 'journey',
    shipTags: ['sensor'],
  },
  weight: 10,
  cooldown: 5,
  passages: [
    {
      text: 'The sensor array chirps—an anomaly in the void...',
      choices: [
        {
          text: 'Investigate the signal',
          effects: {
            resources: { fuel: -5 },
            setFlags: { signal_investigated: true },
          },
          nextPassage: 1,
        },
        // ...
      ],
    },
    // ...
  ],
};
```

---

## Authoring Tool Layouts

### Text Mode
```
┌──────────────────────────────────────────────────────────────────────────┐
│ Holdsmith                                   [Validate] [Export] [⚙]     │
├────────────────┬─────────────────────────────────────────────────────────┤
│ FILES          │ [Text ◀] [Visual] [Preview]                             │
│ ────────────── ├─────────────────────────────────────────────────────────┤
│ 📁 journey     │  ---                                                    │
│   📄 strange-  │  id: strange_signal                                     │
│      signal ◀  │  title: Signal in the Dark                              │
│   📄 hull-     │  tags: [mystery, discovery, prior]                      │
│      breach    │  ---                                                    │
│ 📁 port        │                                                         │
│   📄 desperate │  === intro                                              │
│                │                                                         │
│ ────────────── │  The sensor array chirps—an anomaly in the void.       │
│ PASSAGES       │                                                         │
│ ────────────── │  * [Investigate the signal]                            │
│ ● intro        │    ~ fuel -= 5                                          │
│ ○ derelict     │    ~ flag signal_investigated                           │
│                │    -> derelict                                          │
│ ────────────── │                                                         │
│ ERRORS (0)     │  * [Log coordinates]                                    │
│ WARNINGS (0)   │    -> END                                               │
├────────────────┴─────────────────────────────────────────────────────────┤
│ EFFECTS: [Credits] [Fuel] [Flag] [Card] [Chronicle] [Damage] [Rep]      │
└──────────────────────────────────────────────────────────────────────────┘
```

### Visual Mode
```
┌──────────────────────────────────────────────────────────────────────────┐
│ Holdsmith                                   [Validate] [Export] [⚙]     │
├────────────────┬─────────────────────────────────────────────────────────┤
│ FILES          │ [Text] [Visual ◀] [Preview]                             │
│                ├─────────────────────────────────────────────────────────┤
│ ...            │                                                         │
│                │    ┌─────────────────┐                                  │
│                │    │     intro       │                                  │
│                │    │─────────────────│        ┌─────────────────┐       │
│                │    │ The sensor      │───────▶│    derelict     │       │
│                │    │ array chirps... │ Invest │─────────────────│       │
│                │    │                 │        │ The source is   │       │
│                │    │ ○ Investigate ──┤        │ a derelict...   │       │
│                │    │ ○ Log coords ───┼──┐     │                 │       │
│                │    │ ○ Ignore ───────┼─┐│     │ ○ Take ─────────┼─▶[END]│
│                │    └─────────────────┘ ││     │ ○ Leave ────────┼─▶[END]│
│                │                        ││     └─────────────────┘       │
│                │                        │└────────────────────────▶[END] │
│                │                        └─────────────────────────▶[END] │
│                │                                                         │
│                │    [+ Add Passage]                                      │
├────────────────┴─────────────────────────────────────────────────────────┤
│ SELECTED: intro | Choices: 3 | Connections: 2                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Preview Mode
```
┌──────────────────────────────────────────────────────────────────────────┐
│ Holdsmith                                   [Validate] [Export] [⚙]     │
├────────────────┬─────────────────────────────────────────────────────────┤
│ FILES          │ [Text] [Visual] [Preview ◀]                             │
│                ├─────────────────────────────────────────────────────────┤
│ ...            │  TEST STATE                              [Reset] [Edit] │
│                │  Credits: 100  Fuel: 50  Hull: 80%  Morale: 60%         │
│                │  Crew: [engineering]  Ship: [sensor]                    │
│                │  Flags: (none)                                          │
│                │  ─────────────────────────────────────────────────────  │
│                │                                                         │
│                │              Signal in the Dark                         │
│                │                                                         │
│                │  The sensor array chirps—an anomaly in the void.       │
│                │  A signal, repeating in patterns that feel almost      │
│                │  like language. Old. Pre-Silence old.                  │
│                │                                                         │
│                │  ┌────────────────────────────────────────────────────┐ │
│                │  │ Investigate the signal                     [-5 ⛽] │ │
│                │  └────────────────────────────────────────────────────┘ │
│                │  ┌────────────────────────────────────────────────────┐ │
│                │  │ Log coordinates and continue                       │ │
│                │  └────────────────────────────────────────────────────┘ │
│                │  ┌────────────────────────────────────────────────────┐ │
│                │  │ Ignore it—nothing good comes from the dark         │ │
│                │  └────────────────────────────────────────────────────┘ │
│                │                                                         │
│                │  PATH: intro → (waiting for choice)                     │
├────────────────┴─────────────────────────────────────────────────────────┤
│ EFFECTS PREVIEW: Investigate → fuel: 50→45, flag: signal_investigated   │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Success Criteria

### Phase 1 Complete When:
- [ ] Can parse example `.scene` files without errors
- [ ] Compiled output matches existing Scenelet type structure
- [ ] CLI compiles directory of scenes
- [ ] Tests pass for valid and invalid inputs

### Phase 2 Complete When:
- [ ] Schema files loaded and validated
- [ ] Unknown card/port/tag IDs caught
- [ ] Error messages include line numbers
- [ ] Watch mode detects changes

### Phase 3 Complete When:
- [ ] Web app loads and displays file tree
- [ ] Can edit `.scene` files with syntax highlighting
- [ ] Autocomplete suggests valid IDs
- [ ] Errors display inline

### Phase 4 Complete When:
- [ ] Visual graph renders from scene
- [ ] Can drag to connect passages
- [ ] Edits in visual mode update text
- [ ] Edits in text mode update visual

### Phase 5 Complete When:
- [ ] Preview renders scene passages
- [ ] Can click through choices
- [ ] State updates visible
- [ ] Can edit test state

### Phase 6 Complete When:
- [ ] Documentation complete
- [ ] Templates available
- [ ] Migration tool works
- [ ] Ready for external users

---

## Open Questions (To Resolve During Implementation)

1. **Hot reload**: Should compiled scenes hot-reload in the game during dev?
2. **Versioning**: How to handle schema changes over time?
3. **Localization**: Should the DSL support multiple languages?
4. **Extensions**: Plugin system for custom effects?
5. **Collaboration**: Multi-user editing someday?

---

## Getting Started

```bash
cd tools/holdsmith
npm install
npm run dev        # Start authoring tool
npm run build      # Build CLI + app
npm test           # Run tests
```

---

*Last updated: January 2025*
