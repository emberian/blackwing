//! Player panel for testing scenes.

use leptos::prelude::*;
use std::sync::Arc;

use blackwing_ui::{Choice, ChoiceButton, StaticPassage};

use crate::app::AppContext;

/// Player panel for testing scenes interactively.
#[component]
pub fn PlayerPanel() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    // Create clones for each closure that needs them
    let ctx_state1 = ctx.clone();
    let ctx_state2 = ctx.clone();
    let player_state_controls = move || {
        ctx_state1.state().with(|s| s.player.clone())
    };
    let player_state_content = move || {
        ctx_state2.state().with(|s| s.player.clone())
    };

    let ctx_scenes = ctx.clone();
    let available_scenes = move || {
        ctx_scenes.state().with(|s| s.analyzer.compiled_scene_ids.clone())
    };

    let selected_scene = RwSignal::new(String::new());

    let ctx_start = ctx.clone();
    let start_play = Arc::new(move |_: web_sys::MouseEvent| {
        let scene_id = selected_scene.get();
        if !scene_id.is_empty() {
            ctx_start.dispatch(holdsmith_controller::Command::StartPlay { scene_id });
        }
    });

    let ctx_stop = ctx.clone();
    let stop_play = Arc::new(move |_: web_sys::MouseEvent| {
        ctx_stop.dispatch(holdsmith_controller::Command::StopPlay);
    });

    let ctx_restart = ctx.clone();
    let restart = Arc::new(move |_: web_sys::MouseEvent| {
        ctx_restart.dispatch(holdsmith_controller::Command::RestartScene);
    });

    let ctx_choice = ctx.clone();
    let make_choice = Arc::new(move |index: usize| {
        ctx_choice.dispatch(holdsmith_controller::Command::MakeChoice { index });
    });

    view! {
        <div class="player-panel">
            <div class="player-controls">
                {move || {
                    let state = player_state_controls();
                    let stop_play = stop_play.clone();
                    let restart = restart.clone();
                    if state.active {
                        view! {
                            <div class="active-controls">
                                <button on:click={
                                    let restart = restart.clone();
                                    move |ev| restart(ev)
                                } title="Restart Scene">
                                    "Restart"
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
                        let scenes = available_scenes();
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
                                    "Play"
                                </button>
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            <div class="player-content">
                {move || {
                    let state = player_state_content();
                    let make_choice = make_choice.clone();
                    if state.active {
                        // Convert controller choices to blackwing_ui choices
                        let choices: Vec<_> = state.choices.iter().map(|c| {
                            if c.enabled {
                                Choice::new(&c.text)
                            } else {
                                Choice::disabled(&c.text, c.disabled_reason.as_deref().unwrap_or(""))
                            }
                        }).collect();

                        view! {
                            <div class="passage">
                                <StaticPassage text=state.passage_text.clone() />

                                <div class="bw-choices">
                                    {choices.into_iter().enumerate().map(|(idx, choice)| {
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
                        }.into_any()
                    } else {
                        view! {
                            <div class="no-scene">
                                <p>"Select a scene to play"</p>
                                <p class="hint">"Scenes must be saved and compiled first"</p>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
