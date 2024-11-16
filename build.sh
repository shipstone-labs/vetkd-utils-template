#!/usr/bin/env bash
RELEASE=""
RELEASE_PATH="target/wasm32-unknown-unknown/debug"
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
  RELEASE_PATH="target/wasm32-unknown-unknown/release"
fi
cargo build $RELEASE
cd packages/vetkd-notes-client
wasm-pack build --target web $RELEASE
rm pkg/README.md pkg/.gitignore pkg/LICENSE pkg/package.json
mv pkg/* src/
ls src
rm -rf pkg
pnpm tsup
mv src/*.wasm* dist/
ls dist
pnpm pack
cd -

cd packages/vetkd-notes-canister
wasm-pack build $RELEASE
cd -
candid-extractor \
  "$RELEASE_PATH/vetkd_notes_canister.wasm" \
  > packages/vetkd-notes-canister/vetkd-notes-canister.did
./didc.sh \
  bind \
  -t js \
  packages/vetkd-notes-canister/vetkd-notes-canister.did \
  > packages/vetkd-notes-canister/src/vetkd_notes_canister.mjs

./didc.sh \
  bind \
  -t ts \
  packages/vetkd-notes-canister/vetkd-notes-canister.did \
  > packages/vetkd-notes-canister/src/vetkd_notes_canister.d.ts

cd packages/frontend
pnpm build
cd -