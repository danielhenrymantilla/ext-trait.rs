#!/bin/sh

set -euxo pipefail

(cd src/proc_macros
    cargo publish
)

cargo publish

(cd extension-traits && cargo publish)
