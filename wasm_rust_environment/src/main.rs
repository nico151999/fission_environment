mod spec;

use std::fs;
use std::path::Path;
use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, web};
use actix_web::error::{ErrorInternalServerError, ErrorNotImplemented};
use actix_web::http::StatusCode;
use environment_server::{UserFunctionLoaderV1Response, UserFunctionLoaderV2Response, V1SpecializeRequest, V2SpecializeRequest, UserFunctionRunnerResponse};
use fp_bindgen_support::host::errors::InvocationError;
use http::HeaderMap;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting program...");
    environment_server::run_server::<UserFunctionContainer>(
        load_user_function_v1,
        load_user_function_v2,
        run_user_function
    ).await
}

struct UserFunctionContainer {
    runtime: spec::bindings::Runtime,
    handler_function: fn(&spec::bindings::Runtime, HttpRequest, web::Bytes) -> Result<spec::types::HttpResponse, InvocationError>
}

impl UserFunctionContainer {
    fn new(user_func_path: &str, user_function: String) -> Result<UserFunctionContainer, Box<dyn std::error::Error>> {
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
        let user_function = match user_function.as_str() {
            // handle is the only user function name currently supported
            "handle" => {
                |rt: &spec::bindings::Runtime, req: HttpRequest, req_body: web::Bytes| {
                    let actix_headers = req.headers();
                    let mut headers = HeaderMap::with_capacity(actix_headers.len());
                    for (header_name, header_value) in actix_headers {
                        headers.append(header_name.clone(), header_value.clone());
                    }
                    rt.handle(
                        spec::types::HttpRequest{
                            method: req.method().clone(),
                            uri: req.uri().clone(),
                            headers,
                            body: req_body.to_vec()
                        }
                    )
                }
            }
            _ => {
                println!("Tried to call user function {} which is not implemented", user_function);
                return Err(
                    Box::new(
                        ErrorNotImplemented(
                            format!("The function {} you tried to call is not implemented", user_function)
                        )
                    )
                );
            }
        };
        Ok(UserFunctionContainer{
            runtime: rt,
            handler_function: user_function
        })
    }

    fn invoke(&self, req: HttpRequest, req_body: web::Bytes) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let res = (self.handler_function)(&self.runtime, req, req_body)?;
        let headers = res.headers;
        let mut response = HttpResponseBuilder::new(
            StatusCode::from_u16(res.status_code)?
        );
        for (header_name, header_value) in headers {
            if let Some(header_name) = header_name {
                response.append_header((header_name, header_value));
            }
        }
        Ok(
            response.body(res.body)
        )
    }
}

const DEFAULT_FUNCTION_NAME: &str = "handle";

fn load_user_function_v1(_req: HttpRequest, req_body: V1SpecializeRequest, user_func_path: &str) -> UserFunctionLoaderV1Response<UserFunctionContainer> {
    UserFunctionContainer::new(
        user_func_path,
        req_body.function_name.unwrap_or_else(|| DEFAULT_FUNCTION_NAME.to_string())
    )
}

fn load_user_function_v2(_req: HttpRequest, req_body: V2SpecializeRequest) -> UserFunctionLoaderV2Response<UserFunctionContainer> {
    UserFunctionContainer::new(
        req_body.filepath.as_str(),
        req_body.function_name.unwrap_or_else(|| DEFAULT_FUNCTION_NAME.to_string())
    )
}

fn run_user_function(req: HttpRequest, req_body: web::Bytes, function_container: &UserFunctionContainer) -> UserFunctionRunnerResponse {
    function_container.invoke(req, req_body)
}