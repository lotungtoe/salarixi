#!/bin/bash

echo ""

version="$(jq -r '.version' package.json)"
echo "- package.json: $version"

version="$(jq -r '.version' package-lock.json)"
echo "- package-lock.json: $version"

version="$(jq -r '.version' ./src-tauri/tauri.conf.json)"
echo "- src-tauri/tauri.conf.json: $version"

version="$(sed -n 3p ./src-tauri/Cargo.toml | cut -c 12-16)"
echo "- src-tauri/Cargo.toml: $version"

version="$(sed -n 2p ./src/typescript/version.ts | cut -c 32-36)"
echo "- src/typescript/version.ts: $version"