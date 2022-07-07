extern crate core;

mod v1;
mod v2;
mod libloader;

pub use crate::v1::{UserFunctionLoaderV1Response, V1SpecializeRequest};
pub use crate::v2::{UserFunctionLoaderV2Response, V2SpecializeRequest};

use crate::v1::{FunctionLoaderV1, UserFunctionLoaderV1};
use crate::v2::{FunctionLoaderV2, UserFunctionLoaderV2};

use std::sync::Mutex;
use actix_web::{App, HttpServer, Responder, HttpRequest, HttpResponse, web, http};
use actix_web::error::ErrorConflict;
use clap::Parser;
use crate::libloader::LibLoader;

pub type UserFunctionRunnerResponse = Result<HttpResponse /* better would be impl Responder */, Box<dyn std::error::Error>>;
pub type UserFunctionRunner<UserFunction> = fn(HttpRequest, web::Bytes, &UserFunction) -> UserFunctionRunnerResponse;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CmdlineArgs {
    /// The port the server will run on
    #[clap(short, long, value_parser, default_value_t = 8888)]
    port: u16,
    /// The path the user function is located at in v1 API
    #[clap(short, long, value_parser, default_value = "/userfunc/user")]
    code_path: String,
}

struct FunctionManager<UserFunction> {
    function_loader_v1: FunctionLoaderV1<UserFunction>,
    function_loader_v2: FunctionLoaderV2<UserFunction>,
    function_runner: UserFunctionRunner<UserFunction>,
    function: Option<UserFunction>
}

impl<UserFunction> FunctionManager<UserFunction> {
    fn new(loader_v1: FunctionLoaderV1<UserFunction>, loader_v2: FunctionLoaderV2<UserFunction>, runner: UserFunctionRunner<UserFunction>) -> Self {
        Self{
            function_loader_v1: loader_v1,
            function_loader_v2: loader_v2,
            function_runner: runner,
            function: None
        }
    }

    fn load_function_v1(&mut self, specialization_req: HttpRequest, req_body: web::Json<V1SpecializeRequest>) -> Result<(), Box<dyn std::error::Error>> {
        self.function = Some(
            self.function_loader_v1.load(specialization_req, req_body)?
        );
        Ok(())
    }

    fn load_function_v2(&mut self, specialization_req: HttpRequest, req_body: web::Json<V2SpecializeRequest>) -> Result<(), Box<dyn std::error::Error>> {
        self.function = Some(
            self.function_loader_v2.load(specialization_req, req_body)?
        );
        Ok(())
    }

    fn run_function(&self, req: HttpRequest, req_body: web::Bytes) -> UserFunctionRunnerResponse {
        let function = match &self.function {
            None => return Err(Box::new(ErrorConflict("user function has not been loaded yet and can therefore not be executed"))),
            Some(function) => function
        };
        (self.function_runner)(req, req_body, function)
    }
}

async fn specialize_v1<UserFunction>(
    req: HttpRequest,
    req_body: web::Json<V1SpecializeRequest>,
    data: web::Data<Mutex<FunctionManager<UserFunction>>>
) -> impl Responder {
    let mut fn_manager = match data.lock() {
        Ok(fn_manager) => fn_manager,
        Err(e) => {
            let error = format!("Failed locking function manager specializing function: {}", e);
            println!("{}", error);
            return HttpResponse::InternalServerError().body(error);
        }
    };
    match fn_manager.load_function_v1(req, req_body) {
        Ok(_) => {
            let msg = "Successfully loaded function";
            println!("{}", msg);
            HttpResponse::Accepted().body(msg)
        },
        Err(err) => {
            let error = format!("Failed loading user function: {}", err);
            println!("{}", error);
            HttpResponse::InternalServerError().body(error)
        }
    }
}

async fn specialize_v2<UserFunction>(
    req: HttpRequest,
    req_body: web::Json<V2SpecializeRequest>,
    data: web::Data<Mutex<FunctionManager<UserFunction>>>
) -> impl Responder {
    let mut fn_manager = match data.lock() {
        Ok(fn_manager) => fn_manager,
        Err(e) => {
            let error = format!("Failed locking function manager specializing function via v2: {}", e);
            println!("{}", error);
            return HttpResponse::InternalServerError().body(error);
        }
    };
    match fn_manager.load_function_v2(req, req_body) {
        Ok(_) => {
            let msg = "Successfully loaded function via v2";
            println!("{}", msg);
            HttpResponse::Accepted().body(msg)
        },
        Err(err) => {
            let error = format!("Failed loading user function via v2: {}", err);
            println!("{}", error);
            HttpResponse::InternalServerError().body(error)
        }
    }
}

async fn run_function<UserFunction>(
    req: HttpRequest,
    req_body: web::Bytes,
    data: web::Data<Mutex<FunctionManager<UserFunction>>>
) -> impl Responder {
    let fn_manager = match data.lock() {
        Ok(fn_manager) => fn_manager,
        Err(e) => {
            let error = format!("Failed locking function manager running specialized function: {}", e);
            println!("{}", error);
            return HttpResponse::InternalServerError().body(error);
        }
    };
    match fn_manager.run_function(req, req_body) {
        Ok(result) => {
            println!("Function was run successfully");
            result
        },
        Err(err) => {
            let error = format!("Failed running the user function: {}", err);
            println!("{}", error);
            HttpResponse::InternalServerError().body(error)
        }
    }
}

pub async fn run_server<UserFunction>(
    function_loader: UserFunctionLoaderV1<UserFunction>,
    function_loader_v2: UserFunctionLoaderV2<UserFunction>,
    function_runner: UserFunctionRunner<UserFunction>
) -> std::io::Result<()>
    where UserFunction : Sync + Send + 'static {
    let args: CmdlineArgs = CmdlineArgs::parse();
    let data = web::Data::new(
        Mutex::new(
            FunctionManager::new(
                FunctionLoaderV1::new(function_loader, args.code_path),
                FunctionLoaderV2::new(function_loader_v2),
                function_runner
            )
        )
    );
    println!("Launching server on port {}", args.port);
    // TODO: support websockets
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(
                web::resource("/healthz").route(
                    web::method(http::Method::GET).to(HttpResponse::Ok)
                )
            )
            .service(
                web::resource("/specialize").route(
                    web::method(http::Method::POST).to(specialize_v1::<UserFunction>)
                )
            )
            .service(
                web::resource("/v2/specialize").route(
                    web::method(http::Method::POST).to(specialize_v2::<UserFunction>)
                )
            )
            .default_service(web::to(run_function::<UserFunction>))
    })
        .bind(("0.0.0.0", args.port))?
        .run()
        .await
}