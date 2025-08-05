#!/bin/bash

cargo fmt
cargo build
cp target/debug/rekopGBC.exe .