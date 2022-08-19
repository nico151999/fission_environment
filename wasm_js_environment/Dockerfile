ARG RUST_IMG_VERSION='1.63-bullseye'
ARG DEIAN_IMG_VERSION='bullseye-slim'

FROM rust:${RUST_IMG_VERSION}

WORKDIR /usr/src/app
COPY ./environment_server ./environment_server
COPY ./wasm_js_environment ./wasm_js_environment

WORKDIR wasm_js_environment/fission_wasm_js_protocol
RUN rustup component add rustfmt &&\
    cargo run

WORKDIR ..
RUN cargo build -r


FROM debian:${DEIAN_IMG_VERSION}

COPY --from=0 /usr/src/app/wasm_js_environment/target/release/wasm_js_environment ./usr/bin/

CMD ["wasm_js_environment"]