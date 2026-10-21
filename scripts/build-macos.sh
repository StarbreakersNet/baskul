#!/usr/bin/env bash
# Construit dist/Bascule.app (et dist/Bascule-macOS.zip).
#
#   ./scripts/build-macos.sh                 # pour l'architecture de ce Mac
#   UNIVERSAL=1 ./scripts/build-macos.sh     # Apple Silicon + Intel
#   CODESIGN_IDENTITY="Bascule Local" ./scripts/build-macos.sh
#       signe avec un certificat local : les autorisations macOS
#       (Accessibilité) sont alors conservées d'une version à l'autre.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
APP=dist/Bascule.app
BIN="$APP/Contents/MacOS/bascule"

rm -rf "$APP" dist/Bascule-macOS.zip
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

if [[ "${UNIVERSAL:-0}" == "1" ]]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  cargo build --release --target aarch64-apple-darwin
  cargo build --release --target x86_64-apple-darwin
  lipo -create -output "$BIN" \
    target/aarch64-apple-darwin/release/bascule \
    target/x86_64-apple-darwin/release/bascule
else
  cargo build --release
  cp target/release/bascule "$BIN"
fi

sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"

# Signature : « - » = signature ad hoc (aucun certificat nécessaire).
codesign --force --deep --sign "${CODESIGN_IDENTITY:--}" "$APP"
codesign --verify --verbose "$APP"

(cd dist && ditto -c -k --keepParent Bascule.app Bascule-macOS.zip)
echo
echo "OK : $APP (version $VERSION)"
echo "Installation : glissez Bascule.app dans /Applications."
