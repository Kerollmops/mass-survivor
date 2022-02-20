#!/usr/bin/env bash

set -e

if [ ! -f "Cargo.toml" ]; then
    echo "Please, run this script at the root of the repository"
    exit 1
fi

# compile and prepare the project for web target
wasm-pack build --target web

# create the folder to upload on itch.io (assets folder, wasm and js files)
rm -rf release/
mkdir release
cp -R assets release/
cp itch.io/index.html release/
cp pkg/mass_survivor_bg.wasm release/mass_survivor.wasm
cp pkg/mass_survivor.js release/mass_survivor.js

# zip the content to send to itch.io
zip -r release.zip release

echo "You can upload the release.zip file to itch.io"
