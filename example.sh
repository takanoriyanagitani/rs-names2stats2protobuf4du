#!/bin/bash

set -u

bin="./target/release/names2stats2protobuf4du"

mkdir -p ./HIDDEN_DIR
chmod 700 ./HIDDEN_DIR
touch ./HIDDEN_DIR/HIDDEN_FILE
chmod 000 ./HIDDEN_DIR

input(){
  ls Cargo.toml
  ls Cargo.lock
  echo ./HIDDEN_DIR
  echo ./HIDDEN_DIR/HIDDEN_FILE
  echo I_AM_NOT_FOUND
}

conv(){
  cat /dev/stdin |
    "${bin}"
}

run_fq(){
  input | conv | fq -d protobuf
}

run_json(){
  helper="${HOME}/go/bin/wasip1_wasm/pmaps2jmaps"
  test -f "${helper}" || exec sh -c '
    echo pmaps2jmaps missing.
    echo you can install it using go cmd from:
    echo   github.com/takanoriyanagitani/go-protomaps2jsonmaps
    exit 1
  '

  input |
    conv |
    wasmtime run "${helper}" |
    jq -c
}

#run_fq
run_json

chmod 755 ./HIDDEN_DIR
