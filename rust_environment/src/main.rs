use std::path::Path;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::error::ErrorInternalServerError;
#[cfg(libloading_docs)]
use libloading::os::unix as imp;
#[cfg(all(not(libloading_docs), unix))]
use libloading::os::unix as imp;
#[cfg(all(not(libloading_docs), windows))]
use libloading::os::windows as imp;
use environment_server::{UserFunctionLoaderV1Response, UserFunctionLoaderV2Response, V1SpecializeRequest, V2SpecializeRequest, UserFunctionRunnerResponse};

type UserFunction = extern "Rust" fn(HttpRequest, web::Bytes) -> UserFunctionRunnerResponse;
struct UserFunctionContainer {
    // this field exists to keep the library in memory which is necessary to use a raw symbol;
    // an alternative would be to load the library, check for a symbol's existence, but not put
    // the symbol itself but the symbol name in the struct and read the symbol again when the
    // function is invoked. This requires the symbol to be loaded twice though and either way the
    // library needs to remain loaded anyway.
    library: libloading::Library,
    function: imp::Symbol<UserFunction>
}
impl UserFunctionContainer {
    unsafe fn new(user_func_path: &str, user_function: String) -> Result<UserFunctionContainer, Box<dyn std::error::Error>> {
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
        let lib = libloading::Library::new(lib_path)?;
        Ok(
            UserFunctionContainer{
                function: (lib.get(user_function.as_bytes())? as libloading::Symbol<UserFunction>).into_raw(),
                library: lib
            }
        )
    }

    fn invoke(&self, req: HttpRequest, req_body: web::Bytes) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        (self.function)(req, req_body)
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting program...");
    environment_server::run_server::<UserFunctionContainer>(
        load_user_function_v1,
        load_user_function_v2,
        run_user_function
    ).await
}

const DEFAULT_FUNCTION_NAME: &str = "handle";

fn load_user_function_v1(_req: HttpRequest, req_body: V1SpecializeRequest, user_func_path: &str) -> UserFunctionLoaderV1Response<UserFunctionContainer> {
    unsafe {
        UserFunctionContainer::new(
            user_func_path,
            req_body.function_name.unwrap_or_else(|| DEFAULT_FUNCTION_NAME.to_string())
        )
    }
}

fn load_user_function_v2(_req: HttpRequest, req_body: V2SpecializeRequest) -> UserFunctionLoaderV2Response<UserFunctionContainer> {
    unsafe {
        UserFunctionContainer::new(
            req_body.filepath.as_str(),
            req_body.function_name.unwrap_or_else(|| DEFAULT_FUNCTION_NAME.to_string())
        )
    }
}

fn run_user_function(req: HttpRequest, req_body: web::Bytes, function_container: &UserFunctionContainer) -> UserFunctionRunnerResponse {
    function_container.invoke(req, req_body)
}