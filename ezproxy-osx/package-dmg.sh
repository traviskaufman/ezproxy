#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/EZProxyOSX"

BUILD_DIR="$(mktemp -d)"
STAGING_DIR="$(mktemp -d)"
trap 'rm -rf "$BUILD_DIR" "$STAGING_DIR"' EXIT

xcodebuild -scheme EZProxyOSX -configuration Release -derivedDataPath "$BUILD_DIR" \
  -allowProvisioningUpdates build

APP="$BUILD_DIR/Build/Products/Release/EZProxyOSX.app"
VERSION="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$APP/Contents/Info.plist")"
DMG="$(cd .. && pwd)/EZProxy-$VERSION.dmg"

cp -R "$APP" "$STAGING_DIR/"
ln -s /Applications "$STAGING_DIR/Applications"

rm -f "$DMG"
hdiutil create -volname "EZProxy" -srcfolder "$STAGING_DIR" -ov -format UDZO "$DMG"
echo "Created $DMG"
