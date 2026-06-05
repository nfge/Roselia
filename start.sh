#!/usr/bin/env bash

set -e

MODE=$1

if [[ "$MODE" == "release" ]]; then
    BUILD_MODE="release"
else
    BUILD_MODE="debug"
fi

build_dev() {
    cargo +nightly build -p kernel --target x86_64.json
    cargo build -p bootloader --target x86_64-unknown-uefi

    mkdir -p os/EFI/BOOT

    cp target/x86_64/debug/kernel os/kernel.elf
    cp target/x86_64-unknown-uefi/debug/bootloader.efi os/EFI/BOOT/BOOTX64.efi
}

build_release() {
    cargo +nightly build -p kernel --target x86_64.json --release
    cargo build -p bootloader --target x86_64-unknown-uefi --release

    mkdir -p os/EFI/BOOT

    cp target/x86_64/release/kernel os/kernel.elf
    cp target/x86_64-unknown-uefi/release/bootloader.efi os/EFI/BOOT/BOOTX64.efi
}

run_qemu() {
    qemu-system-x86_64 \
        -machine q35 \
        -m 218 \
        -bios usr/share/OVMF/OVMF_CODE.fd \
        -boot menu=on \
        -drive format=raw,file=fat:rw:os/ \
        -rtc clock=host,base=utc \
        -cpu qemu64,+x2apic \
        -d int,cpu_reset,guest_errors \
        -D qemu.log
}

if [[ "$BUILD_MODE" == "release" ]]; then
    build_release
else
    build_dev
fi

run_qemu