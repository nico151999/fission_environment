use actix_web::{HttpRequest, web};
use serde::{Serialize, Deserialize};
use crate::libloader::LibLoader;

pub type UserFunctionLoaderV2Response<UserFunction> = Result<UserFunction, Box<dyn std::error::Error>>;

pub type UserFunctionLoaderV2<UserFunction> = fn(HttpRequest, V2SpecializeRequest) -> UserFunctionLoaderV2Response<UserFunction>;

pub struct FunctionLoaderV2<UserFunction> {
    function_loader: UserFunctionLoaderV2<UserFunction>
}

impl<UserFunction> FunctionLoaderV2<UserFunction> {
    pub fn new(function_loader: UserFunctionLoaderV2<UserFunction>) -> Self {
        FunctionLoaderV2{
            function_loader
        }
    }
}

impl<UserFunction> LibLoader<V2SpecializeRequest, UserFunctionLoaderV2Response<UserFunction>> for FunctionLoaderV2<UserFunction> {
    fn load(&self, specialization_req: HttpRequest, req_body: web::Json<V2SpecializeRequest>) -> UserFunctionLoaderV2Response<UserFunction> {
        (self.function_loader)(specialization_req, req_body.into_inner())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V2SpecializeRequest {
    #[serde(rename = "functionName", with = "serde_with::rust::string_empty_as_none")]
    pub function_name: Option<String>,
    pub filepath: String,
    #[serde(with = "serde_with::rust::string_empty_as_none", default)]
    pub url: Option<String>
    // TODO: add metadata
}