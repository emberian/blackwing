//! Root application component.

use leptos::prelude::*;

use crate::context::{GameContext, Screen};
use crate::screens::{NarrativeScreen, QuickActionBar, ShipStatusBar};

/// Additional styles for the game app.
const GAME_STYLES: &str = r#"
.game-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bw-bg-primary);
    color: var(--bw-text-primary);
}

.ship-status-bar {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 0.75rem 1rem;
    background: var(--bw-bg-secondary);
    border-bottom: 1px solid var(--bw-border);
}

.status-location {
    font-weight: 600;
    color: var(--bw-accent);
}

.status-cycle {
    color: var(--bw-text-secondary);
    font-family: var(--bw-font-mono);
    font-size: 0.875rem;
}

.main-content {
    flex: 1;
    overflow-y: auto;
    padding: 2rem 1rem;
}

.narrative-screen {
    max-width: 800px;
    margin: 0 auto;
}

.quick-action-bar {
    display: flex;
    justify-content: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--bw-bg-secondary);
    border-top: 1px solid var(--bw-border);
}

.quick-action {
    padding: 0.5rem 1rem !important;
    font-size: 0.875rem !important;
}

.welcome-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    padding: 2rem;
}

.welcome-title {
    font-size: 3rem;
    font-weight: 700;
    color: var(--bw-accent);
    margin-bottom: 1rem;
    letter-spacing: 0.1em;
}

.welcome-subtitle {
    font-size: 1.25rem;
    color: var(--bw-text-secondary);
    margin-bottom: 3rem;
    max-width: 40ch;
}

.start-button {
    font-size: 1.25rem;
    padding: 1rem 2rem;
}
"#;

/// Inject game-specific styles.
fn inject_game_styles() {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(head) = document.head() {
                let style = document.create_element("style").unwrap();
                style.set_text_content(Some(GAME_STYLES));
                let _ = head.append_child(&style);
            }
        }
    }
}

/// Root application component.
#[component]
pub fn App() -> impl IntoView {
    // Inject styles once
    inject_game_styles();

    // Create and provide game context
    let ctx = GameContext::new();
    provide_context(ctx.clone());

    let screen = ctx.screen();
    let is_loaded = move || ctx.is_loaded();

    view! {
        <div class="game-shell">
            {move || {
                if is_loaded() {
                    view! {
                        <ShipStatusBar />
                        <div class="main-content">
                            {move || match screen.get() {
                                Screen::Narrative => view! { <NarrativeScreen /> }.into_any(),
                                Screen::Inventory => view! { <PlaceholderScreen name="Inventory" /> }.into_any(),
                                Screen::Navigation => view! { <PlaceholderScreen name="Navigation" /> }.into_any(),
                                Screen::Chronicle => view! { <PlaceholderScreen name="Chronicle" /> }.into_any(),
                                Screen::Settings => view! { <PlaceholderScreen name="Settings" /> }.into_any(),
                                Screen::GameOver => view! { <PlaceholderScreen name="Game Over" /> }.into_any(),
                            }}
                        </div>
                        <QuickActionBar />
                    }.into_any()
                } else {
                    view! { <WelcomeScreen /> }.into_any()
                }
            }}
        </div>
    }
}

/// Welcome/start screen shown when no game is loaded.
#[component]
fn WelcomeScreen() -> impl IntoView {
    let ctx = expect_context::<GameContext>();

    let on_new_game = move |_: web_sys::MouseEvent| {
        // Create a minimal test bundle for now
        let bundle = serde_json::json!({
            "manifest": {
                "name": "Blackwing",
                "version": "0.1.0"
            },
            "schema": {
                "resources": []
            },
            "templates": [],
            "dialogues": [],
            "quests": [],
            "scenes": []
        });

        let seed = js_sys::Date::now() as u64;
        if let Err(e) = ctx.new_game(&bundle.to_string(), seed) {
            tracing::error!("Failed to start game: {}", e);
        }
    };

    view! {
        <div class="welcome-screen">
            <h1 class="welcome-title">"BLACKWING"</h1>
            <p class="welcome-subtitle">
                "You are an artilect—a sophont machine intelligence. "
                "The Blackwing is your body. Her hull is your skin. Her systems are your nerves."
            </p>
            <button
                class="bw-choice-btn start-button"
                on:click=on_new_game
            >
                "Begin Journey"
            </button>
        </div>
    }
}

/// Placeholder for screens not yet implemented.
#[component]
fn PlaceholderScreen(name: &'static str) -> impl IntoView {
    let ctx = expect_context::<GameContext>();

    view! {
        <div style="text-align: center; padding: 2rem;">
            <h2 style="color: var(--bw-text-secondary); margin-bottom: 1rem;">
                {name}
            </h2>
            <p style="color: var(--bw-text-secondary); margin-bottom: 2rem;">
                "Coming soon..."
            </p>
            <button
                class="bw-choice-btn"
                on:click=move |_| ctx.set_screen(Screen::Narrative)
            >
                "Back to Narrative"
            </button>
        </div>
    }
}
