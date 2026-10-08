#!/bin/zsh
set -euo pipefail

repo_root="${0:A:h:h}"
keychain="$(security default-keychain -d user | tr -d '"' | xargs)"
if [[ -z "$keychain" ]]; then
    print -u2 'No login Keychain was found.'
    exit 1
fi

existing="$(security find-identity -p codesigning "$keychain" | sed -n 's/.*\([0-9A-F]\{40\}\) "Samsung TV Remote Local Test".*/\1/p' | head -1)"
if [[ -n "$existing" ]]; then
    mkdir -p "$repo_root/target"
    print -r -- "$existing" > "$repo_root/target/local-signing-identity.txt"
    print 'Existing local signing identity found in login Keychain.'
    print 'Run: export SAMSUNG_TV_CODESIGN_IDENTITY="$(cat target/local-signing-identity.txt)"'
    exit 0
fi

temporary="$(mktemp -d "${TMPDIR:-/tmp}/samsung-tv-remote-signing.XXXXXX")"
trap 'rm -rf "$temporary"' EXIT
chmod 700 "$temporary"

openssl req -x509 -newkey rsa:3072 -nodes -days 3650 \
    -subj '/CN=Samsung TV Remote Local Test/O=Local Development/' \
    -addext 'keyUsage=critical,digitalSignature' \
    -addext 'extendedKeyUsage=codeSigning' \
    -keyout "$temporary/key.pem" \
    -out "$temporary/cert.pem" 2> "$temporary/openssl.log"
openssl rand -hex 24 > "$temporary/password.txt"
chmod 600 "$temporary/password.txt"
openssl pkcs12 -export -legacy \
    -inkey "$temporary/key.pem" \
    -in "$temporary/cert.pem" \
    -out "$temporary/identity.p12" \
    -passout "file:$temporary/password.txt"

security import "$temporary/identity.p12" -k "$keychain" -P "$(cat "$temporary/password.txt")" -T /usr/bin/codesign
identity="$(security find-identity -p codesigning "$keychain" | sed -n 's/.*\([0-9A-F]\{40\}\) "Samsung TV Remote Local Test".*/\1/p' | head -1)"
if [[ -z "$identity" ]]; then
    print -u2 'The certificate was imported but macOS could not find it for code signing.'
    exit 1
fi

mkdir -p "$repo_root/target"
print -r -- "$identity" > "$repo_root/target/local-signing-identity.txt"
print 'Local signing identity imported into login Keychain.'
print 'Run: export SAMSUNG_TV_CODESIGN_IDENTITY="$(cat target/local-signing-identity.txt)"'
print 'Then: zsh scripts/build_macos_bundle.sh'
