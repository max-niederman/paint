#!/usr/bin/env bash

crates=(canvas-lms)

json_schema_dir=$(mktemp -d)
script_dir=$(dirname $(realpath $0))
out_dir=$(realpath $script_dir/../../types)

echo Writing JSON Schema files to $json_schema_dir...

for crate in $crates
do
    mkdir $json_schema_dir/$crate
    cargo run --bin schema-$crate -- $json_schema_dir/$crate
done

echo Generating TypeScript defitions and writing to $out_dir...

rm -r $out_dir
mkdir -p $out_dir

for crate in $crates
do
    mkdir $out_dir/$crate
    $script_dir/gen-ts-definitions.js $json_schema_dir/$crate $out_dir/$crate/index.d.ts
done