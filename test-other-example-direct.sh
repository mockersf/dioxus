#! /bin/bash

git restore test-hotpatching

cargo build --package dioxus-cli
cargo build --package dioxus-cli

random=`openssl rand --hex 20`

(
    sleep 15 &&
    sed -i '' "s/\"[^ ]*\"/\"$random\"/g" test-hotpatching/*/*.rs &&
    sed -i '' "s/\"[^ ]*\"/\"$random\"/g" test-hotpatching/*/*/*.rs &&
    echo "==FILE CHANGED=="
) &

RUST_LOG=trace cargo run --package dioxus-cli -- serve --hot-patch --exit-on-error --package test-hotpatching --example in-other-example-direct
