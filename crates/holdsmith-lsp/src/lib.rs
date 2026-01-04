//! Holdsmith Language Server
//!
//! Provides IDE features for `.scene` files:
//! - Diagnostics (parse errors, compile errors, analyzer warnings)
//! - Real-time feedback as you type

use std::collections::HashMap;
use std::sync::Arc;

use holdsmith_analyzer::{analyze_scene, generate_diagnostics, Severity};
use holdsmith_compiler::compile;
use holdsmith_parser::{parse, Span};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService};
use tracing::{debug, error, info, warn};

/// Document state tracked by the server.
pub struct Document {
    /// The document's text content.
    pub text: String,
    /// Line start offsets for span-to-position conversion.
    line_offsets: Vec<usize>,
}

impl Document {
    pub fn new(text: String) -> Self {
        let line_offsets = compute_line_offsets(&text);
        Self { text, line_offsets }
    }

    /// Convert a byte offset to an LSP Position (0-indexed line and character).
    pub fn offset_to_position(&self, offset: usize) -> Position {
        // Binary search for the line containing this offset
        let line = self
            .line_offsets
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);

        let line_start = self.line_offsets.get(line).copied().unwrap_or(0);
        let character = offset.saturating_sub(line_start);

        Position {
            line: line as u32,
            character: character as u32,
        }
    }

    /// Convert a span to an LSP Range.
    pub fn span_to_range(&self, span: &Span) -> Range {
        Range {
            start: self.offset_to_position(span.start),
            end: self.offset_to_position(span.end),
        }
    }
}

/// Compute line start offsets for a text.
fn compute_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (i, c) in text.char_indices() {
        if c == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// The Holdsmith language server backend.
pub struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, Document>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Analyze a document and publish diagnostics.
    pub async fn analyze_document(&self, uri: Url) {
        debug!(uri = %uri, "Analyzing document");

        let documents = self.documents.read().await;
        let Some(doc) = documents.get(&uri) else {
            warn!(uri = %uri, "Document not found in cache");
            return;
        };

        let mut diagnostics = Vec::new();

        // Parse the document
        let filename = uri.path();
        match parse(&doc.text, filename) {
            Ok(ast) => {
                debug!(passages = ast.passages.len(), "Parse succeeded");

                // Compile the AST
                match compile(ast) {
                    Ok(scene) => {
                        debug!(scene_id = %scene.id, "Compile succeeded");

                        // Run the analyzer
                        let analysis = analyze_scene(&scene);
                        let analyzer_diagnostics = generate_diagnostics(&analysis);

                        for diag in analyzer_diagnostics {
                            // Map analyzer diagnostics to LSP diagnostics
                            // Note: We don't have spans for analyzer diagnostics yet,
                            // so we report them at the start of the file
                            let severity = match diag.severity {
                                Severity::Error => DiagnosticSeverity::ERROR,
                                Severity::Warning => DiagnosticSeverity::WARNING,
                                Severity::Info => DiagnosticSeverity::INFORMATION,
                            };

                            let mut message = format!("[{}] {}", diag.code.as_str(), diag.message);
                            if let Some(details) = &diag.details {
                                message.push_str("\n\n");
                                message.push_str(details);
                            }
                            if let Some(help) = &diag.help {
                                message.push_str("\n\nHint: ");
                                message.push_str(help);
                            }

                            diagnostics.push(Diagnostic {
                                range: Range::default(), // TODO: Map to actual span
                                severity: Some(severity),
                                code: Some(NumberOrString::String(diag.code.as_str().to_string())),
                                source: Some("holdsmith".to_string()),
                                message,
                                ..Default::default()
                            });
                        }
                    }
                    Err(e) => {
                        // Compile error
                        error!(error = %e, "Compile failed");

                        let range = e
                            .span()
                            .map(|s| doc.span_to_range(s))
                            .unwrap_or_default();

                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("holdsmith".to_string()),
                            message: e.to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            Err(e) => {
                // Parse error
                error!(error = %e, span = ?e.span(), "Parse failed");

                let range = doc.span_to_range(&e.span());
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("holdsmith".to_string()),
                    message: e.to_string(),
                    ..Default::default()
                });
            }
        }

        info!(
            uri = %uri,
            count = diagnostics.len(),
            errors = diagnostics.iter().filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).count(),
            warnings = diagnostics.iter().filter(|d| d.severity == Some(DiagnosticSeverity::WARNING)).count(),
            "Publishing diagnostics"
        );

        // Publish diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("LSP initialize request received");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "holdsmith-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("LSP initialized notification received");
        self.client
            .log_message(MessageType::INFO, "Holdsmith LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        info!("LSP shutdown request received");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;

        info!(uri = %uri, bytes = text.len(), "Document opened");

        {
            let mut documents = self.documents.write().await;
            documents.insert(uri.clone(), Document::new(text));
        }

        self.analyze_document(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        // We use FULL sync, so there's only one change with the full content
        if let Some(change) = params.content_changes.into_iter().next() {
            debug!(uri = %uri, bytes = change.text.len(), "Document changed");

            {
                let mut documents = self.documents.write().await;
                documents.insert(uri.clone(), Document::new(change.text));
            }

            self.analyze_document(uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        info!(uri = %uri, "Document closed");

        {
            let mut documents = self.documents.write().await;
            documents.remove(&uri);
        }

        // Clear diagnostics for closed document
        self.client.publish_diagnostics(uri, vec![], None).await;
    }
}

/// Create an LSP service for the Holdsmith language server.
pub fn create_service() -> (LspService<Backend>, tower_lsp::ClientSocket) {
    LspService::new(Backend::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_offset_to_position() {
        let doc = Document::new("line1\nline2\nline3".to_string());

        // Start of file
        assert_eq!(doc.offset_to_position(0), Position { line: 0, character: 0 });

        // Middle of first line
        assert_eq!(doc.offset_to_position(3), Position { line: 0, character: 3 });

        // Start of second line
        assert_eq!(doc.offset_to_position(6), Position { line: 1, character: 0 });

        // Middle of second line
        assert_eq!(doc.offset_to_position(9), Position { line: 1, character: 3 });

        // Start of third line
        assert_eq!(doc.offset_to_position(12), Position { line: 2, character: 0 });
    }

    #[test]
    fn test_document_span_to_range() {
        let doc = Document::new("# Passage\nSome text here".to_string());

        let span = Span { start: 0, end: 9 };
        let range = doc.span_to_range(&span);

        assert_eq!(range.start, Position { line: 0, character: 0 });
        assert_eq!(range.end, Position { line: 0, character: 9 });
    }

    #[test]
    fn test_multiline_span() {
        let doc = Document::new("# Passage\n* Choice -> Target".to_string());

        // Span covering both lines
        let span = Span { start: 0, end: 28 };
        let range = doc.span_to_range(&span);

        assert_eq!(range.start, Position { line: 0, character: 0 });
        assert_eq!(range.end, Position { line: 1, character: 18 });
    }

    // Test the full analysis pipeline that the LSP uses
    #[test]
    fn test_analysis_pipeline_valid_scene() {
        let scene_text = r#"---
id: test
title: Test Scene
context: journey
---

=== start

Welcome to the test.

* [Continue]
  -> end

=== end

The end.
"#;
        let ast = parse(scene_text, "test.scene").expect("parse failed");
        let scene = compile(ast).expect("compile failed");
        let analysis = analyze_scene(&scene);
        let diagnostics = generate_diagnostics(&analysis);

        assert!(diagnostics.is_empty(), "Expected no diagnostics for valid scene, got: {:?}",
            diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_analysis_pipeline_unreachable_passage() {
        let scene_text = r#"---
id: test
title: Test Scene
context: journey
---

=== start

Welcome.

* [Go to end]
  -> end_passage

=== unreachable

This passage cannot be reached.

* [Continue]
  -> END

=== end_passage

Done.
"#;
        let ast = parse(scene_text, "test.scene").expect("parse failed");
        let scene = compile(ast).expect("compile failed");
        let analysis = analyze_scene(&scene);
        let diagnostics = generate_diagnostics(&analysis);

        let has_unreachable = diagnostics.iter().any(|d| {
            d.code.as_str().contains("unreachable") ||
            d.message.to_lowercase().contains("unreachable")
        });
        assert!(has_unreachable, "Expected unreachable passage diagnostic, got: {:?}",
            diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>());
    }

    #[test]
    fn test_analysis_pipeline_parse_error() {
        // Missing frontmatter should be a parse error
        let scene_text = "=== start\n* [Choice]\n";
        let result = parse(scene_text, "test.scene");
        assert!(result.is_err(), "Expected parse error for missing frontmatter");
    }

    #[test]
    fn test_lsp_diagnostic_mapping() {
        // Test that we can map analyzer diagnostics to LSP format
        let scene_text = r#"---
id: test
title: Test Scene
context: journey
---

=== start

* [Option]
  -> end_passage

=== orphan

Never reached.

* [Continue]
  -> END

=== end_passage

Done.
"#;
        let ast = parse(scene_text, "test.scene").unwrap();
        let scene = compile(ast).unwrap();
        let analysis = analyze_scene(&scene);
        let analyzer_diags = generate_diagnostics(&analysis);

        // Map to LSP diagnostics (mimics what analyze_document does)
        let lsp_diags: Vec<Diagnostic> = analyzer_diags.iter().map(|diag| {
            let severity = match diag.severity {
                Severity::Error => DiagnosticSeverity::ERROR,
                Severity::Warning => DiagnosticSeverity::WARNING,
                Severity::Info => DiagnosticSeverity::INFORMATION,
            };

            Diagnostic {
                range: Range::default(),
                severity: Some(severity),
                code: Some(NumberOrString::String(diag.code.as_str().to_string())),
                source: Some("holdsmith".to_string()),
                message: diag.message.clone(),
                ..Default::default()
            }
        }).collect();

        assert!(!lsp_diags.is_empty());
        assert!(lsp_diags.iter().all(|d| d.source == Some("holdsmith".to_string())));
    }
}
