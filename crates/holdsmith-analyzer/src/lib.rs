//! Holdsmith scene analyzer.
//!
//! This crate provides static analysis tools for Holdsmith scenes:
//! - Control flow graph (CFG) construction
//! - Script analysis (reads, writes, RNG detection)
//! - Symbolic execution with Z3 (optional, requires `z3` feature)
//! - Scene-level analysis (reachability, dead code detection)
//! - Structured diagnostics

mod analyzer;
mod cfg;
mod diagnostics;

#[cfg(feature = "z3")]
mod symbolic;

pub use analyzer::{
    analyze_cfg, analyze_scene, analyze_scenes, analyze_scenes_parallel, AnalysisResult,
    ImpossibleChoice, ReadLocation, SceneStats, StateRefKind, TautologicalChoice, UninitializedRead,
};
pub use cfg::{build_cfg, CfgEdge, CfgError, CfgLocation, CfgNode, CfgTarget, EdgeId, NodeId, SceneCfg};
pub use diagnostics::{
    generate_diagnostics, Diagnostic, DiagnosticCode, DiagnosticLocation, DiagnosticSummary, Severity,
};

#[cfg(feature = "z3")]
pub use symbolic::{create_context, SymbolicState};
