//! Holdsmith CLI - Scene authoring and analysis tools.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use engine_core::Scene;
use holdsmith_analyzer::{
    analyze_cross_scene, analyze_scenes_parallel, generate_diagnostics, Severity,
};
use holdsmith_compiler::compile;
use holdsmith_parser::parse;

#[derive(Parser)]
#[command(name = "holdsmith")]
#[command(about = "Holdsmith scene authoring and analysis CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze scenes for issues (reachability, impossible choices, etc.)
    Analyze(AnalyzeArgs),

    /// Profile scenes with simulation runs
    Profile(ProfileArgs),

    /// Check cross-scene consistency
    Check(CheckArgs),

    /// Compile .scene files to JSON
    Compile(CompileArgs),
}

#[derive(clap::Args)]
struct AnalyzeArgs {
    /// Path to scene file or directory containing scenes
    #[arg(short, long)]
    path: PathBuf,

    /// Output format
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,

    /// Enable deep analysis (Z3-based condition checking)
    #[arg(long)]
    deep: bool,

    /// Include cross-scene analysis
    #[arg(long)]
    cross_scene: bool,

    /// Only show errors (no warnings/info)
    #[arg(long)]
    errors_only: bool,
}

#[derive(clap::Args)]
struct ProfileArgs {
    /// Path to scene file
    #[arg(short, long)]
    path: PathBuf,

    /// Number of simulation runs
    #[arg(short, long, default_value = "100")]
    runs: usize,

    /// Exploration strategy
    #[arg(short, long, default_value = "random")]
    strategy: Strategy,

    /// Random seed for reproducibility
    #[arg(long, default_value = "42")]
    seed: u64,

    /// Output format
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,
}

#[derive(clap::Args)]
struct CheckArgs {
    /// Path to directory containing scenes
    #[arg(short, long)]
    path: PathBuf,

    /// Output format
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,
}

#[derive(clap::Args)]
struct CompileArgs {
    /// Path to scene file
    #[arg(short, long)]
    path: PathBuf,

    /// Output path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print JSON output
    #[arg(long)]
    pretty: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Strategy {
    Random,
    First,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze(args) => cmd_analyze(args),
        Commands::Profile(args) => cmd_profile(args),
        Commands::Check(args) => cmd_check(args),
        Commands::Compile(args) => cmd_compile(args),
    }
}

fn cmd_analyze(args: AnalyzeArgs) -> Result<()> {
    let scenes = load_scenes(&args.path)?;

    if scenes.is_empty() {
        println!("No scenes found.");
        return Ok(());
    }

    // Analyze scenes
    let results = analyze_scenes_parallel(&scenes);

    // Generate diagnostics
    let all_diagnostics: Vec<_> = results
        .iter()
        .flat_map(|r| generate_diagnostics(r))
        .collect();

    // Filter by severity if requested
    let diagnostics: Vec<_> = if args.errors_only {
        all_diagnostics
            .into_iter()
            .filter(|d| matches!(d.severity, Severity::Error))
            .collect()
    } else {
        all_diagnostics
    };

    // Cross-scene analysis
    let cross_scene = if args.cross_scene {
        Some(analyze_cross_scene(&scenes))
    } else {
        None
    };

    match args.format {
        OutputFormat::Text => {
            println!("Analyzed {} scene(s):\n", scenes.len());

            // Per-scene results
            for result in &results {
                println!("Scene: {}", result.scene_id);
                println!(
                    "  Reachable passages: {}",
                    result.reachable_passages.len()
                );

                if !result.unreachable_passages.is_empty() {
                    println!(
                        "  Unreachable passages: {:?}",
                        result.unreachable_passages
                    );
                }

                if !result.impossible_choices.is_empty() {
                    println!("  Impossible choices: {}", result.impossible_choices.len());
                    for ic in &result.impossible_choices {
                        println!(
                            "    - Passage {}, choice {}: \"{}\"",
                            ic.passage_index, ic.choice_index, ic.choice_text
                        );
                    }
                }

                if !result.tautological_choices.is_empty() {
                    println!(
                        "  Tautological choices: {}",
                        result.tautological_choices.len()
                    );
                }

                if result.uses_rng {
                    println!("  Uses RNG: yes");
                }

                println!();
            }

            // Diagnostics
            if !diagnostics.is_empty() {
                println!("Diagnostics ({}):", diagnostics.len());
                for diag in &diagnostics {
                    let severity = match diag.severity {
                        Severity::Error => "ERROR",
                        Severity::Warning => "WARN",
                        Severity::Info => "INFO",
                    };
                    println!("  [{}] {:?}: {}", severity, diag.code, diag.message);
                    if let Some(ref help) = diag.help {
                        println!("         hint: {}", help);
                    }
                }
                println!();
            }

            // Cross-scene results
            if let Some(ref cs) = cross_scene {
                println!("Cross-scene analysis:");
                println!("  State variables: {}", cs.state_vars.len());

                if !cs.use_before_set.is_empty() {
                    println!("  Use-before-set issues: {}", cs.use_before_set.len());
                    for issue in &cs.use_before_set {
                        println!("    - {:?} '{}' read but never written", issue.kind, issue.var_name);
                    }
                }

                if !cs.unused_writes.is_empty() {
                    println!("  Unused writes: {}", cs.unused_writes.len());
                    for issue in &cs.unused_writes {
                        println!("    - {:?} '{}' written but never read", issue.kind, issue.var_name);
                    }
                }

                // Check for cycles
                let cycles = cs.dependencies.find_cycles();
                if !cycles.is_empty() {
                    println!("  Dependency cycles: {}", cycles.len());
                    for cycle in &cycles {
                        println!("    - {}", cycle.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" -> "));
                    }
                }
            }
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "scenes_analyzed": scenes.len(),
                "results": results.iter().map(|r| serde_json::json!({
                    "scene_id": r.scene_id,
                    "reachable_passages": r.reachable_passages.len(),
                    "unreachable_passages": r.unreachable_passages,
                    "impossible_choices": r.impossible_choices.len(),
                    "tautological_choices": r.tautological_choices.len(),
                    "uses_rng": r.uses_rng,
                })).collect::<Vec<_>>(),
                "diagnostics": diagnostics.iter().map(|d| serde_json::json!({
                    "severity": format!("{:?}", d.severity),
                    "code": format!("{:?}", d.code),
                    "message": d.message,
                })).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    // Exit with error if there are errors
    let error_count = diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Error))
        .count();
    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn cmd_profile(args: ProfileArgs) -> Result<()> {
    let scenes = load_scenes(&args.path)?;

    if scenes.is_empty() {
        anyhow::bail!("No scenes found at {:?}", args.path);
    }

    // For now, just show that profiling would run
    // Full profiler integration would require a GameSchema and ContentRegistry

    match args.format {
        OutputFormat::Text => {
            println!("Profile simulation:");
            println!("  Scenes: {}", scenes.len());
            println!("  Runs: {}", args.runs);
            println!("  Strategy: {:?}", args.strategy);
            println!("  Seed: {}", args.seed);
            println!();
            println!("Note: Full profiling requires a schema. Use the library API for complete simulation.");
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": "not_fully_implemented",
                "scenes": scenes.len(),
                "runs": args.runs,
                "strategy": format!("{:?}", args.strategy),
                "seed": args.seed,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

fn cmd_check(args: CheckArgs) -> Result<()> {
    let scenes = load_scenes(&args.path)?;

    if scenes.is_empty() {
        println!("No scenes found.");
        return Ok(());
    }

    let analysis = analyze_cross_scene(&scenes);

    match args.format {
        OutputFormat::Text => {
            println!("Cross-scene consistency check:");
            println!("  Scenes analyzed: {}", analysis.scene_count);
            println!("  State variables tracked: {}", analysis.state_vars.len());
            println!();

            let mut issues = 0;

            if !analysis.use_before_set.is_empty() {
                println!("Use-before-set issues ({}):", analysis.use_before_set.len());
                for issue in &analysis.use_before_set {
                    println!(
                        "  {:?} '{}' is read in {:?} but never written",
                        issue.kind, issue.var_name, issue.read_scenes
                    );
                }
                issues += analysis.use_before_set.len();
                println!();
            }

            if !analysis.unused_writes.is_empty() {
                println!("Unused writes ({}):", analysis.unused_writes.len());
                for issue in &analysis.unused_writes {
                    println!(
                        "  {:?} '{}' is written in {:?} but never read",
                        issue.kind, issue.var_name, issue.write_scenes
                    );
                }
                issues += analysis.unused_writes.len();
                println!();
            }

            // Check dependency graph for cycles
            let cycles = analysis.dependencies.find_cycles();
            if !cycles.is_empty() {
                println!("Dependency cycles ({}):", cycles.len());
                for cycle in &cycles {
                    println!(
                        "  {}",
                        cycle
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(" -> ")
                    );
                }
                println!();
            }

            if issues == 0 && cycles.is_empty() {
                println!("No issues found.");
            } else {
                std::process::exit(1);
            }
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "scene_count": analysis.scene_count,
                "state_vars": analysis.state_vars.len(),
                "use_before_set": analysis.use_before_set.iter().map(|i| serde_json::json!({
                    "kind": format!("{:?}", i.kind),
                    "var_name": i.var_name,
                    "read_scenes": i.read_scenes,
                })).collect::<Vec<_>>(),
                "unused_writes": analysis.unused_writes.iter().map(|i| serde_json::json!({
                    "kind": format!("{:?}", i.kind),
                    "var_name": i.var_name,
                    "write_scenes": i.write_scenes,
                })).collect::<Vec<_>>(),
                "dependency_cycles": analysis.dependencies.find_cycles(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

fn cmd_compile(args: CompileArgs) -> Result<()> {
    let source = std::fs::read_to_string(&args.path)
        .with_context(|| format!("Failed to read {:?}", args.path))?;

    let filename = args.path.to_string_lossy();
    let parsed = parse(&source, &filename).map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;
    let scene = compile(parsed).map_err(|e| anyhow::anyhow!("Compile error: {:?}", e))?;

    let json = if args.pretty {
        serde_json::to_string_pretty(&scene)?
    } else {
        serde_json::to_string(&scene)?
    };

    match args.output {
        Some(path) => {
            std::fs::write(&path, &json)
                .with_context(|| format!("Failed to write {:?}", path))?;
            println!("Compiled to {:?}", path);
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

fn load_scenes(path: &PathBuf) -> Result<Vec<Scene>> {
    let mut scenes = Vec::new();

    if path.is_file() {
        let scene = load_scene_file(path)?;
        scenes.push(scene);
    } else if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "scene").unwrap_or(false) {
                match load_scene_file(&path) {
                    Ok(scene) => scenes.push(scene),
                    Err(e) => {
                        eprintln!("Warning: Failed to load {:?}: {}", path, e);
                    }
                }
            }
        }
    } else {
        anyhow::bail!("Path {:?} does not exist", path);
    }

    Ok(scenes)
}

fn load_scene_file(path: &PathBuf) -> Result<Scene> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {:?}", path))?;

    let filename = path.to_string_lossy();
    let parsed = parse(&source, &filename).map_err(|e| anyhow::anyhow!("Parse error in {:?}: {:?}", path, e))?;
    let scene = compile(parsed).map_err(|e| anyhow::anyhow!("Compile error in {:?}: {:?}", path, e))?;

    Ok(scene)
}
