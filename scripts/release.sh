#!/usr/bin/env bash
set -euo pipefail

TYPE=${1:-}

if [ -z "$TYPE" ]; then
  echo "Usage: ./scripts/release.sh [patch|minor|major]"
  exit 1
fi

case "$TYPE" in
  patch|minor|major)
    ;;
  *)
    echo "Invalid release type: $TYPE"
    echo "Usage: ./scripts/release.sh [patch|minor|major]"
    exit 1
    ;;
esac

cargo set-version --workspace --bump "$TYPE"

VERSION=$(awk '
  /^\[workspace\.package\]$/ { in_section=1; next }
  /^\[/ && in_section { in_section=0 }
  in_section && /^version\s*=\s*"[^"]+"/ {
    gsub(/.*"|".*/, "", $0)
    print
    exit
  }
' Cargo.toml)

if [ -z "$VERSION" ]; then
  echo "Could not read workspace version from Cargo.toml"
  exit 1
fi

git add Cargo.toml crates/*/Cargo.toml Cargo.lock
git commit -m "chore(release): v$VERSION"
git tag "v$VERSION"

echo "Released v$VERSION"
