ARG BUILDER_IMAGE='fission/builder'
ARG RUST_IMG_VERSION='1.63-bullseye'

FROM ${BUILDER_IMAGE}

FROM rust:${RUST_IMG_VERSION}

COPY --from=0 /builder /builder
ADD rust_environment/v1/rust_builder/build.sh /usr/local/bin/build