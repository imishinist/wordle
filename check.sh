#!/bin/bash

cargo test
cargo check
cargo fmt -- --check
cargo clippy -- -D warnings

