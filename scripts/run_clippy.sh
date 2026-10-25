#!/bin/bash

build_type=""
output_file=clippy_output.log

if [[ "$1" == "--release" ]]; then
    build_type="--release"
fi

echo Running clippy and outputting to $output_file...
cargo clippy $build_type > $output_file 2>&1
echo Done!
