//! CodeMirror 6 JavaScript bindings.

use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

#[wasm_bindgen]
extern "C" {
    /// Initialize CodeMirror on the given element.
    #[wasm_bindgen(js_namespace = window, js_name = initCodeMirror)]
    fn js_init_codemirror(element: &HtmlElement, content: &str, callback: &Closure<dyn Fn(String)>);

    /// Update the content of a CodeMirror instance.
    #[wasm_bindgen(js_namespace = window, js_name = updateCodeMirrorContent)]
    fn js_update_content(element: &HtmlElement, content: &str);

    /// Set diagnostics in CodeMirror (for underlines).
    #[wasm_bindgen(js_namespace = window, js_name = setCodeMirrorDiagnostics)]
    fn js_set_diagnostics(element: &HtmlElement, diagnostics: &str);
}

/// Initialize CodeMirror on a DOM element.
///
/// The `on_change` callback is called whenever the content changes.
pub fn init_codemirror<F>(element: &HtmlElement, initial_content: &str, on_change: F)
where
    F: Fn(String) + 'static,
{
    // Create a closure that can be called from JS
    let closure = Closure::new(move |content: String| {
        on_change(content);
    });

    js_init_codemirror(element, initial_content, &closure);

    // Leak the closure so it lives forever (it's tied to the editor lifetime)
    closure.forget();
}

/// Update the content of an existing CodeMirror instance.
///
/// This is called when content changes externally (e.g., file opened).
#[allow(dead_code)]
pub fn update_codemirror_content(element: &HtmlElement, content: &str) {
    js_update_content(element, content);
}

/// Set diagnostic markers in the editor.
#[allow(dead_code)]
pub fn set_diagnostics(element: &HtmlElement, diagnostics: &[holdsmith_analyzer::Diagnostic]) {
    // Convert diagnostics to a simple JSON format
    let diag_data: Vec<_> = diagnostics.iter().map(|d| {
        serde_json::json!({
            "severity": format!("{:?}", d.severity),
            "code": format!("{:?}", d.code),
            "message": d.message,
        })
    }).collect();
    let json = serde_json::to_string(&diag_data).unwrap_or_else(|_| "[]".to_string());
    js_set_diagnostics(element, &json);
}
