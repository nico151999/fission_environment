# Fission environment

This repository utilizes Fission to create a FaaS solution that safely runs functions created by a third party that
cannot be analyzed in terms of malicious behavior and therefore need to run inside a sandboxed environment. Please note
that all Dockerfiles you find in this repository need to have this repository's root as their context.