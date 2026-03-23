#!/usr/bin/env bash
set -e

if [ -z "$1" ]; then
    echo "Gebruik: ./release.sh <versienummer>  (bijv. 0.2.0)"
    exit 1
fi

VERSIE=$1

echo "==> Versie instellen: $VERSIE"
sed -i "s/^version = .*/version = \"$VERSIE\"/" Cargo.toml

echo "==> Bouwen..."
trunk build --release

echo "==> Versie invullen in syntaxisreferentie..."
sed -i "s/@@VERSIE@@/$VERSIE/g" dist/ecol_syntaxis.html

echo "==> Committen en taggen..."
git add Cargo.toml dist/ docs/
git commit -m "Release v$VERSIE"
git tag "v$VERSIE"

echo ""
echo "Klaar. Vergeet niet: git push && git push --tags"