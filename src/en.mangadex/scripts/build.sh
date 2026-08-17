#!/usr/bin/env bash
set -e

echo "🔨 Building MangaDex source..."

# Build WASM
cargo build --release --target wasm32-unknown-unknown

echo "✓ Build complete!"
