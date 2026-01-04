//! Utility JavaScript bindings.

use wasm_bindgen::JsCast;

use crate::app::AppContext;

/// Show a confirmation dialog and return the result.
pub fn confirm(message: &str) -> bool {
    web_sys::window()
        .and_then(|w| Some(w.confirm_with_message(message).unwrap_or(false)))
        .unwrap_or(false)
}

/// Trigger a file download in the browser.
pub fn download_blob(data: &[u8], filename: &str, mime_type: &str) {
    use js_sys::{Array, Uint8Array};

    let array = Uint8Array::new_with_length(data.len() as u32);
    array.copy_from(data);

    let parts = Array::new();
    parts.push(&array.buffer());

    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime_type);

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .expect("Failed to create blob");

    let url = web_sys::Url::create_object_url_with_blob(&blob).expect("Failed to create URL");

    // Create a temporary link and click it
    let document = web_sys::window()
        .and_then(|w| w.document())
        .expect("No document");

    let link = document
        .create_element("a")
        .expect("Failed to create link")
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .expect("Not an anchor");

    link.set_href(&url);
    link.set_download(filename);
    link.click();

    // Clean up
    let _ = web_sys::Url::revoke_object_url(&url);
}

/// Handle file import from an input element.
pub fn handle_file_import(ev: web_sys::Event, ctx: AppContext) {
    use gloo_file::callbacks::read_as_bytes;
    use wasm_bindgen::JsCast;

    let target = ev.target().unwrap();
    let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();

    if let Some(files) = input.files() {
        if let Some(file) = files.get(0) {
            let gloo_file = gloo_file::File::from(file);

            read_as_bytes(&gloo_file, move |result| {
                match result {
                    Ok(data) => {
                        if let Err(e) = ctx.import_zip(&data) {
                            web_sys::console::error_1(
                                &format!("Import failed: {}", e).into(),
                            );
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Read failed: {:?}", e).into());
                    }
                }
            });
        }
    }

    // Clear the input so the same file can be selected again
    input.set_value("");
}
