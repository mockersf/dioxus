#! /bin/bash

cargo build --package dioxus-cli

random=`openssl rand -base64 12`

(sleep 10 && sed -i '' "s/\"[^ ]*\"/\"$random\"/g" test-hotpatching/src/*.rs) &

RUST_LOG=trace cargo run --package dioxus-cli -- serve --hot-patch --exit-on-error --package test-hotpatching
