#!/bin/sh

set -euxo pipefail

(cd src/proc_macros
    cargo publish
)

for i in $(seq 10)
do
    cargo publish && break
    sleep 4
done
