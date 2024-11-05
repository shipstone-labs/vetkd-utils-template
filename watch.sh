#!/usr/bin/env bash
cargo watch -x "build" -s "cd packages/wasm-client && wasm-pack build"
