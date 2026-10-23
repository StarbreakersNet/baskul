#!/usr/bin/env bash
# Construit dist/Baskul.app (et dist/Baskul-macOS.zip) : Baskul et, dans
# Contents/Helpers, son app de réglages (Tauri + Vue).
# Nécessite Rust, Node.js et les outils Xcode en ligne de commande.
#
#   ./scripts/build-macos.sh                 # pour l'architecture de ce Mac
#   UNIVERSAL=1 ./scripts/build-macos.sh     # Apple Silicon + Intel
#   CODESIGN_IDENTITY="Baskul Local" ./scripts/build-macos.sh
#       signe avec un certificat local : les autorisations macOS
#       (Accessibilité) sont alors conservées d'une version à l'autre.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)
APP=dist/Baskul.app
HELPER="$APP/Contents/Helpers/BaskulSettings.app"
ICON=app/src-tauri/icons/icon.icns

rm -rf "$APP" dist/Baskul-macOS.zip
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$HELPER/Contents/MacOS" "$HELPER/Contents/Resources"

# Interface de réglages, intégrée ensuite à l'exécutable Tauri.
(cd app && corepack yarn install --immutable && corepack yarn build)

# build <paquet> <destination> [options cargo…]
build() {
  local pkg=$1 dest=$2
  shift 2
  if [[ "${UNIVERSAL:-0}" == "1" ]]; then
    for target in aarch64-apple-darwin x86_64-apple-darwin; do
      cargo build --release -p "$pkg" --target "$target" "$@"
    done
    lipo -create -output "$dest" \
      "target/aarch64-apple-darwin/release/$pkg" \
      "target/x86_64-apple-darwin/release/$pkg"
  else
    cargo build --release -p "$pkg" "$@"
    cp "target/release/$pkg" "$dest"
  fi
}

if [[ "${UNIVERSAL:-0}" == "1" ]]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
fi
build baskul "$APP/Contents/MacOS/baskul"
build baskul-settings "$HELPER/Contents/MacOS/baskul-settings" --features custom-protocol

sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"
sed "s/@VERSION@/$VERSION/g" packaging/macos/Settings-Info.plist > "$HELPER/Contents/Info.plist"
cp "$ICON" "$APP/Contents/Resources/Baskul.icns"
cp "$ICON" "$HELPER/Contents/Resources/Baskul.icns"

# Signature de l'intérieur vers l'extérieur ; « - » = signature ad hoc
# (aucun certificat nécessaire).
codesign --force --sign "${CODESIGN_IDENTITY:--}" "$HELPER"
codesign --force --sign "${CODESIGN_IDENTITY:--}" "$APP"
codesign --verify --deep --verbose "$APP"

(cd dist && ditto -c -k --keepParent Baskul.app Baskul-macOS.zip)
echo
echo "OK : $APP (version $VERSION)"
echo "Installation : glissez Baskul.app dans /Applications."
