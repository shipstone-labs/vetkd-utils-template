#!/usr/bin/env bash
cargo watch -x "build" -s "cd packages/wasm-client && wasm-pack build && mv pkg/wasm_client.js pkg/wasm_client.mjs && pnpm tsup"
