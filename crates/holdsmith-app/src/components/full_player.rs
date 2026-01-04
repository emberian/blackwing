//! Full player panel with complete game engine integration.
//!
//! Unlike the simple player which just walks through passages,
//! this panel uses the full game engine with Rhai script execution,
//! resources, flags, and all game mechanics.

use leptos::prelude::*;
use std::sync::Arc;

use blackwing_ui::{Choice, ChoiceButton, StaticPassage};
use holdsmith_controller::{ChronicleSnapshot, FlagSnapshot, FullPlayerSnapshot, ResourceSnapshot};

use crate::app::AppContext;

/// Full player panel with game state display.
#[component]
pub fn FullPlayerPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    // Get full player state
    let ctx_state = ctx.clone();
    let full_player_state = move || {
        ctx_state.state().with(|s| s.full_player.clone())
    };

    // Available scenes
    let ctx_scenes = ctx.clone();
    let available_scenes = move || {
        ctx_scenes.state().with(|s| s.analyzer.compiled_scene_ids.clone())
    };

    let selected_scene = RwSignal::new(String::new());

    // Start full play
    let ctx_start = ctx.clone();
    let start_play = Arc::new(move |_: web_sys::MouseEvent| {
        let scene_id = selected_scene.get();
        if !scene_id.is_empty() {
            ctx_start.dispatch(holdsmith_controller::Command::StartFullPlay { scene_id });
        }
    });

    // Stop full play
    let ctx_stop = ctx.clone();
    let stop_play = Arc::new(move |_: web_sys::MouseEvent| {
        ctx_stop.dispatch(holdsmith_controller::Command::StopFullPlay);
    });

    // Reset game state
    let ctx_reset = ctx.clone();
    let reset_state = Arc::new(move |_: web_sys::MouseEvent| {
        ctx_reset.dispatch(holdsmith_controller::Command::ResetFullPlayState { seed: None });
    });

    // Make choice
    let ctx_choice = ctx.clone();
    let make_choice = Arc::new(move |index: usize| {
        ctx_choice.dispatch(holdsmith_controller::Command::FullPlayChoice { index });
    });

    view! {
        <div class="full-player-panel">
            <ControlBar
                full_player_state=Signal::derive(full_player_state.clone())
                available_scenes=Signal::derive(available_scenes)
                selected_scene=selected_scene
                start_play=start_play
                stop_play=stop_play.clone()
                reset_state=reset_state.clone()
            />

            <div class="full-player-content">
                {move || {
                    let state = full_player_state();
                    if state.active {
                        view! {
                            <div class="full-player-active">
                                <PassageView
                                    text=state.passage_text.clone()
                                    choices=state.choices.clone()
                                    make_choice=make_choice.clone()
                                />
                                <StatePanel
                                    resources=state.resources.clone()
                                    flags=state.flags.clone()
                                    chronicle=state.chronicle.clone()
                                    script_errors=state.script_errors.clone()
                                />
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="full-player-inactive">
                                <p>"Select a scene to play with full game engine"</p>
                                <p class="hint">"Full play mode executes Rhai scripts and tracks resources/flags"</p>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Control bar for full player.
#[component]
fn ControlBar(
    full_player_state: Signal<FullPlayerSnapshot>,
    available_scenes: Signal<Vec<String>>,
    selected_scene: RwSignal<String>,
    start_play: Arc<dyn Fn(web_sys::MouseEvent) + Send + Sync>,
    stop_play: Arc<dyn Fn(web_sys::MouseEvent) + Send + Sync>,
    reset_state: Arc<dyn Fn(web_sys::MouseEvent) + Send + Sync>,
) -> impl IntoView {
    view! {
        <div class="full-player-controls">
            {move || {
                let state = full_player_state.get();
                let stop_play = stop_play.clone();
                let reset_state = reset_state.clone();

                if state.active {
                    view! {
                        <div class="active-controls">
                            <span class="scene-label">
                                {format!("Playing: {}", state.scene_id.as_deref().unwrap_or("unknown"))}
                            </span>
                            <button on:click={
                                let reset_state = reset_state.clone();
                                move |ev| reset_state(ev)
                            } title="Reset Game State">
                                "Reset State"
                            </button>
                            <button on:click={
                                let stop_play = stop_play.clone();
                                move |ev| stop_play(ev)
                            } title="Stop Playing">
                                "Stop"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    let scenes = available_scenes.get();
                    let start_play = start_play.clone();
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
                                    let start_play = start_play.clone();
                                    move |ev| start_play(ev)
                                }
                                disabled=move || selected_scene.get().is_empty()
                            >
                                "Full Play"
                            </button>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Passage view with choices.
#[component]
fn PassageView(
    text: String,
    choices: Vec<holdsmith_controller::FullPlayerChoiceSnapshot>,
    make_choice: Arc<dyn Fn(usize) + Send + Sync>,
) -> impl IntoView {
    let ui_choices: Vec<_> = choices.iter().map(|c| {
        if c.enabled {
            Choice::new(&c.text)
        } else {
            Choice::disabled(&c.text, c.disabled_reason.as_deref().unwrap_or(""))
        }
    }).collect();

    view! {
        <div class="passage-view">
            <StaticPassage text=text />

            <div class="bw-choices">
                {ui_choices.into_iter().enumerate().map(|(idx, choice)| {
                    let make_choice = make_choice.clone();
                    let on_click: Arc<dyn Fn(usize) + Send + Sync> = Arc::new(move |i| make_choice(i));
                    view! {
                        <ChoiceButton
                            choice=choice
                            index=idx
                            on_click=on_click
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

/// State panel showing resources, flags, chronicle, and errors.
#[component]
fn StatePanel(
    resources: Vec<ResourceSnapshot>,
    flags: Vec<FlagSnapshot>,
    chronicle: Vec<ChronicleSnapshot>,
    script_errors: Vec<String>,
) -> impl IntoView {
    view! {
        <div class="state-panel">
            // Resources section
            <div class="state-section resources">
                <h4>"Resources"</h4>
                {if resources.is_empty() {
                    view! { <p class="empty">"No resources"</p> }.into_any()
                } else {
                    view! {
                        <ul>
                            {resources.into_iter().map(|r| {
                                view! {
                                    <li>
                                        <span class="name">{r.name}</span>
                                        <span class="value">{r.value}</span>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    }.into_any()
                }}
            </div>

            // Flags section
            <div class="state-section flags">
                <h4>"Flags"</h4>
                {if flags.is_empty() {
                    view! { <p class="empty">"No flags set"</p> }.into_any()
                } else {
                    view! {
                        <ul>
                            {flags.into_iter().map(|f| {
                                view! {
                                    <li>
                                        <span class="name">{f.name}</span>
                                        <span class="value">{f.value}</span>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    }.into_any()
                }}
            </div>

            // Chronicle section
            <div class="state-section chronicle">
                <h4>"Chronicle"</h4>
                {if chronicle.is_empty() {
                    view! { <p class="empty">"No chronicle entries"</p> }.into_any()
                } else {
                    view! {
                        <ul>
                            {chronicle.into_iter().map(|c| {
                                view! {
                                    <li>
                                        <strong>{c.title}</strong>
                                        <p>{c.text}</p>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    }.into_any()
                }}
            </div>

            // Script errors section (for debugging)
            {if !script_errors.is_empty() {
                view! {
                    <div class="state-section errors">
                        <h4>"Script Errors"</h4>
                        <ul class="error-list">
                            {script_errors.into_iter().map(|e| {
                                view! {
                                    <li class="error">{e}</li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }}
        </div>
    }
}
