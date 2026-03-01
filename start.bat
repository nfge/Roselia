@echo off

cargo bootimage -p kernel
cargo build -p bootloader --target x86_64-unknown-uefi