//! Choice components for interactive decisions.

use leptos::prelude::*;
use std::sync::Arc;

/// A single choice option.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    /// Display text for the choice
    pub text: String,
    /// Whether this choice is currently available
    pub enabled: bool,
    /// Reason why the choice is disabled (if any)
    pub disabled_reason: Option<String>,
}

impl Choice {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            enabled: true,
            disabled_reason: None,
        }
    }

    pub fn disabled(text: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            enabled: false,
            disabled_reason: Some(reason.into()),
        }
    }
}

/// A single choice button.
#[component]
pub fn ChoiceButton(
    /// The choice to display
    choice: Choice,
    /// Index of this choice (passed to on_click)
    index: usize,
    /// Callback when clicked
    on_click: Arc<dyn Fn(usize) + Send + Sync>,
) -> impl IntoView {
    let is_enabled = choice.enabled;
    let title = choice.disabled_reason.clone().unwrap_or_default();
    let text = choice.text.clone();

    view! {
        <button
            class="bw-choice-btn"
            disabled=!is_enabled
            title=title
            on:click=move |_| {
                if is_enabled {
                    on_click(index);
                }
            }
        >
            {text}
        </button>
    }
}

/// A list of choices.
#[component]
pub fn ChoiceList(
    /// The available choices
    choices: RwSignal<Vec<Choice>>,
    /// Callback when a choice is selected
    on_select: Arc<dyn Fn(usize) + Send + Sync>,
) -> impl IntoView {
    view! {
        <div class="bw-choices">
            {move || {
                let on_select = on_select.clone();
                choices.get().into_iter().enumerate().map(|(idx, choice)| {
                    let on_select = on_select.clone();
                    view! {
                        <ChoiceButton
                            choice=choice
                            index=idx
                            on_click=on_select
                        />
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
