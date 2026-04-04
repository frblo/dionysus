use rustwell::{A4, Exporter, ExporterExt, HtmlExporter, PdfExporter, parse};
use wasm_bindgen::prelude::*;

/// Runs the Rustwell fountain parser and compiles the script into stylized HTML.
#[wasm_bindgen]
pub fn generate_standalone_html(fountain: String) -> Result<String, JsValue> {
    let parsed = parse(fountain);
    let exporter = HtmlExporter {
        standalone: true,
        synopses: false,
        include_source_positions: false,
    };
    let html_string = exporter
        .export_to_string(&parsed)
        .map_err(|_| JsValue::from_str("Invalid UTF-8 sequence"))?;

    Ok(html_string)
}

/// Gets the standard `css` for Rustwell `html`.
#[wasm_bindgen]
pub fn get_css() -> String {
    HtmlExporter::export_css().to_string()
}

/// Generates the `html` representation of the script using Rustwell.
#[wasm_bindgen]
pub fn generate_html(fountain: String) -> Result<String, JsValue> {
    let screenplay = parse(fountain);
    let exporter = HtmlExporter {
        standalone: false,
        synopses: false,
        include_source_positions: false,
    };
    let html_string = exporter
        .export_to_string(&screenplay)
        .map_err(|_| JsValue::from_str("Invalid UTF-8 sequence"))?;

    Ok(html_string)
}

/// Runs the Rustwell fountain parser and compiles the script into stylized pdf.
#[wasm_bindgen]
pub fn generate_pdf(fountain: String) -> Result<Vec<u8>, JsValue> {
    let parsed = parse(fountain);
    let mut buffer = Vec::new();
    let exporter = PdfExporter {
        synopses: false,
        paper_size: A4,
    };
    exporter
        .export(&parsed, &mut buffer)
        .map_err(|_| JsValue::from_str("Invalid UTF-8 sequence"))?;

    Ok(buffer)
}
