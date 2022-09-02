ARG BUILDER_IMAGE='fission/builder'
ARG RUST_IMG_VERSION='1.63-bullseye'

FROM ${BUILDER_IMAGE} as builder


FROM rust:${RUST_IMG_VERSION}

ARG WASI_VERSION=16
ARG WASI_VERSION_SEMI="${WASI_VERSION}.0"
ARG WASI_VERSION_FULL="${WASI_VERSION_SEMI}-linux"
# the following must not be root
ARG WASI_SDK_PATH='/opt/wasi-sdk'
ARG APP_DIR='/usr/src/app'
WORKDIR ${APP_DIR}
COPY ./environment_server ./environment_server
COPY ./wasm_js_environment/v1 ./wasm_js_environment

ENV JS_RUNTIME_WASM="${APP_DIR}/wasm_js_runtime.wasm"
ENV QUICKJS_WASM_SYS_WASI_SDK_PATH="${WASI_SDK_PATH}"

RUN cd "${WASI_SDK_PATH}/.." &&\
    wget "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_VERSION}/wasi-sdk-${WASI_VERSION_FULL}.tar.gz" &&\
    tar xvf "wasi-sdk-${WASI_VERSION_FULL}.tar.gz" &&\
    mv "wasi-sdk-${WASI_VERSION_SEMI}" "${WASI_SDK_PATH}" &&\
    rm "wasi-sdk-${WASI_VERSION_FULL}.tar.gz" &&\
    rustup component add rustfmt &&\
    rustup target add wasm32-wasi &&\
    apt update &&\
    apt install -y cmake clang libclang-dev &&\
    cd "${APP_DIR}/wasm_js_environment/fission_wasm_js_protocol" &&\
    cargo run &&\
    cd '../wasm_js_builder' &&\
    CARGO_TARGET_DIR=./ cargo build --release &&\
    mv './release/wasm_js_builder' '/usr/local/bin/build' &&\
    cd 'wasm_js_runtime' &&\
    CARGO_TARGET_DIR=./ cargo build --release &&\
    mv './wasm32-wasi/release/wasm_js_runtime.wasm' '../../../wasm_js_runtime.wasm' &&\
    cd '../../..' &&\
    rm -rf 'wasm_js_environment'

COPY --from=builder /builder /builder