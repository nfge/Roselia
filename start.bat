@echo off

if "%1"=="release" goto release
goto dev


:dev
call :build_dev
goto run

:release
call :build_release
goto run


:build_dev
cargo +nightly build -p kernel --target x86_64.json 
cargo build -p bootloader --target x86_64-unknown-uefi

if not exist os\ mkdir os
if not exist os\EFI\BOOT mkdir os\EFI\BOOT

copy target\x86_64\debug\kernel os\kernel.elf
copy target\x86_64-unknown-uefi\debug\bootloader.efi os\EFI\BOOT\BOOTX64.efi
exit /b


:build_release
cargo +nightly build -p kernel --target x86_64.json --release
cargo build -p bootloader --target x86_64-unknown-uefi --release

if not exist os\ mkdir os
if not exist os\EFI\BOOT mkdir os\EFI\BOOT

copy target\x86_64\release\kernel os\kernel.elf
copy target\x86_64-unknown-uefi\release\bootloader.efi os\EFI\BOOT\BOOTX64.efi
exit /b


:run
qemu-system-x86_64 -m 512 -bios OVMF_CODE.fd -drive format=raw,file=fat:rw:os/ -rtc clock=host,base=utc -cpu qemu64,+x2apic -d int,cpu_reset,guest_errors -D qemu.log
goto :eof