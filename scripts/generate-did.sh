#!/bin/sh

SCRIPT=$(readlink -f "$0")
SCRIPT_DIR=$(dirname "$SCRIPT")
cd $SCRIPT_DIR/..

echo Generating vibeverse_backend did file

if ! cargo install --list | grep -Fxq "candid-extractor v0.1.2:"
then
  cargo install --version 0.1.2 candid-extractor
fi

candid-extractor target/wasm32-unknown-unknown/release/vector_database_backend.wasm > src/vector_database_backend/vector_database_backend.did