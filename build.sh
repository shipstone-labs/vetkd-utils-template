#!/usr/bin/env bash
cargo build
cd packages/wasm-client && wasm-pack build
