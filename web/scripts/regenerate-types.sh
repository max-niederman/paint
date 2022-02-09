#!/usr/bin/env bash

crates=(canvas-lms)

script_dir=$(dirname $(realpath $0))
out_dir=$(realpath $script_dir/../types)

echo Generating TypeScript definitions and writing to $out_dir...

mkdir -p $out_dir

for crate in $crates
do
    cargo run --bin ts-definitions-canvas-lms --features=typescript-definitions > $out_dir/$crate.d.ts
    pnpx prettier -w $out_dir/$crate.d.ts
done