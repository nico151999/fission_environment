use actix_web::{HttpRequest, web};

pub trait LibLoader<SpecializeRequest, UserFunctionLoaderResponse> {
    fn load(&self, specialization_req: HttpRequest, req_body: web::Json<SpecializeRequest>) -> UserFunctionLoaderResponse;
}