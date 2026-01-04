//! Analyzer panel with diagnostics and CFG visualization.

use leptos::prelude::*;

use crate::app::AppContext;
use holdsmith_controller::DiagnosticViewModel;

/// Analyzer panel showing diagnostics and CFG.
#[component]
pub fn AnalyzerPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    let ctx_diag = ctx.clone();
    let diagnostics = move || {
        ctx_diag.state().with(|s| s.editor.diagnostics.clone())
    };

    // Create separate clones for has_cfg usage in multiple closures
    let ctx_cfg1 = ctx.clone();
    let ctx_cfg2 = ctx.clone();
    let has_cfg_effect = move || {
        ctx_cfg1.state().with(|s| s.analyzer.has_cfg)
    };
    let has_cfg_button = move || {
        ctx_cfg2.state().with(|s| s.analyzer.has_cfg)
    };

    // Reference for the CFG container
    let cfg_container = NodeRef::<leptos::html::Div>::new();

    // TODO: CFG visualization requires access to full CFG data
    // For now, the CFG tab shows a placeholder when in Tauri mode
    // In pure WASM mode, we could access the controller directly
    Effect::new(move || {
        if has_cfg_effect() {
            if let Some(_el) = cfg_container.get() {
                // CFG rendering disabled for now - would need to serialize CFG data
                // or add a separate command to fetch it
            }
        }
    });

    // Tab state
    let show_cfg = RwSignal::new(false);

    view! {
        <div class="analyzer-panel">
            <div class="analyzer-tabs">
                <button
                    class:active=move || !show_cfg.get()
                    on:click=move |_| show_cfg.set(false)
                >
                    "Diagnostics"
                </button>
                <button
                    class:active=move || show_cfg.get()
                    on:click=move |_| show_cfg.set(true)
                    disabled=move || !has_cfg_button()
                >
                    "CFG"
                </button>
            </div>

            <div class="analyzer-content">
                {move || if show_cfg.get() {
                    view! {
                        <div class="cfg-view" node_ref=cfg_container>
                            // Cytoscape will render here
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <DiagnosticsList diagnostics=diagnostics() />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

/// List of diagnostics.
#[component]
fn DiagnosticsList(diagnostics: Vec<DiagnosticViewModel>) -> impl IntoView {
    let _ctx = expect_context::<AppContext>();

    if diagnostics.is_empty() {
        return view! {
            <div class="no-diagnostics">
                <p>"No issues found"</p>
            </div>
        }
        .into_any();
    }

    // Group by severity
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.severity, holdsmith_controller::SeverityViewModel::Error))
        .cloned()
        .collect();
    let warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.severity, holdsmith_controller::SeverityViewModel::Warning))
        .cloned()
        .collect();
    let infos: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.severity, holdsmith_controller::SeverityViewModel::Info))
        .cloned()
        .collect();

    view! {
        <div class="diagnostics-list">
            {if !errors.is_empty() {
                view! {
                    <div class="diagnostic-group errors">
                        <h4>"Errors (" {errors.len()} ")"</h4>
                        {errors.into_iter().map(|d| view! {
                            <DiagnosticItem diag=d />
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            {if !warnings.is_empty() {
                view! {
                    <div class="diagnostic-group warnings">
                        <h4>"Warnings (" {warnings.len()} ")"</h4>
                        {warnings.into_iter().map(|d| view! {
                            <DiagnosticItem diag=d />
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            {if !infos.is_empty() {
                view! {
                    <div class="diagnostic-group infos">
                        <h4>"Info (" {infos.len()} ")"</h4>
                        {infos.into_iter().map(|d| view! {
                            <DiagnosticItem diag=d />
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
    .into_any()
}

/// A single diagnostic item.
#[component]
fn DiagnosticItem(diag: DiagnosticViewModel) -> impl IntoView {
    let severity_class = match diag.severity {
        holdsmith_controller::SeverityViewModel::Error => "error",
        holdsmith_controller::SeverityViewModel::Warning => "warning",
        holdsmith_controller::SeverityViewModel::Info => "info",
    };

    view! {
        <div class={format!("diagnostic-item {}", severity_class)}>
            <div class="diagnostic-header">
                <span class="diagnostic-code">{diag.code}</span>
                {diag.location.as_ref().map(|loc| {
                    view! {
                        <span class="diagnostic-location">{loc.description.clone()}</span>
                    }
                })}
            </div>
            <div class="diagnostic-message">{diag.message}</div>
            {diag.help.map(|h| {
                view! {
                    <div class="diagnostic-help">"Hint: " {h}</div>
                }
            })}
        </div>
    }
}
