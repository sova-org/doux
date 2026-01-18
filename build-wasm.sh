#!/bin/bash
set -e

echo "Building WASM..."
cargo build --target wasm32-unknown-unknown --release --no-default-features

echo "Copying to website/static/..."
cp target/wasm32-unknown-unknown/release/doux.wasm website/static/doux.wasm

echo "Verifying..."
ls -la website/static/doux.wasm
echo ""
echo "Done! WASM updated in website/static/"
