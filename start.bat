@echo off

cargo +nightly build -p kernel --target x86_64.json
cargo build -p bootloader --target x86_64-unknown-uefi

if not exist os\ (
    mkdir os
)
if not exist os\EFI\BOOT\ (
    mkdir os\EFI\BOOT
)

copy target\x86_64\debug\kernel os\kernel.elf
copy target\x86_64-unknown-uefi\debug\bootloader.efi os\EFI\BOOT\BOOTX64.efi


qemu-system-x86_64 -m 218 -bios OVMF_CODE.fd -drive format=raw,file=fat:rw:os/ -rtc clock=host,base=utc -cpu qemu64,+x2apic