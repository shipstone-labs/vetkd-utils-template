#!/usr/bin/env bash
RELEASE=""
if [[ "$1" == "clean" ]]
then
  rm -rf \
    packages/vetkd-notes-client/pkg \
    packages/vetkd-notes-client/dist \
    packages/vetkd-notes-client/node_modules \
    packages/vetkd-notes-canister/pkg \
    target
  exit 0
fi
if [[ "$1" == "release" ]]
then
  RELEASE="--release"
fi
cargo build $RELEASE
cd packages/vetkd-notes-client
wasm-pack build --target web $RELEASE
rm pkg/README.md pkg/.gitignore pkg/LICENSE pkg/package.json
mv pkg/vetkd_notes_client.js pkg/vetkd_notes_client.mjs
mv pkg/* src
rm -rf pkg
pnpm tsup
pnpm pack
cd -

cd packages/vetkd-notes-canister
wasm-pack build $RELEASE
cd -
