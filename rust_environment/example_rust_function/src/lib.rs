use actix_web::{HttpRequest, HttpResponse, web};

#[no_mangle]
pub extern "Rust" fn handler(_req: HttpRequest, req_body: web::Bytes) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    Ok(HttpResponse::Ok().body(
        format!("Body {:?}!", String::from_utf8(req_body.to_vec()).unwrap())
    ))
}