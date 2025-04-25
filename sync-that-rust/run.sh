#!/bin/bash

SOURCE="/Users/robertwilliams/source"
COUNT="1"
#ARGS="--p2p-lan-enabled"
ARGS="--shared-key --tcp-connect-port 51111"

pushd "$SOURCE/dittotest/sync-that-rust"
cargo build
for i in $(seq 1 $COUNT); do
    mkdir -p "target/debug/$i"
    cp "target/debug/sync-that-rust" "target/debug/$i"
    if [ $i == $COUNT ]; then
        "./target/debug/$i/sync-that-rust" $ARGS $@
    else
        "./target/debug/$i/sync-that-rust" --no-stdin $ARGS $@ &
    fi
done
popd

#trap 'killall sync-that-rust' SIGINT

wait
