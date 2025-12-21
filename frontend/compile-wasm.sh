#!/bin/sh

cargo install wasm-pack
cd ./src/lib/converter
wasm-pack build --target web
