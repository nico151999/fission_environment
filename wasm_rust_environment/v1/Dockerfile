ARG RUST_IMG_VERSION='1.63-bullseye'
ARG DEIAN_IMG_VERSION='bullseye-slim'

FROM rust:${RUST_IMG_VERSION}

RUN rustup component add rustfmt

WORKDIR /usr/src/app
COPY ./environment_server ./environment_server
COPY ./wasm_rust_environment/v1 ./wasm_rust_environment

WORKDIR wasm_rust_environment/fission_wasm_rust_protocol
RUN cargo run

WORKDIR ..
RUN cargo build -r


FROM debian:${DEIAN_IMG_VERSION}

COPY --from=0 /usr/src/app/wasm_rust_environment/target/release/wasm_rust_environment ./usr/bin/

CMD ["wasm_rust_environment"]