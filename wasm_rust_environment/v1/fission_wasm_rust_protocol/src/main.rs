use std::collections::{BTreeMap, BTreeSet};
use fp_bindgen::{BindingConfig, BindingsType, RustPluginConfig};
use fp_bindgen::prelude::{fp_export, fp_import, Serializable};
use fp_bindgen::types::CargoDependency;
use http::{Method, Uri};

fp_import! {}

fp_export! {
    fn handle(req: HttpRequest) -> HttpResponse;
}

#[derive(Serializable)]
pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub headers: http::HeaderMap,
    pub body: Vec<u8>
}

#[derive(Serializable)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: http::HeaderMap,
    pub body: Vec<u8>
}

fn main() {
    for bindings_type in [
        (
            BindingsType::RustPlugin(RustPluginConfig {
                name: format!("{}_plugin", env!("CARGO_PKG_NAME")).as_str(),
                authors: format!(r#"["{}"]"#, env!("CARGO_PKG_AUTHORS")).as_str(),
                version: env!("CARGO_PKG_VERSION"),
                dependencies: BTreeMap::from([
                    (
                        "http",
                        CargoDependency {
                            version: Some("0.2.8"),
                            ..CargoDependency::default()
                        },
                    ),
                    (
                        "fp-bindgen-support",
                        CargoDependency {
                            version: Some("2.0.1"),
                            features: BTreeSet::from(["async", "guest", "http"]),
                            // TODO: replace git link by one pointing to fiberplane as soon as it is merged
                            git: Some("https://github.com/nico151999/fp-bindgen.git"),
                            branch: Some("main"),
                            ..CargoDependency::default()
                        },
                    ),
                ]),
            }),
            format!("./bindings/{}_plugin", env!("CARGO_PKG_NAME")).as_str()
        ),
        (
            BindingsType::RustWasmerWasiRuntime,
            "../src/spec"
        )
    ] {
        let config = BindingConfig {
            bindings_type: bindings_type.0,
            path: bindings_type.1,
        };
        fp_bindgen::prelude::fp_bindgen!(config);
        println!("Generated bindings written to `{}/`.", bindings_type.1);
    }
}