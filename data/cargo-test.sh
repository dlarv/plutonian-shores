#!/bin/bash
export RUSTFLAGS=-Awarnings
cargo test -- --nocapture --test-threads=1
