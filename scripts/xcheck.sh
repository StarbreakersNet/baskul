#!/usr/bin/env bash
# Vérifie (sans les exécuter) le code Windows et macOS depuis Linux.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo clippy --features xcheck -- -D warnings
echo "Code Windows et macOS : OK"
