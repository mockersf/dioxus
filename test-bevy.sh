#! /bin/bash

git restore test-hotpatching

cargo build --package dioxus-cli
cargo build --package dioxus-cli

(
    sleep 30 &&
    sed -i '' "s/(0.0)/(1.0)/g" test-hotpatching/with-bevy/src/main.rs &&
    echo "==BIN FILE CHANGED=="
    sleep 15 &&
    sed -i '' "s/(0.0, 0.0, 0.0)/(1.0, 0.0, 1.0)/g" test-hotpatching/bevy-dependency/src/lib.rs &&
    echo "==DEPENDENCY FILE CHANGED==" &&
    sleep 15 &&
    sed -i '' "s/(1.0)/(0.0)/g" test-hotpatching/with-bevy/src/main.rs &&
    echo "==BIN FILE CHANGED BACK=="
) &

cargo run --release --package dioxus-cli -- serve --hot-patch --exit-on-error --package with-bevy
