# Rust environment

This is an environment which directly loads a user function that was compiled to a dynamic library and executes this
function whenever requested to do so. ATTENTION: The environment does not feature any kind of sandboxing of the user function.
This means it might make use of the kube service account assigned to the pod it is running in.