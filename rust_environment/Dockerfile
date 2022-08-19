ARG RUST_IMG_VERSION='1.63-bullseye'
ARG DEIAN_IMG_VERSION='buster-slim'

FROM rust:${RUST_IMG_VERSION}

WORKDIR /usr/src/app
COPY ./environment_server ./environment_server
COPY ./rust_environment ./rust_environment

WORKDIR rust_environment
RUN cargo build -r


FROM debian:${DEIAN_IMG_VERSION}

COPY --from=0 /usr/src/app/rust_environment/target/release/rust_environment ./usr/bin/

CMD ["rust_environment"]