#!/bin/bash
set -e

echo "Installing cargo-llvm-cov..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
cargo install cargo-llvm-cov

echo "Installation complete!"
