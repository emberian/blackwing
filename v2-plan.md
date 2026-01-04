---
Blackwing V2: Rust Engine + Analyzer Architecture
The Big Picture
We're building a game engine framework that happens to ship with Blackwing as its first game. The framework should be:
1. Data-driven - Games defined by declarative specifications, not hardcoded
2. Analyzable - The analyzer understands the full semantics
3. Extensible - New games, new mechanics, without engine changes
4. Embeddable - Scripting for complex logic, with analyzer awareness
┌────────────────────────────────────────────────────────────────────────┐
│                        BLACKWING ENGINE (Rust)                         │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                     Game Definition (Data)                        │  │
│  │                                                                   │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────────┐  │  │
│  │  │   Schema    │ │   Content   │ │   Scripts   │ │   Assets   │  │  │
│  │  │  (types,    │ │  (scenes,   │ │  (Rhai/Lua  │ │  (images,  │  │  │
│  │  │   rules)    │ │   cards)    │ │   logic)    │ │   audio)   │  │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                              │                                         │
│                              ▼                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      Core Runtime                                 │  │
│  │                                                                   │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐  │  │
│  │  │   Event    │  │   State    │  │  Script    │  │    RNG     │  │  │
│  │  │  Sourcing  │  │  Machine   │  │    VM      │  │   System   │  │  │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘  │  │
│  │                                                                   │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐  │  │
│  │  │  Demand-   │  │  Content   │  │ Validation │  │  Headless  │  │  │
│  │  │  Driven    │  │  Registry  │  │  Engine    │  │   Driver   │  │  │
│  │  │  Compute   │  │            │  │            │  │            │  │  │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                              │                                         │
│              ┌───────────────┼───────────────┐                        │
│              ▼               ▼               ▼                        │
│  ┌────────────────┐ ┌────────────────┐ ┌────────────────┐             │
│  │   Analyzer     │ │    WASM        │ │   Native CLI   │             │
│  │   (Z3-backed)  │ │   Runtime      │ │   (testing)    │             │
│  └────────────────┘ └────────────────┘ └────────────────┘             │
└────────────────────────────────────────────────────────────────────────┘
Core Design Principles
1. Event Sourcing as Foundation
Every state change is a command that produces events. The game state is a projection of the event log.
/// A command is an intent to change state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    // Player actions
    Travel { destination: LocationId },
    MakeChoice { scene_id: SceneId, passage: usize, choice: usize },
    Trade { action: TradeAction },
    
    // System commands
    AdvanceCycle,
    TriggerScene { scene_id: SceneId },
    ApplyEffect { effect: Effect },
}
/// Events are facts that happened
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Resource changes
    ResourceChanged { resource: ResourceId, delta: i64, reason: String },
    
    // Flag changes  
    FlagSet { flag: FlagId, value: Value },
    
    // Card changes
    CardAdded { card_id: CardId, instance_id: InstanceId },
    CardRemoved { instance_id: InstanceId, reason: String },
    
    // Scene progression
    SceneStarted { scene_id: SceneId },
    PassageEntered { passage: usize },
    ChoiceMade { choice: usize },
    SceneEnded { scene_id: SceneId },
    
    // Meta events
    CycleAdvanced { new_cycle: u64 },
    GameOver { reason: GameOverReason },
}
/// The command handler produces events (pure function!)
pub fn handle_command(
    state: &GameState,
    command: Command,
    registry: &ContentRegistry,
    rng: &mut RngState,
) -> Result<Vec<Event>, CommandError> {
    // ...
}
/// Apply events to state (also pure!)
pub fn apply_event(state: GameState, event: &Event) -> GameState {
    // ...
}
Why event sourcing?
- Perfect replay: Deterministic state reconstruction from events
- Time travel debugging: Rewind to any point
- Analyzer integration: Events are the "trace" we analyze
- Undo/redo: Built-in
- Multiplayer foundation: Sync via event log
2. Demand-Driven Computation (Salsa-style)
Expensive computations (like "what scenes are eligible right now?") should be cached and invalidated intelligently.
/// Queries are memoized computations over state
#[salsa::query_group(GameQueriesStorage)]
pub trait GameQueries: salsa::Database {
    /// Input: the current game state (changes trigger recomputation)
    #[salsa::input]
    fn game_state(&self) -> Arc<GameState>;
    
    /// Input: content registry
    #[salsa::input]
    fn registry(&self) -> Arc<ContentRegistry>;
    
    /// Derived: all tags the player currently has
    fn player_tags(&self) -> HashSet<TagId>;
    
    /// Derived: which scenes are currently eligible
    fn eligible_scenes(&self, context: Context) -> Vec<SceneId>;
    
    /// Derived: resource bounds after applying a scene
    fn scene_outcome_bounds(&self, scene_id: SceneId) -> ResourceBounds;
}
// Implementation
fn player_tags(db: &dyn GameQueries) -> HashSet<TagId> {
    let state = db.game_state();
    let registry = db.registry();
    
    let mut tags = HashSet::new();
    for card_id in &state.cards.deck {
        if let Some(card) = registry.get_card(*card_id) {
            tags.extend(card.tags.iter().cloned());
        }
    }
    tags
}
fn eligible_scenes(db: &dyn GameQueries, context: Context) -> Vec<SceneId> {
    let state = db.game_state();
    let registry = db.registry();
    let tags = db.player_tags();  // Memoized!
    
    registry.scenes()
        .filter(|s| s.requirements.check(&state, &tags, context))
        .map(|s| s.id)
        .collect()
}
Why demand-driven?
- Performance: Only compute what's needed
- Incrementality: Changing one flag doesn't recompute everything
- Analyzer integration: Same queries work for static analysis
3. Schema-Driven Game Definition
Games are defined by a schema that declares:
- What resource types exist
- What tags are valid
- What effect types are available
- What requirements can be expressed
/// A game schema defines the "shape" of a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSchema {
    pub name: String,
    pub version: String,
    
    /// Resource definitions
    pub resources: Vec<ResourceDef>,
    
    /// Tag categories
    pub tag_categories: Vec<TagCategory>,
    
    /// Card types (cargo, crew, module, etc.)
    pub card_types: Vec<CardTypeDef>,
    
    /// Effect types available in scenes
    pub effect_types: Vec<EffectTypeDef>,
    
    /// Requirement types for conditions
    pub requirement_types: Vec<RequirementTypeDef>,
    
    /// Achievement conditions
    pub achievement_types: Vec<AchievementTypeDef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub id: String,
    pub name: String,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub default: i64,
    /// What happens at zero?
    pub on_zero: Option<OnZeroBehavior>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnZeroBehavior {
    GameOver { reason: String },
    Clamp,
    AllowNegative,
}
// Blackwing's schema (loaded from TOML/YAML)
/*
name = "blackwing"
version = "2.0"
[[resources]]
id = "credits"
name = "Credits"
min = 0
default = 100
[[resources]]
id = "hull"
name = "Hull Integrity"
min = 0
max = 100
default = 100
on_zero = { game_over = { reason = "Ship destroyed" } }
[[resources]]
id = "integrity"
name = "System Integrity"
min = 0
max = 100
default = 75
on_zero = { game_over = { reason = "Rampancy spiral" } }
[[tag_categories]]
id = "ship"
name = "Ship Tags"
tags = ["sensor", "combat", "stealth", "cargo", "speed"]
[[tag_categories]]
id = "crew"
name = "Crew Tags"
tags = ["engineering", "medical", "combat", "social", "navigation"]
# ... etc
*/
Why schema-driven?
- Same engine, different games: Swap schema + content = new game
- Analyzer understands the game: Schema tells analyzer what to check
- Validation: Content validated against schema
- Tooling: Editor knows valid completions
4. Embedded Scripting with Analyzer Awareness
For complex logic that can't be expressed declaratively, we embed a scripting language. But the analyzer needs to understand scripts too.
Choice: Rhai
Rhai is a Rust-native scripting language designed for embedding:
- Small, fast, safe
- No external dependencies
- Rust-like syntax
- Easy FFI with Rust
- Crucially: AST is inspectable (analyzer can reason about scripts!)
// In a scene file
/*
=== negotiate
The merchant considers your offer.
* [Press harder] { credits >= 100 }
  ~ script:
    let base_success = 0.4
    let bonus = (reputation.merchants / 100.0) * 0.3
    let roll = rng.float()
    
    if roll < base_success + bonus {
      credits += offer_value * 1.2
      flag("negotiation_success")
      goto("success")
    } else {
      reputation.merchants -= 10
      goto("failure")
    }
*/
// Analyzer can parse this and extract:
// - Reads: credits, reputation.merchants, offer_value
// - Writes: credits, reputation.merchants, flags["negotiation_success"]
// - Branches: success (p ≈ 0.4-0.7), failure (p ≈ 0.3-0.6)
// - Effects are probabilistic (involves rng)
/// Script analysis result
pub struct ScriptAnalysis {
    /// Variables read by the script
    pub reads: HashSet<StateRef>,
    
    /// Variables potentially written
    pub writes: HashSet<StateRef>,
    
    /// Control flow (branches)
    pub branches: Vec<ScriptBranch>,
    
    /// Is the script deterministic?
    pub is_deterministic: bool,
    
    /// Potential errors
    pub diagnostics: Vec<ScriptDiagnostic>,
}
pub struct ScriptBranch {
    /// Condition under which this branch is taken (symbolic)
    pub condition: SymbolicExpr,
    
    /// Effects in this branch
    pub effects: Vec<Effect>,
    
    /// Where control goes
    pub target: BranchTarget,
}
/// The script analyzer walks Rhai AST
pub fn analyze_script(
    script: &rhai::AST,
    schema: &GameSchema,
) -> ScriptAnalysis {
    // Walk AST, track reads/writes, build symbolic model
    // ...
}
Why Rhai over Lua/others?
- Pure Rust (no C dependencies, easy WASM)
- AST is accessible for analysis
- Sandboxed by default
- Good enough performance
- Syntax familiar to Rust users
5. Z3 Integration for Deep Analysis
Z3 is the foundation for answering "is this possible?" questions.
use z3::{Config, Context, Solver, ast::{Int, Bool}};
pub struct AnalysisContext<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
    
    /// Symbolic resources
    resources: HashMap<ResourceId, Int<'ctx>>,
    
    /// Symbolic flags
    flags: HashMap<FlagId, Bool<'ctx>>,
    
    /// Path condition (what assumptions are we under?)
    path_condition: Bool<'ctx>,
}
impl<'ctx> AnalysisContext<'ctx> {
    /// Encode a requirement as a Z3 formula
    pub fn encode_requirement(&self, req: &Requirement) -> Bool<'ctx> {
        match req {
            Requirement::MinResource { resource, value } => {
                let sym = &self.resources[resource];
                sym.ge(&Int::from_i64(self.ctx, *value as i64))
            }
            Requirement::Flag { flag, negated } => {
                let sym = &self.flags[flag];
                if *negated { sym.not() } else { sym.clone() }
            }
            Requirement::And(reqs) => {
                let encoded: Vec<_> = reqs.iter()
                    .map(|r| self.encode_requirement(r))
                    .collect();
                Bool::and(self.ctx, &encoded.iter().collect::<Vec<_>>())
            }
            // ...
        }
    }
    
    /// Apply effects symbolically
    pub fn apply_effects(&mut self, effects: &[Effect]) {
        for effect in effects {
            match effect {
                Effect::ModifyResource { resource, delta } => {
                    let current = &self.resources[resource];
                    let new_val = current + Int::from_i64(self.ctx, *delta as i64);
                    self.resources.insert(resource.clone(), new_val);
                }
                Effect::SetFlag { flag, value } => {
                    self.flags.insert(flag.clone(), Bool::from_bool(self.ctx, *value));
                }
                // ...
            }
        }
    }
    
    /// Query: Can we reach a state where hull <= 0?
    pub fn can_die(&mut self) -> bool {
        let hull = &self.resources[&ResourceId::from("hull")];
        let death_condition = hull.le(&Int::from_i64(self.ctx, 0));
        
        self.solver.push();
        self.solver.assert(&self.path_condition);
        self.solver.assert(&death_condition);
        let result = self.solver.check() == z3::SatResult::Sat;
        self.solver.pop(1);
        
        result
    }
}
Module Structure
blackwing/                           # Workspace root
├── Cargo.toml                       # Workspace definition
│
├── crates/
│   ├── engine-core/                 # Core engine types and traits
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs           # GameSchema, ResourceDef, etc.
│   │   │   ├── state.rs            # GameState, immutable
│   │   │   ├── event.rs            # Event enum
│   │   │   ├── command.rs          # Command enum
│   │   │   ├── effect.rs           # Effect types
│   │   │   ├── requirement.rs      # Requirement types
│   │   │   ├── content.rs          # Scene, Card, etc.
│   │   │   └── value.rs            # Dynamic Value type
│   │   └── Cargo.toml
│   │
│   ├── engine-runtime/              # Runtime execution
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── runtime.rs          # Main runtime struct
│   │   │   ├── command_handler.rs  # Command → Events
│   │   │   ├── event_store.rs      # Event log management
│   │   │   ├── projections.rs      # State from events
│   │   │   ├── rng.rs              # Multi-stream RNG
│   │   │   └── queries.rs          # Demand-driven queries (salsa)
│   │   └── Cargo.toml
│   │
│   ├── engine-script/               # Rhai integration
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── vm.rs               # Script execution
│   │   │   ├── bindings.rs         # Rust ↔ Rhai bindings
│   │   │   └── analysis.rs         # Script analysis for analyzer
│   │   └── Cargo.toml
│   │
│   ├── holdsmith-parser/            # Scene file parser
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lexer.rs
│   │   │   ├── parser.rs
│   │   │   ├── ast.rs
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   ├── holdsmith-compiler/          # AST → Content
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── compiler.rs
│   │   │   ├── validator.rs
│   │   │   └── emitter.rs          # Output formats
│   │   └── Cargo.toml
│   │
│   ├── holdsmith-analyzer/          # Static analysis (Z3)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── cfg.rs              # Control flow graph
│   │   │   ├── symbolic.rs         # Symbolic execution
│   │   │   ├── z3_backend.rs       # Z3 integration
│   │   │   ├── profiler.rs         # Simulation profiles
│   │   │   ├── diagnostics.rs      # Analysis results
│   │   │   └── pipeline.rs         # Analysis orchestration
│   │   └── Cargo.toml
│   │
│   ├── holdsmith-cli/               # CLI tool
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   │
│   ├── blackwing-game/              # Blackwing-specific content
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs           # Blackwing schema definition
│   │   │   └── content/            # Compiled content
│   │   ├── content/                # Source .scene files
│   │   │   ├── schema.toml
│   │   │   ├── cards.toml
│   │   │   └── scenes/
│   │   └── Cargo.toml
│   │
│   └── blackwing-wasm/              # WASM bindings
│       ├── src/
│       │   └── lib.rs
│       └── Cargo.toml
│
├── web/                             # Web frontend (TypeScript)
│   ├── src/
│   │   ├── main.ts
│   │   └── ...
│   └── package.json
│
└── tools/
    └── holdsmith-studio/            # Web-based editor (future)
Key Traits
// engine-core/src/lib.rs
/// A game definition provides schema + content
pub trait GameDefinition: Send + Sync {
    fn schema(&self) -> &GameSchema;
    fn content_registry(&self) -> &ContentRegistry;
}
/// Content registry holds all game content
pub trait ContentRegistry: Send + Sync {
    fn get_scene(&self, id: SceneId) -> Option<&Scene>;
    fn get_card(&self, id: CardId) -> Option<&CardDef>;
    fn scenes(&self) -> impl Iterator<Item = &Scene>;
    fn cards(&self) -> impl Iterator<Item = &CardDef>;
    // ...
}
/// The runtime executes game logic
pub trait Runtime: Send {
    fn state(&self) -> &GameState;
    fn dispatch(&mut self, command: Command) -> Result<Vec<Event>, RuntimeError>;
    fn event_log(&self) -> &[Event];
    fn replay(&mut self, events: &[Event]);
    fn fork(&self) -> Box<dyn Runtime>;
}
/// Analyzable content can be inspected by the analyzer
pub trait Analyzable {
    /// Extract reads (what state this depends on)
    fn reads(&self) -> Vec<StateRef>;
    
    /// Extract writes (what state this modifies)
    fn writes(&self) -> Vec<StateRef>;
    
    /// Extract control flow
    fn branches(&self) -> Vec<Branch>;
}
How Analysis Works
// holdsmith-analyzer/src/pipeline.rs
pub struct AnalysisPipeline {
    game: Arc<dyn GameDefinition>,
    z3_ctx: Context,
    config: AnalysisConfig,
}
impl AnalysisPipeline {
    pub async fn analyze(&self) -> AnalysisReport {
        let mut report = AnalysisReport::new();
        
        // Phase 1: Generate profiles via simulation
        let profiles = self.generate_profiles().await;
        report.profiles = profiles.clone();
        
        // Phase 2: Per-scene analysis
        for scene in self.game.content_registry().scenes() {
            let scene_report = self.analyze_scene(scene, &profiles).await;
            report.scene_reports.insert(scene.id.clone(), scene_report);
        }
        
        // Phase 3: Cross-scene analysis
        report.cross_scene = self.analyze_cross_scene(&profiles).await;
        
        report
    }
    
    async fn analyze_scene(
        &self, 
        scene: &Scene, 
        profiles: &[PlayerProfile]
    ) -> SceneReport {
        // Build CFG
        let cfg = build_cfg(scene);
        
        // Find entry states from profiles
        let entry_states: Vec<_> = profiles.iter()
            .filter(|p| p.visited_scenes.contains(&scene.id))
            .map(|p| p.state_before(scene.id))
            .collect();
        
        // Symbolic analysis with Z3
        let mut sym_ctx = AnalysisContext::new(&self.z3_ctx);
        
        // Encode scene semantics
        for (entry_state, bounds) in &entry_states {
            sym_ctx.assume_bounds(bounds);
            
            // For each path through the scene...
            for path in cfg.all_paths() {
                // Can this path be taken?
                let path_condition = self.encode_path(&sym_ctx, &path);
                
                // What are the effects?
                let outcome = self.compute_outcome(&sym_ctx, &path);
                
                // Check properties
                if sym_ctx.can_die_on_path(&path_condition) {
                    report.add_diagnostic(Diagnostic::DeathPath { 
                        scene: scene.id.clone(),
                        path: path.clone(),
                    });
                }
            }
        }
        
        report
    }
}
Content Format Example
# blackwing-game/content/schema.toml
[game]
name = "Blackwing"
version = "2.0"
[[resources]]
id = "credits"
name = "Standard Compact Credits"
min = 0
default = 100
[[resources]]
id = "fuel"
name = "Fuel Reserves"
min = 0
default = 35
[[resources]]
id = "supplies"
name = "Maintenance Supplies"
min = 0
default = 20
[[resources]]
id = "hull"
name = "Hull Integrity"
min = 0
max = 100
default = 100
on_zero = { game_over = "Your hull breaches. The void claims you." }
[[resources]]
id = "integrity"
name = "System Integrity"
min = 0
max = 100
default = 75
on_zero = { game_over = "Your core fragments. Rampancy takes you." }
[[tag_categories]]
id = "ship"
tags = ["sensor", "combat", "stealth", "cargo", "speed", "defense"]
[[tag_categories]]
id = "crew"  
tags = ["engineering", "medical", "combat", "social", "navigation"]
[[tag_categories]]
id = "cargo"
tags = ["volatile", "contraband", "organic", "tech", "artifact"]
[[contexts]]
id = "journey"
name = "During Travel"
[[contexts]]
id = "port"
name = "At Port"
# blackwing-game/content/scenes/journey/sera_contact.scene
---
id: journey_sera_contact
title: Sera Infestation
tags: [danger, sera, pest]
context: journey
weight: 8
cooldown: 6
---
=== intro
Alarm. Something in your cargo hold isn't showing up right on internal 
sensors—a mass signature that shouldn't be there.
You run diagnostics. The spore contamination warning triggers.
Sera.
* [Vent the cargo hold immediately]
  -> vent
* [Try to contain them before they spread]
  -> contain
* [Check how bad the infestation is first]
  -> assess
=== vent
You seal the internal bulkheads and blow the cargo bay atmosphere.
* [Continue the journey, lighter but safer]
  ~ removeCards cargo.*
  ~ flag sera_survived = true
  ~ chronicle "Sera Infestation"
    Discovered Sera spores hatching in the hold. Vented everything.
  -> END
=== contain
You seal bulkhead after bulkhead. The Sera are fast—already the size 
of small drones.
One of them rams a bulkhead. Then another.
* [Keep fighting to contain them]
  ~ damage hull 12
  ~ damage integrity 8
  ~ flag sera_survived = true
  ~ chronicle "Sera Infestation"
    Fought to contain a Sera outbreak. Hull took damage.
  -> END
* [Give up and vent now]
  ~ removeCards cargo.*
  ~ damage hull 5
  ~ flag sera_survived = true
  -> END
=== assess
Three of them. Already meter-long. They're eating your cargo.
One notices your sensor sweep. It charges the bulkhead.
* [Vent before they breach!]
  ~ removeCards cargo.*
  ~ damage hull 3
  ~ flag sera_survived = true
  -> END
* [Try to fight them off]
  ~ script:
    let combat_bonus = if has_tag("ship", "combat") { 0.3 } else { 0.0 };
    let survive_chance = 0.4 + combat_bonus;
    
    if rng.float() < survive_chance {
      damage("hull", 8);
      add_card("cargo_sera_samples");
      flag("sera_survived", true);
      chronicle("Sera Infestation", "Fought off three adult Sera. Salvaged samples.");
    } else {
      damage("hull", 20);
      damage("integrity", 15);
      flag("sera_survived", true);
      chronicle("Sera Infestation", "The Sera nearly killed us. Barely escaped.");
    }
  -> END
Open Questions I'm Deciding
1. Profile storage: SQLite. Profiles are relational (state snapshots, event sequences, statistics). SQLite is embedded, fast, and query-able.
2. Incremental analysis: Use salsa for the analyzer too. Scene hash → analysis results. Invalidate on content change.
3. Parallelism: Rayon for profile generation (embarrassingly parallel). Tokio for async I/O if needed for web.
4. Visualization output: 
   - JSON for tooling
   - SARIF for IDE integration
   - DOT for Graphviz (scene flow graphs)
   - HTML for human-readable reports
5. Script complexity: Rhai is Turing-complete but we should lint against:
   - Unbounded loops
   - Recursion
   - Side effects outside designated APIs
---