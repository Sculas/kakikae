#!/usr/bin/env just --justfile

set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

[working-directory('crates/stage1')]
stage1:
    cargo build --release
    md -Force build > $null
    cp target/thumbv7m-none-eabi/release/stage1 build/stage1.elf
    elfloader --binary build/stage1.elf build/stage1.bin

clean:
    cd crates/stage1 && cargo clean && cd ../..
