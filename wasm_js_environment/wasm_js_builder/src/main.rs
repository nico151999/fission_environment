use std::{fs, env};
use std::path::Path;
use binaryen::{CodegenConfig, Module};
use stdio_override::StdinOverride;
use wizer::Wizer;

const SRC_PKG_ENV_KEY: &str = "SRC_PKG";
const DEPLOY_PKG_ENV_KEY: &str = "DEPLOY_PKG";
const JS_RUNTIME_WASM_ENV_KEY: &str = "JS_RUNTIME_WASM";

fn main() {
    let wasm_file_path = get_env(JS_RUNTIME_WASM_ENV_KEY);
    let wasm_binary = fs::read(wasm_file_path.as_str()).unwrap_or_else(
        |err| panic!("Unable to read WASM file at '{}': {}", wasm_file_path, err)
    );

    let src_pkg = get_env(SRC_PKG_ENV_KEY);
    let mut elem_count: usize = 0;
    for elem in Path::new(src_pkg.as_str()).read_dir().unwrap() {
        if let Ok(elem) = elem {
            if elem.file_name().to_str().unwrap() == "package.json" {
                // TODO: if directory contains a package.json treat it like a node package,
                //  then generate a single js file, delete the other files, set elem_count
                //  to 1 and break.
                elem_count = 1;
                break;
            }
        }
        elem_count += 1;
    }
    if elem_count != 1 {
        panic!("Expected a node package or a single JS file");
    }
    // TODO: polyfill the only JS file using babel

    let src_pkg = Path::new(src_pkg.as_str()).read_dir().unwrap().next().unwrap().unwrap().path();

    let guard = StdinOverride::override_file(src_pkg.to_str().unwrap()).unwrap_or_else(
        |err| panic!("Couldn't override STDIN with file '{}': {}", src_pkg.to_str().unwrap(), err)
    );
    let mut wasm = Wizer::new()
        .allow_wasi(true)
        .expect("Wizer cannot allow WASI")
        .inherit_stdio(true)
        .run(wasm_binary.as_slice())
        .expect("Could not run Wizer on WASM binary");
    drop(guard);

    let codegen_cfg = CodegenConfig {
        optimization_level: 3, // Aggressively optimize for speed.
        shrink_level: 0,       // Don't optimize for size at the expense of performance.
        debug_info: false,
    };

    let mut module = Module::read(&wasm).expect("Unable to read wasm binary for wasm-opt optimizations");
    module.optimize(&codegen_cfg);
    module
        .run_optimization_passes(vec!["strip"], &codegen_cfg)
        .expect("Could not run optimization passes");
    wasm = module.write();

    let deploy_pkg = get_env(DEPLOY_PKG_ENV_KEY);
    fs::write(deploy_pkg.as_str(), wasm).unwrap_or_else(
        |err| panic!("Unable to write deploy pkg to '{}': {}", deploy_pkg, err)
    );
}

fn get_env(key: &str) -> String {
    env::var(key).unwrap_or_else(
        |err| panic!("Could not get {} env variable: {}", key, err)
    )
}