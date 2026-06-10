#!/usr/bin/env bash

set -e

TYPE=$1
MODE=$2

if [[ "$TYPE" == "build" ]]; then
    CMD="build"
elif [[ "$TYPE" == "run" ]]; then
    CMD="run"
else
    echo "Usage: $0 {build|run} {dev|release}"
    exit 1
fi

if [[ "$MODE" == "release" ]]; then
    BUILD_MODE="release"
elif [[ "$MODE" == "dev" ]]; then
    BUILD_MODE="dev"
else
    echo "Usage: $0 {build|run} {dev|release}"
    exit 1
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

if [[ "$CMD" == "build" ]]; then
    if [[ "$BUILD_MODE" == "release" ]]; then
        build_release
    elif [[ "$BUILD_MODE" == "dev" ]]; then
        build_dev
    else
        exit 1
    fi
elif [[ "$CMD" == "run" ]]; then
    if [[ "$BUILD_MODE" == "release" ]]; then
        build_release
        run_qemu
    elif [[ "$BUILD_MODE" == "dev" ]]; then
        build_dev
        run_qemu
    else
        exit 1
    fi
else
    exit 1
fi
