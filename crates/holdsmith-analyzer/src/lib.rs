//! Holdsmith scene analyzer.
//!
//! This crate provides static analysis tools for Holdsmith scenes:
//! - Control flow graph (CFG) construction
//! - Script analysis (reads, writes, RNG detection)
//! - Symbolic execution with Z3 (optional, requires `z3` feature)
//! - Condition encoding to Z3 (optional, requires `z3` feature)
//! - Scene-level analysis (reachability, dead code detection)
//! - Structured diagnostics

mod analyzer;
mod cfg;
mod cross_scene;
mod diagnostics;
mod profiler;

#[cfg(feature = "z3")]
mod condition_encoder;
#[cfg(feature = "z3")]
mod path_analysis;
#[cfg(feature = "z3")]
mod symbolic;

pub use analyzer::{
    analyze_cfg, analyze_scene, analyze_scenes, analyze_scenes_parallel, AnalysisResult,
    ImpossibleChoice, ReadLocation, SceneStats, StateRefKind, TautologicalChoice, UninitializedRead,
};
pub use cfg::{build_cfg, CfgEdge, CfgError, CfgLocation, CfgNode, CfgTarget, EdgeId, NodeId, SceneCfg};
pub use cross_scene::{
    analyze_cross_scene, CrossSceneAnalysis, CycleDetected, SceneDependencyGraph, SceneLocation,
    StateVarInfo, StateVarKind, UnusedWrite, UseBeforeSet,
};
pub use diagnostics::{
    generate_diagnostics, Diagnostic, DiagnosticCode, DiagnosticLocation, DiagnosticSummary, Severity,
};
pub use profiler::{
    ChoiceInfo, ChoiceTarget, Distribution, ExplorationStrategy, FirstAvailableStrategy,
    PathStrategy, ProfileStatistics, Profiler, RandomStrategy, SceneOutcome, SimulationState,
};

#[cfg(feature = "z3")]
pub use condition_encoder::{ConditionEncoder, EncodedCondition};
#[cfg(feature = "z3")]
pub use path_analysis::{
    check_path_feasibility, DeathPath, ExecutionPath, PathAnalysisResult, PathAnalyzer,
    PathFeasibility, PathStep, ResourceBounds, RngWarning,
};
#[cfg(feature = "z3")]
pub use symbolic::{create_context, SymbolicState};
