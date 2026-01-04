//! Narrative screen - the main game view.

use leptos::prelude::*;
use std::sync::Arc;

use blackwing_ui::{Choice, ChoiceList, PassageView, Resource, StaticResourceBar};

use crate::context::GameContext;

/// The main narrative screen showing passages and choices.
#[component]
pub fn NarrativeScreen() -> impl IntoView {
    let ctx = expect_context::<GameContext>();
    let state = ctx.state();

    // Derive passage text signal
    let passage_text = RwSignal::new(String::new());
    Effect::new(move |_| {
        let s = state.get();
        if s.passage_text.is_empty() {
            passage_text.set("The void stretches before you, infinite and patient.\n\nWhat will you do?".to_string());
        } else {
            passage_text.set(s.passage_text);
        }
    });

    // Derive choices signal
    let choices = RwSignal::new(Vec::new());
    Effect::new(move |_| {
        let s = state.get();
        let c: Vec<Choice> = s
            .choices
            .iter()
            .map(|ci| {
                if ci.enabled {
                    Choice::new(&ci.text)
                } else {
                    Choice::disabled(&ci.text, ci.disabled_reason.as_deref().unwrap_or(""))
                }
            })
            .collect();

        // If no choices, provide default actions
        if c.is_empty() {
            choices.set(vec![
                Choice::new("Check ship status"),
                Choice::new("View star map"),
                Choice::new("Rest"),
            ]);
        } else {
            choices.set(c);
        }
    });

    let ctx_choice = ctx.clone();
    let on_choice = Arc::new(move |index: usize| {
        if let Err(e) = ctx_choice.make_choice(index) {
            tracing::error!("Choice failed: {}", e);
        }
    });

    view! {
        <div class="narrative-screen">
            <PassageView text=passage_text />
            <ChoiceList choices=choices on_select=on_choice />
        </div>
    }
}

/// Ship status bar at the top of the screen.
#[component]
pub fn ShipStatusBar() -> impl IntoView {
    let ctx = expect_context::<GameContext>();
    let state = ctx.state();

    view! {
        <div class="ship-status-bar">
            {move || {
                let s = state.get();
                let resources = vec![
                    Resource::new("Credits", "$", s.credits),
                    Resource::new("Fuel", "⛽", s.fuel)
                        .with_max(s.fuel_max)
                        .with_thresholds(30, 10),
                    Resource::new("Hull", "🛡", s.hull)
                        .with_max(s.hull_max)
                        .with_thresholds(50, 25),
                ];
                view! {
                    <div class="status-location">
                        {s.location.unwrap_or_else(|| "Unknown".to_string())}
                    </div>
                    <div class="status-cycle">
                        "Cycle " {s.cycle}
                    </div>
                    <StaticResourceBar resources=resources />
                }
            }}
        </div>
    }
}

/// Quick action bar at the bottom.
#[component]
pub fn QuickActionBar() -> impl IntoView {
    let ctx = expect_context::<GameContext>();

    let ctx_inv = ctx.clone();
    let ctx_nav = ctx.clone();
    let ctx_chr = ctx.clone();

    view! {
        <div class="quick-action-bar">
            <button
                class="bw-choice-btn quick-action"
                on:click=move |_| ctx_inv.set_screen(crate::context::Screen::Inventory)
            >
                "📦 Inventory"
            </button>
            <button
                class="bw-choice-btn quick-action"
                on:click=move |_| ctx_nav.set_screen(crate::context::Screen::Navigation)
            >
                "🗺️ Navigate"
            </button>
            <button
                class="bw-choice-btn quick-action"
                on:click=move |_| ctx_chr.set_screen(crate::context::Screen::Chronicle)
            >
                "📜 Chronicle"
            </button>
        </div>
    }
}
