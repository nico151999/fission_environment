mod spec;

use std::fs;
use std::path::Path;
use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, web};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::StatusCode;
use environment_server::{UserFunctionLoaderV1Response, UserFunctionLoaderV2Response, V1SpecializeRequest, V2SpecializeRequest, UserFunctionRunnerResponse};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting program...");
    environment_server::run_server::<UserFunctionRuntimeContainer>(
        load_user_function_v1,
        load_user_function_v2,
        run_user_function
    ).await
}

struct UserFunctionRuntimeContainer {
    runtime: spec::bindings::Runtime,
    function_name: String // name of the JS function that is to be called
}

impl UserFunctionRuntimeContainer {
    fn new(user_func_path: &str, user_function: String) -> Result<UserFunctionRuntimeContainer, Box<dyn std::error::Error>> {
        println!("Loading user function \"{}\" from \"{}\"", user_function, user_func_path);
        let path = Path::new(user_func_path);
        let lib_path = if path.is_dir() {
            path.join(
                path.read_dir()?.next().ok_or_else(|| Box::new(
                    ErrorInternalServerError("Could not get library containing user function in expected path")
                ))??.path()
            )
        } else {
            path.to_path_buf()
        };
        println!("Loading WASMER runtime for running function {}", user_function);
        let module_wat = fs::read(lib_path)?;
        let rt = spec::bindings::Runtime::new(module_wat)?;
        Ok(UserFunctionRuntimeContainer{
            runtime: rt,
            function_name: user_function
        })
    }

    fn invoke(&self, req: HttpRequest, req_body: web::Bytes) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let actix_headers = req.headers();
        let mut headers = Vec::with_capacity(actix_headers.len());
        for (header_name, header_value) in actix_headers {
            headers.push(spec::types::HttpHeader {
                key: header_name.to_string(),
                value: header_value.to_str().unwrap_or("").to_string()
            });
        }
        let res = self.runtime.handle(
            spec::types::Request {
                user_input: spec::types::HttpRequest{
                    method: req.method().to_string(),
                    uri: req.uri().to_string(),
                    headers,
                    body: req_body.to_vec()
                },
                function_name: self.function_name.clone()
            }
        )?;
        let headers = res.headers;
        let mut response = HttpResponseBuilder::new(
            StatusCode::from_u16(res.status_code)?
        );
        for header in headers {
            response.append_header((header.key, header.value));
        }
        Ok(
            response.body(res.body)
        )
    }
}

const DEFAULT_FUNCTION_NAME: &str = "handle";

fn load_user_function_v1(_req: HttpRequest, req_body: V1SpecializeRequest, user_func_path: &str) -> UserFunctionLoaderV1Response<UserFunctionRuntimeContainer> {
    UserFunctionRuntimeContainer::new(
        user_func_path,
        req_body.function_name.unwrap_or_else(|| DEFAULT_FUNCTION_NAME.to_string())
    )
}

fn load_user_function_v2(_req: HttpRequest, req_body: V2SpecializeRequest) -> UserFunctionLoaderV2Response<UserFunctionRuntimeContainer> {
    UserFunctionRuntimeContainer::new(
        req_body.filepath.as_str(),
        req_body.function_name.unwrap_or_else(|| DEFAULT_FUNCTION_NAME.to_string())
    )
}

fn run_user_function(req: HttpRequest, req_body: web::Bytes, function_container: &UserFunctionRuntimeContainer) -> UserFunctionRunnerResponse {
    function_container.invoke(req, req_body)
}