// This implementation was inspired by Shopify's Javy: A JavaScript to WebAssembly toolchain

use fp_bindgen_macros::fp_export_impl;
use quickjs_wasm_rs::{json, Context, Value};
use once_cell::sync::OnceCell;
use std::io::{self, Read};
use fission_wasm_js_protocol_plugin::{HttpResponse, Request};

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static mut GLOBAL_OBJECT: OnceCell<Value> = OnceCell::new();
static JS_FILE_NAME: &str = "user_function.js";

#[export_name = "wizer.initialize"]
pub extern "C" fn initialize() {
    unsafe {
        let mut context = Context::default();
        context
            // write both STDOUT and STDERR to STDERR to ensure STDOUT does not impact the returned value
            .register_globals(io::stdout(), io::stderr())
            .unwrap();

        let mut contents = String::new();
        io::stdin().read_to_string(&mut contents).unwrap();

        let _ = context.eval_global(JS_FILE_NAME, &contents).expect("Could not evaluate passed JavaScript");
        let global = context.global_object().expect("Could not get the passed JavaScript's global object");

        JS_CONTEXT.set(context).unwrap();
        GLOBAL_OBJECT.set(global).unwrap();
    }
}

#[fp_export_impl(fission_wasm_js_protocol_plugin)]
fn handle(parameter: Request) -> HttpResponse {
    let result: HttpResponse = unsafe {
        let context = JS_CONTEXT.get().unwrap();
        let global = GLOBAL_OBJECT.get().unwrap();

        let main = global.get_property(parameter.function_name.as_str()).unwrap_or_else(
            |err| panic!(
                "Could not get the user function '{}' in the passed JavaScript: {}",
                parameter.function_name,
                err
            )
        );

        let input_value = json::transcode_input(
            context,
            serde_json::to_vec(
                &parameter.user_input
            ).unwrap().as_slice()
        ).unwrap();

        let output_value = main.call(global, &[input_value]).unwrap_or_else(
            |err| panic!("Failed executing user function: {}", err)
        );

        serde_json::from_slice(
            json::transcode_output(output_value).unwrap().as_slice()
        ).unwrap_or_else(
            |err| panic!("The user function returned a value of an unexpected format: {}", err)
        )
    };
    result
}