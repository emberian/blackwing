//! Toast notification components.

use leptos::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

static TOAST_ID: AtomicU64 = AtomicU64::new(0);

/// Toast severity level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastLevel {
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}

impl ToastLevel {
    fn class(&self) -> &'static str {
        match self {
            ToastLevel::Info => "",
            ToastLevel::Success => "success",
            ToastLevel::Warning => "warning",
            ToastLevel::Danger => "danger",
        }
    }
}

/// A toast notification.
#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub level: ToastLevel,
    pub duration_ms: u32,
}

impl Toast {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            id: TOAST_ID.fetch_add(1, Ordering::Relaxed),
            message: message.into(),
            level: ToastLevel::Info,
            duration_ms: 3000,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            id: TOAST_ID.fetch_add(1, Ordering::Relaxed),
            message: message.into(),
            level: ToastLevel::Success,
            duration_ms: 3000,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            id: TOAST_ID.fetch_add(1, Ordering::Relaxed),
            message: message.into(),
            level: ToastLevel::Warning,
            duration_ms: 4000,
        }
    }

    pub fn danger(message: impl Into<String>) -> Self {
        Self {
            id: TOAST_ID.fetch_add(1, Ordering::Relaxed),
            message: message.into(),
            level: ToastLevel::Danger,
            duration_ms: 5000,
        }
    }

    pub fn with_duration(mut self, ms: u32) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Context for managing toasts.
#[derive(Clone)]
pub struct ToastContext {
    toasts: RwSignal<Vec<Toast>>,
}

impl ToastContext {
    pub fn new() -> Self {
        Self {
            toasts: RwSignal::new(Vec::new()),
        }
    }

    pub fn show(&self, toast: Toast) {
        let id = toast.id;
        let duration = toast.duration_ms;
        self.toasts.update(|t| t.push(toast));

        // Auto-dismiss after duration
        let toasts = self.toasts;
        gloo_timers::callback::Timeout::new(duration, move || {
            toasts.update(|t| t.retain(|toast| toast.id != id));
        })
        .forget();
    }

    pub fn dismiss(&self, id: u64) {
        self.toasts.update(|t| t.retain(|toast| toast.id != id));
    }

    pub fn toasts(&self) -> Signal<Vec<Toast>> {
        self.toasts.into()
    }
}

impl Default for ToastContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Display a single toast.
#[component]
fn ToastItem(toast: Toast) -> impl IntoView {
    view! {
        <div class=format!("bw-toast {}", toast.level.class())>
            {toast.message}
        </div>
    }
}

/// Container for toast notifications.
/// Place this once at the root of your app.
#[component]
pub fn ToastContainer(
    /// The toast context
    ctx: ToastContext,
) -> impl IntoView {
    let toasts = ctx.toasts();

    view! {
        <div class="bw-toast-container">
            {move || {
                toasts.get().into_iter().map(|toast| {
                    view! { <ToastItem toast=toast /> }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
