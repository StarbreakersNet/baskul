#!/usr/bin/env bash
# Construit dist/Baskul.app (et dist/Baskul-macOS.zip) avec Tauri : un seul
# exécutable, Baskul et sa fenêtre de réglages (Vue 3).
# Nécessite Rust, Node.js et les outils Xcode en ligne de commande.
#
#   ./scripts/build-macos.sh                 # pour l'architecture de ce Mac
#   UNIVERSAL=1 ./scripts/build-macos.sh     # Apple Silicon + Intel
#   CODESIGN_IDENTITY="Baskul" ./scripts/build-macos.sh
#       signe avec un certificat (nom ou empreinte SHA-1) : les autorisations
#       macOS (Accessibilité) sont alors conservées d'une version à l'autre.
#       C'est ce que fait le CI avec le secret MACOS_CERTIFICATE.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)
APP=dist/Baskul.app

rm -rf "$APP" dist/Baskul-macOS.zip
mkdir -p dist

if [[ "${UNIVERSAL:-0}" == "1" ]]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  target=(--target universal-apple-darwin)
  out=target/universal-apple-darwin/release/bundle/macos/Baskul.app
else
  target=()
  out=target/release/bundle/macos/Baskul.app
fi
# Bundle par Tauri (front construit par beforeBuildCommand). La signature est
# faite ensuite, ici, avec le certificat choisi.
(cd app && corepack yarn install --immutable && corepack yarn tauri build --bundles app ${target[@]+"${target[@]}"})
cp -R "$out" "$APP"

# « - » = signature ad hoc (aucun certificat nécessaire).
codesign --force --sign "${CODESIGN_IDENTITY:--}" "$APP"
codesign --verify --deep --verbose "$APP"

(cd dist && ditto -c -k --keepParent Baskul.app Baskul-macOS.zip)
echo
echo "OK : $APP (version $VERSION)"
echo "Installation : glissez Baskul.app dans /Applications."
