use actix_web::{HttpRequest, web};
use serde::{Serialize, Deserialize};
use crate::libloader::LibLoader;

pub type UserFunctionLoaderV1Response<UserFunction> = Result<UserFunction, Box<dyn std::error::Error>>;

pub type UserFunctionLoaderV1<UserFunction> = fn(HttpRequest, V1SpecializeRequest, &str) -> UserFunctionLoaderV1Response<UserFunction>;

pub struct FunctionLoaderV1<UserFunction> {
    function_loader: UserFunctionLoaderV1<UserFunction>,
    code_path: String
}

impl<UserFunction> FunctionLoaderV1<UserFunction> {
    pub fn new(function_loader: UserFunctionLoaderV1<UserFunction>, code_path: String) -> Self {
        FunctionLoaderV1{
            function_loader,
            code_path
        }
    }
}

impl<UserFunction> LibLoader<V1SpecializeRequest, UserFunctionLoaderV1Response<UserFunction>> for FunctionLoaderV1<UserFunction> {
    fn load(&self, specialization_req: HttpRequest, req_body: web::Json<V1SpecializeRequest>) -> UserFunctionLoaderV1Response<UserFunction> {
        (self.function_loader)(specialization_req, req_body.into_inner(), self.code_path.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V1SpecializeRequest {
    #[serde(rename = "functionName", with = "serde_with::rust::string_empty_as_none")]
    pub function_name: Option<String>,
    #[serde(with = "serde_with::rust::string_empty_as_none")]
    pub url: Option<String>
    // TODO: add metadata
}