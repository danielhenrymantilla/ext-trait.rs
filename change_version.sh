#!/bin/sh

find . \
    -type f \
    -name 'Cargo.toml' \
    -print \
    -a \
    -exec \
        sed -i -E "s/\".*?\"(  # Keep in sync)/\"$1\"\\1/g" '{}' \
    \;
