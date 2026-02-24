use rustwell::{export_html, export_pdf, parse};
use wasm_bindgen::prelude::*;

/// Runs the Rustwell fountain parser and compiles the script into stylized HTML.
#[wasm_bindgen]
pub fn generate_html(fountain: String) -> Result<String, JsValue> {
    let parsed = parse(fountain);
    let mut buffer = Vec::new();
    export_html(&parsed, &mut buffer, true, false);

    let html_string =
        String::from_utf8(buffer).map_err(|_| JsValue::from_str("Invalid UTF-8 sequence"))?;

    Ok(html_string)
}

/// Runs the Rustwell fountain parser and compiles the script into stylized pdf.
#[wasm_bindgen]
pub fn generate_pdf(fountain: String) -> Result<Vec<u8>, JsValue> {
    let parsed = parse(fountain);
    let mut buffer = Vec::new();
    export_pdf(&parsed, &mut buffer, false);

    Ok(buffer)
}
