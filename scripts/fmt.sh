#!/bin/bash

echo ""

echo "Formatting TypeScript code..."
npm run fmt > /dev/null 2>&1

cd ./src-tauri
echo "Formatting Rust code..."
cargo fmt > /dev/null 2>&1
cd ..

echo "Code formatted successfully"