# Blackwing V2 Vision

A comprehensive redesign of the game engine, compiler, and rendering systems.

---

## V1 Architecture Summary

### Game Engine (src/core/)
- **State Management**: Immutable state updates via pure functions, Redux-like dispatch pattern
- **Event System**: Weighted scenelet selection with cooldowns, requirements checking
- **Simulation**: Simple turn-based cycles with journey wear, cargo decay, contract timers
- **Persistence**: localStorage-based save/load with schema versioning

### Holdsmith Compiler (tools/holdsmith/)
- **Lexer**: Hand-written tokenizer with indent/dedent tracking
- **Parser**: Recursive descent, YAML frontmatter + Ink-inspired body
- **Compiler**: AST → TypeScript code generation
- **DSL Features**: Passages, choices, conditions, effects, navigation

### Renderer (src/ui/)
- **Approach**: Vanilla JS string template rendering with innerHTML
- **Re-render**: Full DOM replacement on every state change
- **Animation**: Simple typewriter effect for event text

---

## V2 Vision: "The Blackwing Engine"

### 1. Compiler & Runtime Architecture

#### Current Limitations
- No expressions in effects (can't do `credits += fuel * 2`)
- No variables/interpolation in prose
- No conditional effects (always apply)
- Single-threaded, synchronous evaluation
- No type safety between DSL and runtime

#### V2: Expression Language & VM

```
┌─────────────────────────────────────────────────────────────────┐
│                     HOLDSMITH V2 COMPILER                        │
├─────────────────────────────────────────────────────────────────┤
│  .scene files                                                   │
│       ↓                                                         │
│  ┌─────────────┐   ┌──────────────┐   ┌───────────────────────┐ │
│  │   Lexer     │ → │   Parser     │ → │  Type Checker / IR    │ │
│  │ (tokens)    │   │   (AST)      │   │  (typed expressions)  │ │
│  └─────────────┘   └──────────────┘   └───────────────────────┘ │
│                                              ↓                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                   Code Generator                             │ │
│  │  • TypeScript (current)                                      │ │
│  │  • Bytecode for VM (new)                                     │ │
│  │  • WASM (future)                                             │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

**New DSL Features:**

```ink
=== intro

// Variables and interpolation
~ let reward = base_reward * difficulty_multiplier
The guild offers you {reward} credits for the job.

// Conditional effects
* [Accept the job]
  ~ credits += reward
  ~ if (reputation.guild >= 50) {
      credits += bonus
      chronicle "Guild Favor" "Your standing earns a bonus."
    }
  -> mission_start

// Procedural text selection
The captain speaks. {pick(
  "The void remembers.",
  "Another cycle, another haul.", 
  "We endure."
)}

// State queries in prose
Your hull integrity is at {resources.hull}%.
{if resources.hull < 30: "Warning klaxons echo."}

// Loops for procedural content
~ for (item in cargo) {
    if (item.tag == "volatile") {
      damage integrity 2
    }
  }
```

**Expression VM:**
- Stack-based bytecode interpreter
- Typed operations (number, string, boolean, list, record)
- Built-in functions: `pick()`, `range()`, `count()`, `sum()`
- State queries: `resources.*`, `flags.*`, `cards.has()`, `crew.any()`

#### V2: Incremental Compilation & Hot Reload

```typescript
interface CompilerCache {
  scenes: Map<string, {
    ast: SceneFile;
    bytecode: Uint8Array;
    hash: string;
    dependencies: string[];
  }>;
  schema: SchemaVersion;
}

// Watch mode with incremental rebuild
compiler.watch({
  onSceneChange: (file) => {
    const affected = getAffectedScenes(file);
    recompileIncremental(affected);
    hotReload(affected);
  }
});
```

---

### 2. Runtime Engine

#### Current Limitations
- Full state copy on every mutation (expensive)
- No undo/redo
- Linear event queue
- No parallel/async events
- Simple RNG (LCG)

#### V2: Operational Transform State

```typescript
// Instead of immutable copies, use operations
interface Operation {
  type: 'SET' | 'ADD' | 'REMOVE' | 'TRANSFORM';
  path: string[];
  value?: unknown;
  transform?: (prev: unknown) => unknown;
  timestamp: number;
}

interface GameRuntime {
  state: GameState;
  operations: Operation[];  // History for undo/redo
  subscribers: Map<string, Set<Subscriber>>;  // Path-specific reactivity
  
  apply(op: Operation): void;
  undo(): void;
  redo(): void;
  subscribe(path: string, callback: Subscriber): Unsubscribe;
}
```

**Selective Reactivity:**
```typescript
// Only re-render what changed
runtime.subscribe('resources.credits', (newValue) => {
  updateCreditsDisplay(newValue);
});

runtime.subscribe('cards.deck', (newDeck) => {
  updateDeckView(newDeck);
});
```

#### V2: Async Event System

```typescript
interface EventQueue {
  // Events can be scheduled, delayed, or conditional
  schedule(event: GameEvent, options?: {
    delay?: number;         // Cycles until trigger
    condition?: Expression; // Only trigger if true
    priority?: number;      // Higher = earlier in cycle
    cancellable?: boolean;  // Can be cancelled by other events
  }): EventHandle;
  
  // Parallel event resolution
  async resolve(events: GameEvent[]): Promise<EventResult[]>;
}

// Support for event chains and interrupts
interface GameEvent {
  scenelet: Scenelet;
  interrupt?: boolean;  // Pause current event
  chain?: SceneletId[]; // Events to trigger after
}
```

#### V2: Better RNG

```typescript
// Cryptographically-influenced seeded RNG with multiple streams
interface RNGSystem {
  main: SeededRNG;           // Game state RNG
  events: SeededRNG;         // Event selection (separate for replay)
  cosmetic: SeededRNG;       // UI variation (doesn't affect state)
  
  // Reproducible from seed for testing/replay
  fork(stream: string): SeededRNG;
  
  // Weighted random with guarantees
  weightedPick<T>(items: T[], weights: number[], options?: {
    pityCounter?: number;  // Guarantee rare after N misses
    uniqueUntil?: number;  // No repeats for N picks
  }): T;
}
```

---

### 3. Rendering System

#### Current Limitations
- Full innerHTML replacement every render
- No virtual DOM or diffing
- String template concatenation
- No component reuse
- Blocking typewriter animation

#### V2: Reactive Component System

```typescript
// Lightweight reactive components (no React/Vue dependency)
interface Component<Props, State> {
  props: Props;
  state: State;
  
  // Declarative render
  render(): VNode;
  
  // Lifecycle
  onMount?(): void;
  onUpdate?(prevProps: Props, prevState: State): void;
  onUnmount?(): void;
  
  // State updates trigger re-render of this subtree only
  setState(update: Partial<State>): void;
}

// Virtual DOM with keyed diffing
interface VNode {
  tag: string | Component;
  props: Record<string, unknown>;
  children: (VNode | string)[];
  key?: string;
}

// Efficient updates
function patch(container: Element, oldVNode: VNode, newVNode: VNode): void {
  // Only update changed nodes
  // Key-based list reconciliation
  // Attribute diffing
}
```

**Template DSL:**
```typescript
// JSX-like but compiled to efficient VNode creation
const EventView = component<{ event: TriggeredEvent }>((props) => {
  const { event } = props;
  const [typing, setTyping] = useState(true);
  
  return html`
    <div class="view-event">
      <h2>${event.scenelet.title}</h2>
      <TypewriterText 
        text=${event.passage.text} 
        onComplete=${() => setTyping(false)}
      />
      ${!typing && html`
        <div class="event-choices">
          ${event.choices.map((choice, i) => html`
            <button 
              key=${i}
              class="btn btn-choice"
              disabled=${!meetsRequirements(choice)}
              onclick=${() => selectChoice(i)}
            >
              ${choice.text}
            </button>
          `)}
        </div>
      `}
    </div>
  `;
});
```

#### V2: Animation System

```typescript
interface AnimationController {
  // Declarative animation definitions
  define(name: string, keyframes: Keyframe[], options: AnimationOptions): void;
  
  // Sequence multiple animations
  sequence(animations: Animation[]): AnimationGroup;
  
  // Parallel animations
  parallel(animations: Animation[]): AnimationGroup;
  
  // State-driven transitions
  transition(
    element: Element,
    fromState: string,
    toState: string
  ): Promise<void>;
}

// Typewriter as an animation primitive
const typewriter = animate({
  name: 'typewriter',
  property: 'text-content',
  from: '',
  to: fullText,
  duration: fullText.length * 25,
  easing: 'step-end',
  onUpdate: (progress) => {
    element.textContent = fullText.slice(0, Math.floor(progress * fullText.length));
  }
});
```

#### V2: Canvas/WebGL Hybrid Rendering

```typescript
interface RenderLayer {
  type: 'dom' | 'canvas' | 'webgl';
  zIndex: number;
  render(ctx: RenderContext): void;
}

// DOM for text-heavy UI
const uiLayer: RenderLayer = {
  type: 'dom',
  zIndex: 100,
  render: () => renderUI()
};

// Canvas for starfield, particles
const backgroundLayer: RenderLayer = {
  type: 'canvas',
  zIndex: 0,
  render: (ctx) => {
    drawStarfield(ctx);
    drawNebula(ctx);
  }
};

// WebGL for complex effects
const effectsLayer: RenderLayer = {
  type: 'webgl',
  zIndex: 50,
  render: (gl) => {
    drawJumpGateEffect(gl);
    drawShieldFlicker(gl);
  }
};
```

---

### 4. Content System

#### V2: Entity-Component-System for Cards

```typescript
// Current: monolithic CardDef
// V2: Composable components

interface Entity {
  id: EntityId;
  components: Map<ComponentType, Component>;
}

interface CargoComponent {
  type: 'Cargo';
  baseValue: number;
  mass: number;
  volume: number;
}

interface DecayComponent {
  type: 'Decay';
  rate: number;
  condition: number;
  onDecay?: Effect[];
}

interface VolatileComponent {
  type: 'Volatile';
  triggerChance: number;
  eventPool: SceneletId[];
}

// Compose cards from components
const seraSample: Entity = {
  id: 'cargo_sera_samples',
  components: new Map([
    ['Cargo', { baseValue: 150, mass: 5, volume: 1 }],
    ['Volatile', { triggerChance: 0.15, eventPool: ['sera_incident'] }],
    ['Rare', { rarity: 'rare' }],
    ['Tagged', { tags: ['volatile', 'contraband'] }],
  ])
};

// Systems process entities with specific components
function volatileSystem(entities: Entity[], state: GameState): Effect[] {
  const effects: Effect[] = [];
  for (const entity of entities.filter(e => e.components.has('Volatile'))) {
    const volatile = entity.components.get('Volatile') as VolatileComponent;
    if (rng() < volatile.triggerChance) {
      effects.push({ type: 'TriggerEvent', eventPool: volatile.eventPool });
    }
  }
  return effects;
}
```

#### V2: Procedural Content Generation

```typescript
interface ContentGenerator {
  // Generate scenelets from templates + parameters
  generateScenelet(template: SceneletTemplate, params: GeneratorParams): Scenelet;
  
  // Procedural port generation
  generatePort(seed: number, region: Region): PortState;
  
  // Dynamic contract generation
  generateContract(
    player: PlayerProfile,
    currentPort: PortState,
    difficulty: number
  ): ContractDef;
}

// Template-based generation
const encounterTemplate: SceneletTemplate = {
  titlePattern: '{enemy_type} {encounter_verb}',
  prosePatterns: [
    'Sensors detect {enemy_count} {enemy_type} {approach_verb}.',
    '{dramatic_opening}. The {enemy_type} {action_verb}.',
  ],
  choiceGenerators: [
    {
      condition: 'ship.combat',
      text: 'Engage {enemy_type}',
      effectTemplate: { combat: true, enemy: '{enemy_type}' }
    },
    {
      condition: 'ship.propulsion',
      text: 'Attempt to flee',
      effectTemplate: { flee: true, fuelCost: '{escape_fuel}' }
    }
  ],
  variables: {
    enemy_type: ['pirates', 'sera_drones', 'raiders'],
    encounter_verb: ['Encounter', 'Ambush', 'Contact'],
    // ...
  }
};
```

---

### 5. Advanced Features

#### V2: Mod System

```typescript
interface Mod {
  id: string;
  name: string;
  version: string;
  
  // What the mod provides
  content: {
    scenelets?: Scenelet[];
    cards?: CardDef[];
    ports?: PortDef[];
    factions?: FactionDef[];
  };
  
  // Hooks into engine
  hooks?: {
    onGameStart?: (state: GameState) => GameState;
    onEventSelect?: (events: Scenelet[], ctx: EventContext) => Scenelet[];
    onEffectApply?: (effect: Effect, state: GameState) => Effect;
  };
  
  // Asset overrides
  assets?: {
    css?: string;
    images?: Record<string, string>;
  };
}

// Mod loading
const modLoader = createModLoader({
  validateSchema: true,
  sandboxExecution: true,
  loadOrder: 'dependency-resolved'
});

await modLoader.load('./mods/expanded-universe/');
```

#### V2: Multiplayer Foundation

```typescript
// Deterministic state machine enables multiplayer
interface SyncedGameState {
  // All state changes are operations
  applyOperation(op: Operation): void;
  
  // State can be reconstructed from operation log
  reconstruct(operations: Operation[]): GameState;
  
  // Conflict resolution
  merge(local: Operation[], remote: Operation[]): Operation[];
}

// Async multiplayer (not real-time)
interface TradingPostNetwork {
  // Share discoveries
  shareDiscovery(discovery: Discovery): Promise<void>;
  
  // Async trading
  postTrade(offer: TradeOffer): Promise<TradeId>;
  acceptTrade(id: TradeId): Promise<TradeResult>;
  
  // Shared universe events
  subscribeToEvents(): AsyncIterable<UniverseEvent>;
}
```

#### V2: Analytics & Telemetry

```typescript
interface GameAnalytics {
  // Player behavior
  trackChoice(sceneletId: string, choiceIndex: number, context: ChoiceContext): void;
  trackOutcome(sceneletId: string, outcome: Outcome): void;
  
  // Content health
  getSceneletStats(): Map<SceneletId, {
    timesShown: number;
    choiceDistribution: number[];
    averageEngagement: number;
    skipRate: number;
  }>;
  
  // Balance insights
  getEconomyMetrics(): {
    averageCreditsPerCycle: number;
    resourceBottlenecks: string[];
    cardAcquisitionRates: Map<CardDefId, number>;
  };
}
```

---

### 6. Developer Experience

#### V2: Holdsmith Studio

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ HOLDSMITH STUDIO v2                                    [Build] [Test] [Stats]│
├────────┬─────────────────────────────────────────────────────────────────────┤
│ FILES  │ [Code] [Visual] [Preview] [Debug]                                   │
│        ├─────────────────────────────────────────────────────────────────────┤
│ src/   │  === intro                                                          │
│  j/    │                                                                     │
│   x.s  │  The sensor array chirps. {pick(                                   │
│  p/    │    "Anomaly detected.",                                            │
│        │    "Signal in the void.",                                          │
│ ────── │  )}                                                                 │
│ SCHEMA │                                                                     │
│ ────── │  * [Investigate] { fuel >= 5 }                                     │
│ cards  │    ~ fuel -= 5                                                      │
│ ports  │    ~ let discovered = pick(discoveries)                            │
│ tags   │    ~ chronicle "Discovery" "Found {discovered.name}."              │
│ ────── │    -> outcome_{discovered.type}                                    │
│ ISSUES │                                                                     │
│ ────── │  * [Ignore] -> END                                                 │
│ 0 err  │                                                                     │
│ 1 warn │ ──────────────────────────────────────────────────────────────────  │
│        │ TYPE INFERENCE:                                                     │
│        │   discovered: Discovery { name: string, type: "artifact"|"data" }  │
│        │   outcome_artifact: Passage (exists)                                │
│        │   outcome_data: Passage (exists)                                    │
├────────┴─────────────────────────────────────────────────────────────────────┤
│ WARNINGS: Line 8: `discoveries` not defined in scope. Did you mean global?   │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Features:**
- **LSP Integration**: Full language server for VS Code with errors, completions, hover info
- **Visual Flow Editor**: React Flow with full bidirectional sync to text
- **Type Inference Display**: Shows inferred types inline
- **Live Preview**: Game simulation with editable state
- **Coverage Analysis**: Which paths have been tested
- **Balance Dashboard**: Economy simulation and metrics

---

## Migration Path: V1 → V2

| Component | V1 | V2 |
|-----------|----|----|
| **State** | Immutable copies | Operational transforms |
| **Rendering** | innerHTML templates | Virtual DOM + canvas layers |
| **DSL** | Static effects | Expression language + VM |
| **Compilation** | Full rebuild | Incremental + hot reload |
| **Events** | Synchronous queue | Async with scheduling |
| **Content** | Monolithic types | Entity-Component-System |
| **Tooling** | CLI compiler | Full IDE (Holdsmith Studio) |
| **RNG** | Simple LCG | Multi-stream cryptographic |
| **Extensibility** | None | Mod system with hooks |

---

## Implementation Phases

### Phase 1: Core Infrastructure
- [ ] Operational transform state management
- [ ] Path-based reactivity system
- [ ] Multi-stream RNG
- [ ] Operation history (undo/redo foundation)

### Phase 2: Expression Language
- [ ] Expression lexer/parser
- [ ] Type checker for expressions
- [ ] Bytecode compiler
- [ ] Stack-based VM interpreter
- [ ] Built-in functions (`pick`, `count`, `sum`, etc.)

### Phase 3: Reactive Rendering
- [ ] Virtual DOM implementation
- [ ] Keyed diffing algorithm
- [ ] Component lifecycle
- [ ] Template literal DSL (`html\`...\``)
- [ ] Animation controller

### Phase 4: Visual Layers
- [ ] Canvas background layer (starfield, nebula)
- [ ] Particle system
- [ ] WebGL effects layer (optional)
- [ ] Layer compositing

### Phase 5: Holdsmith V2
- [ ] Expression support in DSL
- [ ] Template interpolation
- [ ] Conditional effects
- [ ] Incremental compilation
- [ ] Hot reload protocol

### Phase 6: Content Systems
- [ ] Entity-Component-System refactor
- [ ] Procedural generation framework
- [ ] Template-based content generation
- [ ] Content validation tools

### Phase 7: Advanced Features
- [ ] Mod loader and sandboxing
- [ ] Analytics integration
- [ ] Async event system
- [ ] Multiplayer foundation (operation sync)

### Phase 8: Holdsmith Studio
- [ ] Language Server Protocol implementation
- [ ] VS Code extension
- [ ] Web-based IDE
- [ ] Visual flow editor
- [ ] Coverage and balance tools

---

## Open Questions

1. **Expression Complexity**: How complex should expressions be? Full Turing-complete, or deliberately limited?
2. **Mod Security**: How to sandbox mod code while allowing powerful hooks?
3. **Multiplayer Scope**: Async trading only, or real-time cooperative play?
4. **Backward Compatibility**: Auto-migrate V1 content, or clean break?
5. **Performance Targets**: Mobile-first, or desktop-optimized?

---

## Success Criteria

V2 is complete when:
- [ ] Can run existing V1 content unchanged (compatibility layer)
- [ ] Expression language handles 90% of procedural content needs
- [ ] Rendering maintains 60fps on mobile with visual effects
- [ ] Hot reload updates content in <100ms
- [ ] At least one community mod published
- [ ] Holdsmith Studio used for all new content

---

*Last updated: January 2025*
