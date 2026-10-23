#!/usr/bin/env bash
# Construit dist/baskul (Baskul + son app de réglages) et
# dist/Baskul-Linux.tar.gz. Nécessite Rust, Node.js et, pour l'app de
# réglages (Tauri), les bibliothèques de développement de WebKitGTK :
#   sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libxdo-dev libssl-dev
set -euo pipefail
cd "$(dirname "$0")/.."

(cd app && corepack yarn install --immutable && corepack yarn build)
cargo build --release -p baskul
cargo build --release -p baskul-settings --features custom-protocol

rm -rf dist/baskul dist/Baskul-Linux.tar.gz
mkdir -p dist/baskul
cp target/release/baskul target/release/baskul-settings dist/baskul/
tar -C dist -czf dist/Baskul-Linux.tar.gz baskul
echo
echo "OK : dist/baskul (baskul + baskul-settings) et dist/Baskul-Linux.tar.gz"
