ARG BUILDER_IMAGE=fission/builder
ARG RUST_IMG_VERSION=1.61-buster

FROM ${BUILDER_IMAGE}

FROM rust:${RUST_IMG_VERSION}

COPY --from=0 /builder /builder
ADD rust_environment/rust_builder/build.sh /usr/local/bin/build