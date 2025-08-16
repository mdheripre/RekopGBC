#!/bin/bash

rm -rf rekopGBC.exe
cargo fmt
cargo build
cp target/debug/rekop_gbc.exe .