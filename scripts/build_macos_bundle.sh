#!/bin/zsh
set -euo pipefail

repo_root="${0:A:h:h}"
cd "$repo_root"

if [[ "${1:-}" != "--unsigned-preview" && -z "${SAMSUNG_TV_CODESIGN_IDENTITY:-}" ]]; then
    print -u2 'Set SAMSUNG_TV_CODESIGN_IDENTITY to a local code-signing identity.'
    print -u2 'Use --unsigned-preview only to inspect bundle structure and metadata.'
    exit 2
fi

cargo build --release

bundle="$repo_root/target/bundle/Samsung TV Remote.app"
contents="$bundle/Contents"
mkdir -p "$contents/MacOS" "$contents/Resources"
cp "$repo_root/target/release/samsung-tv-remote" "$contents/MacOS/samsung-tv-remote"

cat > "$contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>en</string>
    <key>CFBundleDisplayName</key><string>Samsung TV Remote</string>
    <key>CFBundleExecutable</key><string>samsung-tv-remote</string>
    <key>CFBundleIdentifier</key><string>dev.samsungtvremote.local</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleName</key><string>Samsung TV Remote</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>0.1.0</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>NSLocalNetworkUsageDescription</key>
    <string>Connect to a TV on your local network for pairing and remote control.</string>
</dict>
</plist>
PLIST

plutil -lint "$contents/Info.plist"
if [[ "${1:-}" == "--unsigned-preview" ]]; then
    print 'Unsigned bundle preview created at target/bundle/Samsung TV Remote.app'
else
    codesign --force --sign "$SAMSUNG_TV_CODESIGN_IDENTITY" --timestamp=none "$bundle"
    codesign --verify --strict --verbose=2 "$bundle"
    print 'Locally signed bundle created at target/bundle/Samsung TV Remote.app'
fi
