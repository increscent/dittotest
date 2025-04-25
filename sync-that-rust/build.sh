#!/bin/bash

set -e

SOURCE="/Users/robertwilliams/source"

pushd "$SOURCE/ditto/ffi"
cargo build --release
popd

cp "$SOURCE/ditto/target/release/libdittoffi.a" "$SOURCE/dittotest/sync-that-rust/"

pushd "$SOURCE/dittotest/sync-that-rust"
cargo clean
cargo build
popd
