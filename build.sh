#!/usr/bin/env bash
cargo build
cd packages/wasm-client && wasm-pack build --target web && mv pkg/wasm_client.js pkg/wasm_client.mjs && pnpm tsup && pnpm pack