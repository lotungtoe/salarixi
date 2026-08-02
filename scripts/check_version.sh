#!/bin/bash

echo ""

version="$(jq -r '.version' package.json)"
echo "[local] package.json: $version"

version="$(jq -r '.version' package-lock.json)"
echo "[local] package-lock.json: $version"

version="$(jq -r '.version' ./src-tauri/tauri.conf.json)"
echo "[local] src-tauri/tauri.conf.json: $version"

version="$(sed -n 3p ./src-tauri/Cargo.toml | cut -c 12-16)"
echo "[local] src-tauri/Cargo.toml: $version"

version="$(sed -n 2p ./src/typescript/version.ts | cut -c 32-36)"
echo "[local] src/typescript/version.ts: $version"

version="$(jq -r '.version' salarixi.version.json)"
echo "[public] salarixi.version.json: $version"