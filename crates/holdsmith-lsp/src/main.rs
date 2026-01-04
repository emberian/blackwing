//! Holdsmith Language Server binary.
//!
//! Run modes:
//! - `holdsmith-lsp` - Start LSP server (stdio)
//! - `holdsmith-lsp --check <file>` - Check a file and print diagnostics
//! - `holdsmith-lsp --version` - Print version

use std::env;
use std::fs;
use std::process::ExitCode;

use holdsmith_analyzer::{analyze_scene, generate_diagnostics, Severity};
use holdsmith_compiler::compile;
use holdsmith_lsp::create_service;
use holdsmith_parser::parse;
use tower_lsp::Server;
use tracing_subscriber::EnvFilter;

fn print_usage() {
    eprintln!("Usage: holdsmith-lsp [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --check <file>   Check a .scene file and print diagnostics");
    eprintln!("  --version        Print version information");
    eprintln!("  --help           Print this help message");
    eprintln!();
    eprintln!("Without options, starts the LSP server on stdio.");
    eprintln!();
    eprintln!("Environment:");
    eprintln!("  RUST_LOG=holdsmith_lsp=debug   Enable debug logging to stderr");
}

fn check_file(path: &str) -> ExitCode {
    eprintln!("Checking: {}", path);

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: Failed to read file: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Parse
    let ast = match parse(&content, path) {
        Ok(ast) => {
            eprintln!("  parse: OK ({} passages)", ast.passages.len());
            ast
        }
        Err(e) => {
            eprintln!("  parse: FAILED");
            eprintln!();
            eprintln!("Parse error at byte {}..{}:", e.span().start, e.span().end);
            eprintln!("  {}", e);
            eprintln!();

            // Show context around the error
            let start = e.span().start.saturating_sub(20);
            let end = (e.span().end + 20).min(content.len());
            if let Some(snippet) = content.get(start..end) {
                eprintln!("Context:");
                eprintln!("  ...{}...", snippet.replace('\n', "\\n"));
            }

            return ExitCode::FAILURE;
        }
    };

    // Compile
    let scene = match compile(ast) {
        Ok(scene) => {
            eprintln!("  compile: OK (id: {})", scene.id);
            scene
        }
        Err(e) => {
            eprintln!("  compile: FAILED");
            eprintln!();
            eprintln!("Compile error: {}", e);
            if let Some(span) = e.span() {
                eprintln!("  at byte {}..{}", span.start, span.end);
            }
            return ExitCode::FAILURE;
        }
    };

    // Analyze
    let analysis = analyze_scene(&scene);
    let diagnostics = generate_diagnostics(&analysis);

    if diagnostics.is_empty() {
        eprintln!("  analyze: OK (no issues)");
        eprintln!();
        eprintln!("All checks passed!");
        ExitCode::SUCCESS
    } else {
        let errors = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
        let warnings = diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
        eprintln!("  analyze: {} error(s), {} warning(s)", errors, warnings);
        eprintln!();

        for diag in &diagnostics {
            let level = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            };
            eprintln!("{}: [{}] {}", level, diag.code.as_str(), diag.message);

            if let Some(ref details) = diag.details {
                for line in details.lines() {
                    eprintln!("  {}", line);
                }
            }

            if let Some(ref help) = diag.help {
                eprintln!("  hint: {}", help);
            }
            eprintln!();
        }

        if errors > 0 {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    // Handle CLI arguments
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            "--version" | "-V" => {
                eprintln!("holdsmith-lsp {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--check" => {
                if args.len() < 3 {
                    eprintln!("error: --check requires a file path");
                    eprintln!();
                    print_usage();
                    return ExitCode::FAILURE;
                }
                return check_file(&args[2]);
            }
            arg => {
                eprintln!("error: Unknown option: {}", arg);
                eprintln!();
                print_usage();
                return ExitCode::FAILURE;
            }
        }
    }

    // Initialize tracing for LSP mode
    // Users can set RUST_LOG=holdsmith_lsp=debug for verbose output
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("holdsmith_lsp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting holdsmith-lsp v{}", env!("CARGO_PKG_VERSION"));

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = create_service();
    Server::new(stdin, stdout, socket).serve(service).await;

    ExitCode::SUCCESS
}
