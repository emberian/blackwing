# Holdsmith Analyzer v2: Revised Design

*After user feedback: Z3 is core (not optional), integrate with real game engine, profile-based analysis*

## The Core Insight

The analyzer shouldn't be a separate tool that analyzes `.scene` files in isolation. **The analyzer should be integrated with the game engine itself.** The game already has:

1. `GameState` - the complete state representation
2. `applyEffects()` - the actual effect semantics
3. `meetsRequirements()` - the actual guard semantics
4. `createInitialState()` - the canonical starting state
5. `selectEvent()` - the actual scene selection logic

Why would we reimplement these in the analyzer? We'd just be creating a second, possibly divergent, semantics. Instead:

**The analyzer should drive the actual game engine headlessly and use Z3 to explore the state space systematically.**

## Architecture: Symbolic Execution + Concrete Simulation Hybrid

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Holdsmith Analyzer v2                          │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                     Analysis Orchestrator                     │  │
│  │  - Manages exploration strategies                             │  │
│  │  - Collects trajectories                                      │  │  
│  │  - Aggregates diagnostics                                     │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│         ┌────────────────────┼────────────────────┐                │
│         ▼                    ▼                    ▼                │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐          │
│  │  Headless   │     │  Symbolic   │     │   Z3        │          │
│  │  Game       │     │  Executor   │     │   Solver    │          │
│  │  Driver     │     │             │     │             │          │
│  │             │     │ GameState   │     │ - SAT/UNSAT │          │
│  │ Real engine │     │ with        │     │ - Models    │          │
│  │ real rules  │     │ symbolic    │     │ - Optimize  │          │
│  │             │     │ resources   │     │             │          │
│  └─────────────┘     └─────────────┘     └─────────────┘          │
│         │                    │                    │                │
│         └────────────────────┼────────────────────┘                │
│                              ▼                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                     Trajectory Store                          │  │
│  │  - Concrete play traces from simulation                       │  │
│  │  - Symbolic constraints from exploration                      │  │
│  │  - Statistical summaries                                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## Component 1: Headless Game Driver

This is just the game engine, but without UI and with hooks for automated play.

```typescript
// src/core/headless.ts - New file, part of core game

import { createGameController, type GameController } from './controller.js';
import type { GameState, CardDef, Scenelet, PortId, SceneletChoice } from './types.js';

export interface HeadlessController extends GameController {
  /** Get all currently valid choices for the active event */
  getValidChoices(): { index: number; choice: SceneletChoice; meetsRequirements: boolean }[];
  
  /** Get all ports we can travel to from current location */
  getValidDestinations(): PortId[];
  
  /** Force a specific scene to trigger (for testing) */
  forceScene(sceneId: string): boolean;
  
  /** Get the full trajectory of states visited */
  getTrajectory(): TrajectoryStep[];
  
  /** Clone the controller at current state (for branching exploration) */
  fork(): HeadlessController;
  
  /** Fast-forward: apply a sequence of actions */
  replay(actions: PlayAction[]): void;
}

export interface TrajectoryStep {
  state: GameState;
  action: PlayAction;
  sceneTriggered?: string;
  choiceMade?: number;
}

export type PlayAction = 
  | { type: 'travel'; destination: PortId }
  | { type: 'choice'; index: number }
  | { type: 'port_action'; action: GameAction }
  | { type: 'skip_event' };

export function createHeadlessController(
  cardDefs: Map<string, CardDef>,
  scenelets: Scenelet[],
  initialState?: GameState
): HeadlessController;
```

This uses the **real game engine**—same `applyEffects`, same `meetsRequirements`, same everything. No semantic drift.

## Component 2: Exploration Strategies

Different ways to explore the state space:

```typescript
// tools/holdsmith/src/analyzer/strategies.ts

export interface ExplorationStrategy {
  /** Generate next action(s) to try from current state */
  selectActions(controller: HeadlessController): PlayAction[];
  
  /** Called after each step to update strategy state */
  observe(step: TrajectoryStep): void;
  
  /** Should we continue exploring this branch? */
  shouldContinue(controller: HeadlessController): boolean;
}

/** Random playthrough - for statistical coverage */
export class RandomStrategy implements ExplorationStrategy {
  constructor(private seed: number, private maxSteps: number) {}
  // Picks random valid choices, random destinations
}

/** Exhaustive DFS - for small scenes, complete coverage */
export class ExhaustiveStrategy implements ExplorationStrategy {
  constructor(private maxDepth: number) {}
  // Returns ALL valid actions, explores every branch
}

/** Coverage-guided - prioritize unexplored scenes/choices */
export class CoverageGuidedStrategy implements ExplorationStrategy {
  private seenScenes = new Set<string>();
  private seenChoices = new Map<string, Set<number>>();
  // Prioritizes actions that lead to unseen content
}

/** Property-directed - try to reach/avoid specific states */
export class PropertyDirectedStrategy implements ExplorationStrategy {
  constructor(private goal: (state: GameState) => boolean) {}
  // Uses heuristics to reach goal state
}

/** Z3-guided - use solver to find interesting paths */
export class SymbolicStrategy implements ExplorationStrategy {
  constructor(private solver: Z3Backend) {}
  // Asks Z3: "Is there a path where hull <= 0 AND credits >= 500?"
}
```

## Component 3: Symbolic State & Z3 Integration

For queries that concrete simulation can't answer efficiently:

```typescript
// tools/holdsmith/src/analyzer/symbolic.ts

import { init, type Context, type Solver, type Arith, type Bool } from 'z3-solver';

/** Symbolic version of game resources */
export interface SymbolicResources {
  credits: Arith;
  fuel: Arith;
  supplies: Arith;
  hull: Arith;
  integrity: Arith;
}

/** Symbolic flag state */
export interface SymbolicFlags {
  [key: string]: Bool;
}

export class Z3Backend {
  private ctx!: Context;
  private solver!: Solver;
  
  async initialize(): Promise<void>;
  
  /** 
   * Encode a scene as Z3 constraints.
   * Each path through the scene becomes a formula.
   */
  encodeScene(scene: Scenelet, entryState: SymbolicResources): SceneEncoding;
  
  /**
   * Query: Is there ANY path through this scene that results in hull <= 0?
   */
  async canReachDeath(scene: Scenelet, entryBounds: ResourceBounds): Promise<{
    possible: boolean;
    witness?: ConcreteTrajectory;  // If SAT, a concrete path that kills you
  }>;
  
  /**
   * Query: Is there a path where choice C is both reachable AND satisfiable?
   */
  async isChoiceReachable(
    scene: Scenelet, 
    passageName: string, 
    choiceIndex: number,
    entryBounds: ResourceBounds
  ): Promise<boolean>;
  
  /**
   * Query: What are the tightest bounds on credits after this scene?
   */
  async computeOutcomeBounds(
    scene: Scenelet, 
    entryBounds: ResourceBounds
  ): Promise<ResourceBounds>;
  
  /**
   * Query: Given player has seen scenes S1, S2, S3 and has flags F,
   * can they ever see scene S4?
   */
  async isSceneReachable(
    targetScene: Scenelet,
    playerHistory: PlayerHistory,
    allScenes: Scenelet[]
  ): Promise<{ reachable: boolean; requiredPath?: string[] }>;
}

export interface ResourceBounds {
  credits: { min: number; max: number };
  fuel: { min: number; max: number };
  supplies: { min: number; max: number };
  hull: { min: number; max: number };
  integrity: { min: number; max: number };
}
```

## Component 4: Profile-Based Analysis

Simulate real trajectories, then analyze from those profiles.

```typescript
// tools/holdsmith/src/analyzer/profiler.ts

export interface PlayerProfile {
  /** Snapshot of state at profile capture */
  state: GameState;
  
  /** How we got here */
  trajectory: TrajectoryStep[];
  
  /** Summary statistics */
  stats: {
    scenesVisited: Set<string>;
    choicesMade: Map<string, number[]>;  // sceneId -> choice indices
    flagsSet: Set<string>;
    minResources: ResourceBounds;  // Lowest point during play
    maxResources: ResourceBounds;  // Highest point during play
  };
}

export class ProfileGenerator {
  constructor(
    private controller: HeadlessController,
    private strategy: ExplorationStrategy
  ) {}
  
  /** Run N simulated playthroughs, collect profiles */
  async generateProfiles(count: number): Promise<PlayerProfile[]>;
  
  /** Run until we've seen every scene at least once */
  async generateCoverageProfiles(): Promise<PlayerProfile[]>;
  
  /** Run until we find a profile matching predicate */
  async findProfile(predicate: (p: PlayerProfile) => boolean): Promise<PlayerProfile | null>;
}

export class ProfileAnalyzer {
  constructor(private profiles: PlayerProfile[]) {}
  
  /** Which scenes were never reached in any profile? */
  unreachedScenes(allScenes: Scenelet[]): Scenelet[];
  
  /** Which choices were never selected? */
  unchosen(): { scene: string; passage: string; choiceIndex: number }[];
  
  /** What's the distribution of outcomes for a specific scene? */
  sceneOutcomeDistribution(sceneId: string): OutcomeDistribution;
  
  /** Cluster profiles by ending state */
  clusterByOutcome(): ProfileCluster[];
  
  /** Find profiles that lead to game over */
  deathProfiles(): PlayerProfile[];
  
  /** Extract common patterns (e.g., "players who set flag X usually also set Y") */
  flagCorrelations(): FlagCorrelation[];
}
```

## Component 5: Integrated Analysis Pipeline

Combine concrete simulation with symbolic reasoning:

```typescript
// tools/holdsmith/src/analyzer/pipeline.ts

export interface AnalysisConfig {
  /** Number of random playthroughs for statistical coverage */
  simulationCount: number;
  
  /** Max depth for exhaustive scene analysis */
  exhaustiveDepth: number;
  
  /** Use Z3 for precise queries */
  enableSymbolic: boolean;
  
  /** Resource bounds to assume for scene entry */
  entryBounds: ResourceBounds;
}

export class AnalysisPipeline {
  constructor(
    private cardDefs: Map<string, CardDef>,
    private scenelets: Scenelet[],
    private config: AnalysisConfig
  ) {}
  
  async runFullAnalysis(): Promise<AnalysisReport> {
    const report = new AnalysisReport();
    
    // Phase 1: Generate player profiles through simulation
    const profiles = await this.generateProfiles();
    report.profiles = profiles;
    
    // Phase 2: Per-scene structural analysis
    for (const scene of this.scenelets) {
      const sceneReport = await this.analyzeScene(scene, profiles);
      report.sceneReports.set(scene.id, sceneReport);
    }
    
    // Phase 3: Cross-scene analysis
    report.crossScene = await this.analyzeCrossScene(profiles);
    
    // Phase 4: Symbolic verification of critical properties
    if (this.config.enableSymbolic) {
      report.symbolic = await this.symbolicVerification();
    }
    
    return report;
  }
  
  private async analyzeScene(
    scene: Scenelet, 
    profiles: PlayerProfile[]
  ): Promise<SceneReport> {
    // Use profiles to understand realistic entry states
    const entryStates = profiles
      .filter(p => p.stats.scenesVisited.has(scene.id))
      .map(p => /* state just before entering this scene */);
    
    // Exhaustive exploration from each entry state
    const outcomes = await this.exploreScene(scene, entryStates);
    
    // Symbolic analysis for precise bounds
    const symbolicBounds = await this.z3.computeOutcomeBounds(scene, this.config.entryBounds);
    
    return {
      reachability: this.analyzeReachability(scene, outcomes),
      resourceFlow: this.analyzeResourceFlow(scene, outcomes, symbolicBounds),
      deadChoices: this.findDeadChoices(scene, entryStates),
      deathPaths: await this.findDeathPaths(scene),
      statistics: this.computeStatistics(outcomes),
    };
  }
}
```

## What This Enables

### 1. Precise "Can this ever happen?" queries

```typescript
// Can a player ever die in the sera_contact scene?
const result = await z3.canReachDeath(sera_contact_scene, {
  hull: { min: 1, max: 100 },  // Player must be alive to enter
  // ...
});

if (result.possible) {
  console.log("Death path:", result.witness);
  // -> intro -> contain -> "Keep fighting" with hull < 15
}
```

### 2. Grounded balance analysis

```typescript
// What does this scene actually do to players?
const profiles = await profiler.generateProfiles(10000);
const stats = analyzer.sceneOutcomeDistribution('journey_gambling');

// Output:
// journey_gambling outcomes (N=847 encounters):
//   credits: mean=-12.3, std=45.2, min=-80, max=+80
//   60% of players lose money
//   12% of players win big (+45 or more)
//   This is a high-variance scene (coefficient of variation: 3.67)
```

### 3. Cross-scene flag analysis with actual reachability

```typescript
// Is the hollow_contact flag actually settable before it's checked?
const flagAnalysis = await analyzer.analyzeFlagFlow('hollow_contact');

// Output:
// hollow_contact:
//   Set by: port_hollow_contact (3 paths), port_strange_offer (2 paths)
//   Checked by: port_cataclysm_clue:81
//   
//   Reachability analysis (from 10000 profiles):
//     - 23% of playthroughs set this flag
//     - Of those, 89% set it before reaching port_cataclysm_clue
//     - Median cycles to set: 12
//   
//   Symbolic verification: ✓ Flag CAN be set before check
//   Witness: Start -> port_hollow_contact -> choice[0] -> meeting -> choice[0]
```

### 4. Dead content detection with confidence

```typescript
// Find content that's technically reachable but practically never seen
const deadContent = analyzer.findPracticallyDeadContent(profiles);

// Output:
// Practically dead content (seen in <1% of 10000 playthroughs):
//   - port_cataclysm_clue passage "sell_to_hollow" (requires hollow_contact, 0.3% reach rate)
//   - journey_drone_swarm choice "Try different frequency" (requires ship.sensor tag, 0.8% reach rate)
//   
// Suggestions:
//   - Consider lowering hollow_contact requirement, or add more ways to get it
//   - sensor tag only comes from module_sensor_array, consider adding to starting equipment
```

### 5. Counterfactual queries

```typescript
// "What if the player had 50 more starting credits?"
const baseline = await profiler.generateProfiles(1000, { initialCredits: 100 });
const modified = await profiler.generateProfiles(1000, { initialCredits: 150 });

const comparison = analyzer.comparePopulations(baseline, modified);

// Output:
// Impact of +50 starting credits:
//   - 15% more scenes reached on average
//   - 2.3x more likely to survive to cycle 50
//   - Gambling scene participation: 12% -> 45%
//   - Average game length: 23 cycles -> 31 cycles
```

## File Structure

```
tools/holdsmith/
├── src/
│   ├── analyzer/
│   │   ├── index.ts           # Main exports
│   │   ├── pipeline.ts        # Analysis orchestration
│   │   ├── symbolic.ts        # Z3 integration
│   │   ├── profiler.ts        # Profile generation/analysis
│   │   ├── strategies.ts      # Exploration strategies
│   │   ├── diagnostics.ts     # Diagnostic types and formatting
│   │   └── report.ts          # Report generation
│   ├── cli/
│   │   └── index.ts           # CLI (updated with analyze command)
│   ├── parser/                # Existing
│   └── compiler/              # Existing
├── tests/
│   └── analyzer.test.ts
└── package.json               # Add z3-solver dependency

src/core/
├── headless.ts                # NEW: Headless game driver
├── controller.ts              # Existing (minor additions for hooks)
├── events.ts                  # Existing
└── ...
```

## Key Dependencies

```json
{
  "dependencies": {
    "z3-solver": "^4.12.0"
  }
}
```

Z3-solver is ~50MB but it's a **real** dependency, not optional. It's the foundation for answering "is this possible?" questions precisely.

## Open Design Questions

1. **Headless driver location**: Should `headless.ts` live in `src/core/` (part of the game) or `tools/holdsmith/` (part of the analyzer)? I lean toward `src/core/` because it's useful for testing the game itself, not just analysis.

2. **Profile persistence**: Should we save generated profiles to disk? They're expensive to generate but useful for repeated analysis. Could use SQLite or just JSON.

3. **Incremental analysis**: When a single scene changes, we shouldn't re-run everything. How do we cache and invalidate intelligently?

4. **Parallelism**: Profile generation is embarrassingly parallel. Should we use worker threads?

5. **Visualization**: The analysis produces rich data. What output formats? JSON for tooling, but also consider:
   - Graphviz DOT for scene flow graphs
   - HTML reports with interactive charts
   - Integration with VSCode diagnostics

---

## Key Shifts from v1 Design

- **Z3 is core, not optional** - symbolic reasoning is the backbone
- **Real engine, not reimplementation** - the headless driver uses actual game semantics
- **Profile-grounded** - statistical analysis from simulated play, not just theoretical bounds
- **Counterfactual-capable** - can answer "what if?" questions

## Language Discussion

After this design, the question arose: should this be in TypeScript or a different language?

**Arguments for Rust/OCaml/Haskell:**

1. **Z3 bindings are first-class** - The TypeScript z3-solver package is a WASM port that's slower and less complete. Rust has `z3`, OCaml has `z3`, Haskell has `sbv` and `z3`. These are mature, well-maintained FFI bindings to the actual Z3 library.

2. **This is fundamentally a compiler/analyzer problem** - We're doing:
   - AST manipulation
   - Fixed-point computation
   - Symbolic execution
   - Constraint solving
   - Graph algorithms
   
   These are the bread and butter of ML-family languages.

3. **Performance matters** - We're potentially exploring millions of states. Generating thousands of profiles. Solving complex constraint systems.

4. **Type safety for symbolic computation** - In Haskell/OCaml, you can encode the difference between concrete and symbolic values in the type system.

**Decision**: Proceed with full Rust rewrite (see v2-plan.md for the final architecture).
