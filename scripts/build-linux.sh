#!/usr/bin/env bash
# Construit dist/bascule (Bascule + son app de réglages) et
# dist/Bascule-Linux.tar.gz. Nécessite Rust, Node.js et, pour l'app de
# réglages (Tauri), les bibliothèques de développement de WebKitGTK :
#   sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libxdo-dev libssl-dev
set -euo pipefail
cd "$(dirname "$0")/.."

(cd app && corepack yarn install --immutable && corepack yarn build)
cargo build --release -p bascule
cargo build --release -p bascule-settings --features custom-protocol

rm -rf dist/bascule dist/Bascule-Linux.tar.gz
mkdir -p dist/bascule
cp target/release/bascule target/release/bascule-settings dist/bascule/
tar -C dist -czf dist/Bascule-Linux.tar.gz bascule
echo
echo "OK : dist/bascule (bascule + bascule-settings) et dist/Bascule-Linux.tar.gz"
