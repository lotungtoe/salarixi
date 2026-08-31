#!/bin/bash

echo "[build] Compiling code without creating bundles..."

npm run tauri build -- --no-bundle > /dev/null 2>&1

echo "[build] Code compiled successfully!"

echo "[upx] Compressing executable file..."

cd ./target/release
upx --best salarixi > /dev/null 2>&1
cd ../..
rm -rf build
mkdir build
mv ./target/release/salarixi build

echo "[upx] Executable file compressed successfully!"

echo "[package:deb] Extracting information from salarixi.control.json..."

name="$(jq -r '.name' ./salarixi.control.json)"
version="$(jq -r '.version' ./salarixi.control.json)"
architecture="$(jq -r '.architecture' ./salarixi.control.json)"
maintainer="$(jq -r '.maintainer' ./salarixi.control.json)"
depends="$(jq -r '.depends' ./salarixi.control.json)"
description="$(jq -r '.description' ./salarixi.control.json)"

# Debian ещё требует компонент RRR, но я без
# понятия, обязателен он или нет, пока так
fullname="$name""_$version""_$architecture"

echo "[package:deb] Information extracted!"

echo "[package:deb] Organizing the package file structure..."

cd ./build
mkdir deb
cd ./deb

mkdir "$fullname"
cd ./"$fullname"

mkdir DEBIAN
mkdir usr
mkdir usr/bin
mkdir usr/share
mkdir usr/share/applications
mkdir usr/share/icons
mkdir usr/share/icons/hicolor
mkdir usr/share/icons/hicolor/512x512
mkdir usr/share/icons/hicolor/512x512/apps

cd ./DEBIAN
packageinf="""Package: $name
Version: $version
Architecture: $architecture
Maintainer: $maintainer
Priority: optional
Depends: $depends
Description: $description
 (none)
"""
touch control && echo "$packageinf" > control
cd ..

cd ./usr/share/applications
desktopinf="""[Desktop Entry]
Categories=
Comment=$description
Exec=$name
StartupWMClass=$name
Icon=$name
Name=$name
Terminal=false
Type=Application
"""
touch "$name.desktop" && echo "$desktopinf" > "$name.desktop"
cd ..

cp ../../../../../app/icons/icon.png ./icons/hicolor/512x512/apps/$name.png
cd ..

cp ../../../salarixi ./bin/salarixi
cd ../..

echo "[package:deb] File structure successfully organized!"

echo "[package:deb] Creating a package ($fullname.deb)..."

dpkg-deb --root-owner-group --build "$fullname" > /dev/null 2>&1

echo "[package:deb] Package successfully created!"

# TODO: Реализовать создания .AppImage/.rpm пакетов
