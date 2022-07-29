use fp_bindgen_macros::fp_export_impl;
use fission_wasm_rust_protocol_plugin::{HttpRequest, HttpResponse};
use http::HeaderMap;

#[fp_export_impl(fission_wasm_rust_protocol_plugin)]
fn handle(req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: HeaderMap::new(),
        body: Vec::from(format!("Hi {}", String::from_utf8(req.body).unwrap()).as_bytes())
    }
}
