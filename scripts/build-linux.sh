#!/usr/bin/env bash
# Construit dist/baskul (Baskul et sa fenêtre de réglages, avec Tauri) et
# dist/Baskul-Linux.tar.gz. Nécessite Rust, Node.js et les bibliothèques de
# développement de WebKitGTK :
#   sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libxdo-dev libssl-dev \
#     libayatana-appindicator3-dev
set -euo pipefail
cd "$(dirname "$0")/.."

# Front construit par beforeBuildCommand, puis intégré à l'exécutable.
(cd app && corepack yarn install --immutable && corepack yarn tauri build --no-bundle)

rm -rf dist/baskul dist/Baskul-Linux.tar.gz
mkdir -p dist/baskul
cp target/release/baskul dist/baskul/
tar -C dist -czf dist/Baskul-Linux.tar.gz baskul
echo
echo "OK : dist/baskul et dist/Baskul-Linux.tar.gz"
