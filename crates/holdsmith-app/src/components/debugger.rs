//! Debugger panel for stepping through scenes.

use leptos::prelude::*;
use std::sync::Arc;

use crate::app::AppContext;

/// Debugger panel for stepping through scene execution.
#[component]
pub fn DebuggerPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    // Create separate clones for each closure
    let ctx_state1 = ctx.clone();
    let ctx_state2 = ctx.clone();
    let debugger_state_controls = move || ctx_state1.state().with(|s| s.debugger.clone());
    let debugger_state_content = move || ctx_state2.state().with(|s| s.debugger.clone());

    let ctx_scenes = ctx.clone();
    let available_scenes = move || {
        ctx_scenes.state().with(|s| s.analyzer.compiled_scene_ids.clone())
    };

    let selected_scene = RwSignal::new(String::new());

    let ctx_start = ctx.clone();
    let start_debug = Arc::new(move |_: web_sys::MouseEvent| {
        let scene_id = selected_scene.get();
        if !scene_id.is_empty() {
            ctx_start.dispatch(holdsmith_controller::Command::StartDebug { scene_id });
        }
    });

    let ctx_stop = ctx.clone();
    let stop_debug = Arc::new(move |_: web_sys::MouseEvent| {
        ctx_stop.dispatch(holdsmith_controller::Command::StopDebug);
    });

    let ctx_step = ctx.clone();
    let step_into = Arc::new(move |_: web_sys::MouseEvent| {
        ctx_step.dispatch(holdsmith_controller::Command::StepInto);
    });

    let ctx_cont = ctx.clone();
    let continue_exec = Arc::new(move |_: web_sys::MouseEvent| {
        ctx_cont.dispatch(holdsmith_controller::Command::Continue);
    });

    view! {
        <div class="debugger-panel">
            <div class="debugger-controls">
                {move || {
                    let state = debugger_state_controls();
                    let stop_debug = stop_debug.clone();
                    let step_into = step_into.clone();
                    let continue_exec = continue_exec.clone();
                    if state.active {
                        view! {
                            <div class="active-controls">
                                <button on:click={
                                    let step_into = step_into.clone();
                                    move |ev| step_into(ev)
                                } title="Step Into (F11)">
                                    "Step"
                                </button>
                                <button on:click={
                                    let continue_exec = continue_exec.clone();
                                    move |ev| continue_exec(ev)
                                } title="Continue (F5)">
                                    "Continue"
                                </button>
                                <button on:click={
                                    let stop_debug = stop_debug.clone();
                                    move |ev| stop_debug(ev)
                                } title="Stop Debugging">
                                    "Stop"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        let scenes = available_scenes();
                        let start_debug = start_debug.clone();
                        view! {
                            <div class="start-controls">
                                <select
                                    on:change=move |ev| {
                                        selected_scene.set(event_target_value(&ev));
                                    }
                                >
                                    <option value="">"Select a scene..."</option>
                                    {scenes.into_iter().map(|scene| {
                                        let s = scene.clone();
                                        view! {
                                            <option value={scene}>{s}</option>
                                        }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <button
                                    on:click={
                                        let start_debug = start_debug.clone();
                                        move |ev| start_debug(ev)
                                    }
                                    disabled=move || selected_scene.get().is_empty()
                                >
                                    "Debug"
                                </button>
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="debugger-content">
                {move || {
                    let state = debugger_state_content();
                    if state.active {
                        let scene_id_str = state.scene_id.clone().unwrap_or_default();
                        let passage_str = state.passage_index.map(|i| i.to_string()).unwrap_or_else(|| "-".to_string());
                        let breakpoints: Vec<_> = state.breakpoints.iter().map(|(s, p)| format!("{}:{}", s, p)).collect();

                        view! {
                            <div class="debug-info">
                                <div class="current-location">
                                    <h4>"Location"</h4>
                                    <p>"Scene: " {scene_id_str}</p>
                                    <p>"Passage: " {passage_str}</p>
                                    <p>"Paused: " {if state.paused { "Yes" } else { "No" }}</p>
                                </div>

                                <div class="breakpoints">
                                    <h4>"Breakpoints"</h4>
                                    {if breakpoints.is_empty() {
                                        view! { <p class="hint">"No breakpoints set"</p> }.into_any()
                                    } else {
                                        view! {
                                            <ul>
                                                {breakpoints.into_iter().map(|bp| {
                                                    view! { <li>{bp}</li> }
                                                }).collect::<Vec<_>>()}
                                            </ul>
                                        }.into_any()
                                    }}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="no-debug">
                                <p>"Select a scene to debug"</p>
                                <p class="hint">"Use the CFG view to set breakpoints"</p>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
