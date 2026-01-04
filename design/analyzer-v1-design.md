# Holdsmith Analyzer v2: Design Document

*Initial design before Rust rewrite decision*

## 1. Problem Statement

The current analyzer is a simple syntactic checker. For a narrative game like Blackwing, we need **semantic analysis** that can answer questions like:

- "Is this scene always completable, or can the player get stuck?"
- "Does choosing path A make choice B impossible later?"
- "What's the range of possible outcomes (credits, hull, flags) from this scene?"
- "Are there paths that guarantee death?"
- "Does flag X get set before it's checked anywhere in the game?"

This is fundamentally a **program analysis** problem where:
- **Passages** are basic blocks
- **Choices** are conditional branches
- **Effects** are state transformations
- **Requirements** are preconditions/guards

## 2. Formal Model

### 2.1 Scene as a Labeled Transition System

A scene S = (P, p₀, →, F) where:
- **P** = set of passages (nodes)
- **p₀** = initial passage (`intro` by convention)
- **→** ⊆ P × Guard × Effects × P is the transition relation
- **F** = terminal states (passages with `-> END` or no outgoing edges)

Each transition (p, g, e, p') represents:
- From passage p
- If guard g is satisfiable (resource checks, flag checks, tag checks)
- Apply effects e (resource changes, flag sets, card adds)
- Transition to passage p'

### 2.2 Abstract State Domain

We need to track:

```typescript
interface AbstractState {
  // Interval abstraction for resources
  resources: {
    credits: Interval;    // [min, max]
    fuel: Interval;
    supplies: Interval;
    hull: Interval;
    integrity: Interval;
  };
  
  // Three-valued logic for flags
  flags: Map<string, FlagValue>;  // ⊥ (unknown), true, false, ⊤ (both possible)
  
  // Set abstraction for cards (may-have, must-have)
  cards: {
    mayHave: Set<string>;
    mustHave: Set<string>;
  };
  
  // Tags derived from cards
  shipTags: Set<string>;
  crewTags: Set<string>;
  cargoTags: Set<string>;
}

type Interval = { min: number; max: number };
type FlagValue = 'unknown' | 'true' | 'false' | 'maybe';
```

### 2.3 Transfer Functions

For each effect type, define how it transforms the abstract state:

```
⟦credits += n⟧(σ) = σ[credits ↦ [σ.credits.min + n, σ.credits.max + n]]
⟦credits -= n⟧(σ) = σ[credits ↦ [max(0, σ.credits.min - n), max(0, σ.credits.max - n)]]
⟦flag f⟧(σ) = σ[flags[f] ↦ true]
⟦addCard c⟧(σ) = σ[cards.mayHave ∪ {c}, cards.mustHave ∪ {c}]
⟦removeCards pattern⟧(σ) = σ[cards.mustHave ← cards.mustHave \ match(pattern)]
```

### 2.4 Guard Satisfiability

A guard is satisfiable in abstract state σ iff:

```
SAT(credits >= n, σ) ⟺ σ.credits.max >= n
SAT(flag f, σ) ⟺ σ.flags[f] ∈ {true, maybe, unknown}
SAT(!flag f, σ) ⟺ σ.flags[f] ∈ {false, maybe, unknown}
SAT(ship.tag, σ) ⟺ tag ∈ σ.shipTags ∨ σ.shipTags = ⊤
```

## 3. Analyses to Implement

### 3.1 Intra-Scene Analyses

#### 3.1.1 Reachability Analysis
- Build CFG from passages
- Compute strongly connected components (find cycles)
- Find dead passages (no path from initial)
- Find dead choices (guard always false given incoming state)

#### 3.1.2 Termination Analysis
- Check all paths eventually reach END or a terminal passage
- Detect infinite loops (cycles with no exit condition)
- Flag passages that can trap the player

#### 3.1.3 Resource Flow Analysis
- Forward dataflow: compute abstract state at each passage
- Detect: 
  - **Impossible choices**: guard requires credits >= 100 but max possible is 50
  - **Guaranteed death**: hull interval includes 0 with no recovery path
  - **Resource drain**: every path decreases resources with no gain paths

#### 3.1.4 Outcome Computation
- For each terminal state, compute the abstract state
- Report: min/max for each resource across all outcomes
- Report: which flags are definitely/possibly set

### 3.2 Inter-Scene Analyses (Whole-Program)

#### 3.2.1 Flag Dataflow
- Build a flag dependency graph across all scenes
- **Set-Check Analysis**: For each flag check, is there a scene that can set it?
- **Ordering Analysis**: Can flag X be set before scene Y requires it? (requires reasoning about scene selection)

#### 3.2.2 Achievement Verification
- For each achievement that checks a flag, verify the flag can be set
- For resource achievements (credits >= 500), check if any outcome reaches it

#### 3.2.3 Narrative Arc Validation
- **Faction coverage**: Are all factions represented in scenes?
- **Tag coverage**: Are ship tags like `sensor` ever actually gated on?
- **Story continuity**: If scene A sets up a cliffhanger flag, does scene B continue it?

### 3.3 Balance Analysis (Statistical)

#### 3.3.1 Expected Value Computation
- Assign probabilities to choices (uniform by default, or weighted by requirements)
- Compute E[credits], E[hull], etc. for each scene
- Flag scenes that are "traps" (negative EV with no upside paths)

#### 3.3.2 Risk Profile
- Compute variance of outcomes
- Flag high-variance scenes (gambling) vs low-variance (safe encounters)
- Detect "death spiral" scenes (every path is net-negative)

## 4. Implementation Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Analyzer CLI                             │
│   holdsmith analyze <dir> [--strict] [--z3] [--stats]           │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Analysis Pipeline                           │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐     │
│  │  Parse   │ → │ Build IR │ → │ Analyze  │ → │  Report  │     │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘     │
└─────────────────────────────────────────────────────────────────┘
                                │
                ┌───────────────┼───────────────┐
                ▼               ▼               ▼
        ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
        │  CFG Module │ │  Abstract   │ │    Z3       │
        │             │ │Interpreter  │ │  Backend    │
        │ - Build CFG │ │             │ │ (optional)  │
        │ - SCC       │ │ - Intervals │ │             │
        │ - Dominance │ │ - Flags     │ │ - SAT       │
        │ - Paths     │ │ - Cards     │ │ - Symbolic  │
        └─────────────┘ └─────────────┘ └─────────────┘
```

### 4.1 Module: `ir.ts` - Intermediate Representation

```typescript
// Scene as a proper graph
interface SceneIR {
  id: string;
  passages: Map<string, PassageIR>;
  entry: string;
  cfg: CFG;
}

interface PassageIR {
  name: string;
  prose: string;
  choices: ChoiceIR[];
  isTerminal: boolean;
}

interface ChoiceIR {
  text: string;
  guard: Guard;
  effects: Effect[];
  target: string | 'END';
}

// Guards as a proper AST for analysis
type Guard = 
  | { type: 'true' }
  | { type: 'resource'; resource: string; op: Comparator; value: number }
  | { type: 'flag'; name: string; negated: boolean }
  | { type: 'tag'; source: 'ship' | 'crew' | 'cargo'; tag: string }
  | { type: 'and'; clauses: Guard[] };
```

### 4.2 Module: `cfg.ts` - Control Flow Graph

```typescript
interface CFG {
  nodes: Map<string, CFGNode>;
  edges: CFGEdge[];
  entry: string;
  exits: string[];
}

interface CFGNode {
  passage: string;
  predecessors: string[];
  successors: string[];
}

interface CFGEdge {
  from: string;
  to: string;
  guard: Guard;
  effects: Effect[];
}

// Algorithms
function buildCFG(scene: SceneIR): CFG;
function computeSCC(cfg: CFG): string[][];  // Tarjan's
function computeDominators(cfg: CFG): Map<string, Set<string>>;
function findAllPaths(cfg: CFG, from: string, to: string): Path[];
```

### 4.3 Module: `abstract.ts` - Abstract Interpretation

```typescript
// Abstract domain for resources
class IntervalDomain {
  static bot(): Interval { return { min: Infinity, max: -Infinity }; }
  static top(): Interval { return { min: -Infinity, max: Infinity }; }
  static join(a: Interval, b: Interval): Interval;
  static meet(a: Interval, b: Interval): Interval;
  static widen(a: Interval, b: Interval): Interval;  // For loops
}

// Abstract domain for flags
type FlagLattice = 'bot' | 'true' | 'false' | 'top';

class FlagDomain {
  static join(a: FlagLattice, b: FlagLattice): FlagLattice;
  // bot ⊔ true = true, true ⊔ false = top, etc.
}

// Main abstract interpreter
class AbstractInterpreter {
  analyze(scene: SceneIR): Map<string, AbstractState>;
  
  // Fixed-point computation with widening
  private computeFixpoint(cfg: CFG, initial: AbstractState): Map<string, AbstractState>;
  
  // Transfer function for a single edge
  private transfer(state: AbstractState, edge: CFGEdge): AbstractState | null;
  
  // Guard evaluation (returns null if definitely unsatisfiable)
  private evalGuard(guard: Guard, state: AbstractState): AbstractState | null;
}
```

### 4.4 Module: `z3-backend.ts` - SMT Solving (Optional)

For precise queries that abstract interpretation can't answer:

```typescript
import { init } from 'z3-solver';

class Z3Backend {
  private ctx: Context;
  
  // Check if a path is feasible
  async isPathFeasible(path: Path, initialState: Constraints): Promise<boolean>;
  
  // Find a concrete input that reaches a target state
  async findWitness(scene: SceneIR, target: AbstractState): Promise<ConcreteState | null>;
  
  // Verify a property holds for all paths
  async verifyProperty(scene: SceneIR, property: Property): Promise<VerificationResult>;
}

// Encode scene semantics as SMT constraints
function encodeScene(scene: SceneIR): SMTFormula;
function encodeGuard(guard: Guard): SMTFormula;
function encodeEffects(effects: Effect[]): SMTFormula;
```

### 4.5 Module: `diagnostics.ts` - Analysis Results

```typescript
interface AnalysisDiagnostic {
  severity: 'error' | 'warning' | 'info' | 'hint';
  code: DiagnosticCode;
  message: string;
  file: string;
  location?: { passage: string; choice?: number; line?: number };
  
  // Rich diagnostic info
  details?: {
    path?: string[];           // For path-sensitive issues
    abstractState?: AbstractState;  // State at point of issue
    suggestion?: string;
    relatedLocations?: Location[];
  };
}

type DiagnosticCode =
  // Structural
  | 'UNREACHABLE_PASSAGE'
  | 'DEAD_CHOICE'
  | 'INFINITE_LOOP'
  | 'MISSING_TERMINAL'
  
  // Resource flow
  | 'IMPOSSIBLE_REQUIREMENT'
  | 'GUARANTEED_DEATH'
  | 'RESOURCE_DRAIN'
  | 'UNBOUNDED_LOSS'
  
  // Flag flow
  | 'UNSET_FLAG_CHECK'
  | 'ORPHANED_FLAG'
  | 'FLAG_TYPE_MISMATCH'
  
  // Cross-scene
  | 'UNREACHABLE_ACHIEVEMENT'
  | 'MISSING_FLAG_SOURCE'
  | 'UNUSED_TAG_GATE'
  
  // Balance
  | 'NEGATIVE_EV_TRAP'
  | 'HIGH_VARIANCE_OUTCOME'
  | 'ASYMMETRIC_RISK_REWARD';
```

## 5. Diagnostic Examples

### 5.1 Impossible Requirement

```
error[IMPOSSIBLE_REQUIREMENT]: Choice requirement can never be satisfied
  --> src/content/scenelets/scenes/port/gambling.scene:22
   |
22 | * [Enter the competition] { credits >= 50 }
   |                            ^^^^^^^^^^^^^^
   |
   = note: At this point, credits are in range [0, 30]
   = note: Path to reach this state:
           intro -> watch -> bet_underdog (credits -= 30)
   = help: Either increase credits on a preceding path or lower the requirement
```

### 5.2 Guaranteed Death Path

```
warning[GUARANTEED_DEATH]: Path leads to certain destruction
  --> src/content/scenelets/scenes/journey/sera_contact.scene
   |
   = Path: intro -> contain -> "Keep fighting"
   = Hull after path: [-3, 7] (includes ≤ 0)
   = note: Player entering with hull < 15 will die on this path
   = help: Consider adding a hull check or escape route
```

### 5.3 Cross-Scene Flag Issue

```
warning[MISSING_FLAG_SOURCE]: Flag checked but never set in any scene
   |
   = Flag: `cataclysm_deep_data`
   = Checked in: port_cataclysm_clue.scene:81
   = No scene sets this flag
   = help: Either add a scene that sets this flag, or remove the check
```

### 5.4 Dead Choice

```
info[DEAD_CHOICE]: Choice is unreachable due to contradictory requirements
  --> src/content/scenelets/scenes/port/compact_audit.scene:64
   |
64 | * [Continue stalling] { cargo.contraband }
   |                         ^^^^^^^^^^^^^^^^
   |
   = note: This passage is only reachable if contraband was already jettisoned
   = note: Preceding path: intro -> inspection -> panic_jettison
   = Contraband status at this point: definitely_absent
```

## 6. Statistics Output

```
Scene Analysis Report: journey_sera_contact
═══════════════════════════════════════════

Structural Analysis:
  Passages: 4 (intro, vent, contain, assess)
  Choices: 8
  Terminal states: 5
  Cycles: none
  Max path length: 3

Resource Flow:
  ┌─────────────┬─────────────┬─────────────┐
  │ Resource    │ Min Outcome │ Max Outcome │
  ├─────────────┼─────────────┼─────────────┤
  │ credits     │ 0           │ 0           │
  │ hull        │ -15 (!)     │ 0           │
  │ integrity   │ -10         │ 0           │
  │ cargo       │ WIPED       │ +sera_samp  │
  └─────────────┴─────────────┴─────────────┘

  ⚠ Hull can go negative (guaranteed death on some paths)

Flag Effects:
  Sets: sera_survived (all terminal paths)

Expected Value (uniform choice distribution):
  E[hull_damage] = 7.8
  E[cargo_loss] = 0.8 (80% chance of cargo wipe)

Risk Profile: HIGH
  - 3/5 paths result in total cargo loss
  - 1/5 paths deal significant hull damage
  - This is a "dangerous but fair" encounter
```

## 7. Implementation Phases

### Phase 1: Core Infrastructure
- [ ] IR types and scene-to-IR conversion
- [ ] CFG construction
- [ ] Basic reachability (BFS/DFS)
- [ ] Diagnostic framework

### Phase 2: Abstract Interpretation
- [ ] Interval domain for resources
- [ ] Flag lattice
- [ ] Transfer functions
- [ ] Fixed-point iteration (with widening for loops)

### Phase 3: Intra-Scene Analysis
- [ ] Dead passage detection
- [ ] Dead choice detection
- [ ] Termination analysis
- [ ] Resource bound computation
- [ ] Outcome enumeration

### Phase 4: Cross-Scene Analysis
- [ ] Flag set/check global analysis
- [ ] Achievement verification
- [ ] Tag coverage analysis

### Phase 5: Z3 Backend (Optional)
- [ ] SMT encoding of scene semantics
- [ ] Precise path feasibility
- [ ] Counter-example generation
- [ ] Property verification

### Phase 6: Balance Analysis
- [ ] Expected value computation
- [ ] Variance analysis
- [ ] Risk profiling
- [ ] Scene difficulty scoring

## 8. Open Questions

1. **Initial state assumptions**: What should we assume about the player's state when entering a scene? 
   - Option A: Completely unknown (⊤ for everything) - most conservative
   - Option B: Use scene requirements as lower bounds
   - Option C: Configurable "typical player" profile

2. **Z3 dependency**: Are you okay adding z3-solver as an optional dependency? It's ~50MB but enables precise queries. We can make it opt-in (`--z3` flag).

3. **Cross-scene ordering**: Scenes are selected probabilistically, so strict "scene A before scene B" analysis is hard. Should we:
   - Assume any scene can appear in any order?
   - Use cooldowns/requirements to infer partial orderings?
   - Just flag potential issues without proving them?

4. **Prose analysis**: Current analyzer checks for biological references. Want more sophisticated NLP? (Tone consistency, name consistency, lore-checking against a glossary file?)

5. **Output format**: JSON for tool integration? SARIF for IDE support? Plain text? All of the above?
