#!/bin/bash

echo ""

echo "Formatting TypeScript code..."
npm run fmt > /dev/null 2>&1

echo "Formatting Rust code..."
cd ./kernel
cargo fmt > /dev/null 2>&1
cd ../macros
cargo fmt > /dev/null 2>&1
cd ../extensions
cargo fmt > /dev/null 2>&1
cd ..

echo "Code formatted successfully"