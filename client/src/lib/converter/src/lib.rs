use rustwell::{export_html, parse};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[wasm_bindgen]
pub fn preview_play(fountain: String) -> Result<String, JsValue> {
    let parsed = parse(fountain);
    let mut buffer = Vec::new();
    export_html(&parsed, &mut buffer, true, false);

    let html_string =
        String::from_utf8(buffer).map_err(|_| JsValue::from_str("Invalid UTF-8 sequence"))?;

    Ok(html_string)
}
