#!/usr/bin/env bash
set -e

if [ -z "$1" ]; then
    echo "Gebruik: ./release.sh <versienummer>  (bijv. 0.2.0)"
    exit 1
fi

VERSIE=$1

echo "==> Versie instellen: $VERSIE"
sed -i "s/^version = .*/version = \"$VERSIE\"/" Cargo.toml

echo "==> Syntaxispagina kopiëren naar dist/..."
cp docs/ecol_syntaxis.html dist/ecol_syntaxis.html

echo "==> Bouwen..."
~/.cargo/bin/trunk build --release --public-url "/ecol/"

echo "==> Committen en taggen..."
git add Cargo.toml CLAUDE.md src/ dist/ docs/
git commit -m "Release v$VERSIE"
git tag "v$VERSIE"

echo ""
echo "Klaar. Vergeet niet: git push && git push --tags"