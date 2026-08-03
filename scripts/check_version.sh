#!/bin/bash

echo ""

version="$(jq -r '.version' package.json)"
echo "[local] package.json: $version"

version="$(jq -r '.version' package-lock.json)"
echo "[local] package-lock.json: $version"

version="$(jq -r '.version' ./kernel/tauri.conf.json)"
echo "[local] kernel/tauri.conf.json: $version"

version="$(sed -n 3p ./kernel/Cargo.toml | cut -c 12-16)"
echo "[local] kernel/Cargo.toml: $version"

version="$(sed -n 3p ./macros/Cargo.toml | cut -c 12-16)"
echo "[local] macros/Cargo.toml: $version"

version="$(sed -n 3p ./extensions/Cargo.toml | cut -c 12-16)"
echo "[local] extensions/Cargo.toml: $version"

version="$(sed -n 2p ./interface/typescript/version.ts | cut -c 32-36)"
echo "[local] interface/typescript/version.ts: $version"

version="$(jq -r '.version' salarixi.version.json)"
echo "[public] salarixi.version.json: $version"