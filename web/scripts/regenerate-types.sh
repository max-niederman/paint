#!/usr/bin/env bash

crates=("canvas-lms" "oil")

script_dir=$(dirname $(realpath $0))
root_dir=$(realpath $script_dir/../..)
out_dir=$(realpath $script_dir/../types)

pushd $root_dir/api

echo Generating TypeScript definitions and writing to $out_dir...

mkdir -p $out_dir

for crate in "${crates[@]}"
do
    cargo run --bin ts-definitions-$crate --features=typescript-definitions > $out_dir/$crate.d.ts
    pnpx prettier -w $out_dir/$crate.d.ts
done

popd