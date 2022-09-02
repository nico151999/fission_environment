# WASM JS environment

This is an environment which loads a user function that was previously compiled to a WASM binary. Whenever requested to
do so it runs the function in a WASMER runtime environment which allows for the function to be executed in a sandboxed
mode. The WASM binary can be created using the builder which utilizes WIZER in order to create an initialised JS runtime
with a loaded JS file written by the user.