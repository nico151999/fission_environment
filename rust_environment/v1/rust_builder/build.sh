#!/bin/bash

if [ -d "${SRC_PKG}" ]; then
    echo "Building directory ${SRC_PKG}"
    cd "${SRC_PKG}"
    # TODO: ensure no build script can do things we do not want it to do
    #  (e.g. do not allow build scripts, or sandbox build process while allowing to fetch packages)
    cargo build -r --locked --lib || exit 1
    # we do not know the name of the crate, so we use the first artifact
    for f in target/release/*.so; do
        mv "${f}" "${DEPLOY_PKG}"
        break
    done
elif [ -f "${SRC_PKG}" ]; then
    echo "Building file ${SRC_PKG}"
    rustc --crate-type dylib -o "${DEPLOY_PKG}"
else
    >&2 echo "${SRC_PKG} not found"
    exit 1
fi
echo 'Successfully built user function as shared object'