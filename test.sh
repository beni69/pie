#!/bin/sh
set -xe
cargo check --workspace
cargo test --workspace
cargo build --workspace
