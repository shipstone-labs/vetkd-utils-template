#!/usr/bin/env bash
RELEASE=""
if [[ "$1" == "release" ]]
then
  RELEASE="--release"
fi
cargo build $RELEASE
cd packages/vetkd-notes-client
wasm-pack build --target web $RELEASE
mv pkg/wasm_client.js pkg/wasm_client.mjs
pnpm tsup
pnpm pack
cd -

cd packages/vetkd-notes-canister
wasm-pack build $RELEASE
cd -
