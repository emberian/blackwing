//! Theme configuration and CSS variables.

/// Theme colors for the Blackwing UI.
pub struct Theme {
    /// Background color (darkest)
    pub bg_primary: &'static str,
    /// Secondary background
    pub bg_secondary: &'static str,
    /// Tertiary background (cards, panels)
    pub bg_tertiary: &'static str,
    /// Primary text color
    pub text_primary: &'static str,
    /// Secondary/muted text
    pub text_secondary: &'static str,
    /// Accent color (actions, highlights)
    pub accent: &'static str,
    /// Accent hover
    pub accent_hover: &'static str,
    /// Warning color
    pub warning: &'static str,
    /// Danger/error color
    pub danger: &'static str,
    /// Success color
    pub success: &'static str,
}

/// Default dark theme inspired by space/sci-fi aesthetics.
pub const DARK_THEME: Theme = Theme {
    bg_primary: "#0a0a0f",
    bg_secondary: "#12121a",
    bg_tertiary: "#1a1a24",
    text_primary: "#e8e8ec",
    text_secondary: "#8888a0",
    accent: "#4a9eff",
    accent_hover: "#6ab0ff",
    warning: "#ffaa33",
    danger: "#ff4444",
    success: "#44cc66",
};

/// CSS styles for the Blackwing UI components.
/// Call this once at app initialization to inject styles.
pub fn inject_styles() {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(head) = document.head() {
                let style = document.create_element("style").unwrap();
                style.set_text_content(Some(COMPONENT_STYLES));
                let _ = head.append_child(&style);
            }
        }
    }
}

const COMPONENT_STYLES: &str = r#"
:root {
    --bw-bg-primary: #0a0a0f;
    --bw-bg-secondary: #12121a;
    --bw-bg-tertiary: #1a1a24;
    --bw-text-primary: #e8e8ec;
    --bw-text-secondary: #8888a0;
    --bw-accent: #4a9eff;
    --bw-accent-hover: #6ab0ff;
    --bw-warning: #ffaa33;
    --bw-danger: #ff4444;
    --bw-success: #44cc66;
    --bw-border: #2a2a3a;
    --bw-radius: 4px;
    --bw-font: system-ui, -apple-system, sans-serif;
    --bw-font-mono: ui-monospace, 'Cascadia Code', monospace;
}

/* PassageView */
.bw-passage {
    font-family: var(--bw-font);
    color: var(--bw-text-primary);
    line-height: 1.7;
    font-size: 1.1rem;
    max-width: 65ch;
    margin: 0 auto;
    padding: 1rem;
}

.bw-passage p {
    margin: 0 0 1em 0;
}

.bw-passage em {
    color: var(--bw-text-secondary);
    font-style: italic;
}

.bw-passage strong {
    color: var(--bw-accent);
    font-weight: 600;
}

/* ChoiceList */
.bw-choices {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-width: 50ch;
    margin: 1.5rem auto;
}

.bw-choice-btn {
    font-family: var(--bw-font);
    font-size: 1rem;
    padding: 0.875rem 1.25rem;
    background: var(--bw-bg-tertiary);
    color: var(--bw-text-primary);
    border: 1px solid var(--bw-border);
    border-radius: var(--bw-radius);
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
}

.bw-choice-btn:hover:not(:disabled) {
    background: var(--bw-accent);
    border-color: var(--bw-accent);
    color: var(--bw-bg-primary);
}

.bw-choice-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* ResourceBar */
.bw-resource-bar {
    display: flex;
    gap: 1.5rem;
    font-family: var(--bw-font-mono);
    font-size: 0.875rem;
    color: var(--bw-text-secondary);
}

.bw-resource {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.bw-resource-icon {
    font-size: 1rem;
}

.bw-resource-value {
    color: var(--bw-text-primary);
    font-weight: 500;
}

.bw-resource-value.low {
    color: var(--bw-warning);
}

.bw-resource-value.critical {
    color: var(--bw-danger);
}

/* Toast */
.bw-toast-container {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 1000;
}

.bw-toast {
    font-family: var(--bw-font);
    font-size: 0.875rem;
    padding: 0.75rem 1rem;
    background: var(--bw-bg-tertiary);
    color: var(--bw-text-primary);
    border-radius: var(--bw-radius);
    border-left: 3px solid var(--bw-accent);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    animation: bw-toast-in 0.2s ease;
}

.bw-toast.warning {
    border-left-color: var(--bw-warning);
}

.bw-toast.danger {
    border-left-color: var(--bw-danger);
}

.bw-toast.success {
    border-left-color: var(--bw-success);
}

@keyframes bw-toast-in {
    from {
        opacity: 0;
        transform: translateX(1rem);
    }
    to {
        opacity: 1;
        transform: translateX(0);
    }
}

/* Card */
.bw-card {
    background: var(--bw-bg-tertiary);
    border: 1px solid var(--bw-border);
    border-radius: var(--bw-radius);
    padding: 1rem;
}

.bw-card-header {
    font-family: var(--bw-font);
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--bw-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.75rem;
}

.bw-card-body {
    color: var(--bw-text-primary);
}

/* Modal */
.bw-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: bw-fade-in 0.15s ease;
}

.bw-modal {
    background: var(--bw-bg-secondary);
    border: 1px solid var(--bw-border);
    border-radius: var(--bw-radius);
    padding: 1.5rem;
    max-width: 90vw;
    max-height: 90vh;
    overflow: auto;
    animation: bw-scale-in 0.15s ease;
}

.bw-modal-header {
    font-family: var(--bw-font);
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--bw-text-primary);
    margin-bottom: 1rem;
}

@keyframes bw-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes bw-scale-in {
    from {
        opacity: 0;
        transform: scale(0.95);
    }
    to {
        opacity: 1;
        transform: scale(1);
    }
}
"#;
