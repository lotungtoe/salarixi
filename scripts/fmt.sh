#!/bin/bash

echo ""

echo "Formatting TypeScript code..."
npx tsfmt -r > /dev/null 2>&1

echo "Formatting Rust code..."
cargo fmt > /dev/null 2>&1

echo "Code formatted successfully"
