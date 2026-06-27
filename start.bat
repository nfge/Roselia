@echo off

if "%1"=="build" ( 
    if "%2"=="dev" goto :build_dev
    if "%2"=="release" goto :build_release
    goto :build_dev
)
if "%1"=="run" (
    if "%2"=="dev" goto :run_dev
    if "%2"=="release" goto :run_release
    goto :run_dev
)
exit /b

:run_dev
call :build_dev
goto run

:run_release
call :build_release
goto run


:build_dev
cargo +nightly build -p kernel --target x86_64.json 
cargo build -p bootloader --target x86_64-unknown-uefi

if not exist os\ mkdir os
if not exist os\EFI\BOOT mkdir os\EFI\BOOT
if not exist os\EFI\Roselia mkdir os\EFI\Roselia
if not exist os\OVMF\ mkdir os\OVMF\


if not exist os\OVMF\vars.fd copy "C:\Program Files\qemu\share\OVMF_VARS.fd" os\OVMF\vars.fd
copy target\x86_64\debug\kernel os\kernel.elf
copy target\x86_64-unknown-uefi\debug\bootloader.efi os\EFI\BOOT\BOOTX64.efi
copy target\x86_64-unknown-uefi\release\bootloader.efi os\EFI\Roselia\boot.efi
exit /b


:build_release
cargo +nightly build -p kernel --target x86_64.json --release
cargo build -p bootloader --target x86_64-unknown-uefi --release

if not exist os\ mkdir os
if not exist os\EFI\BOOT mkdir os\EFI\BOOT
if not exist os\EFI\Roselia mkdir os\EFI\Roselia
if not exist os\OVMF\ mkdir os\OVMF\


if not exist os\OVMF\vars.fd copy "C:\Program Files\qemu\share\OVMF_VARS.fd" os\OVMF\vars.fd
copy target\x86_64\release\kernel os\kernel.elf
copy target\x86_64-unknown-uefi\release\bootloader.efi os\EFI\BOOT\BOOTX64.efi
copy target\x86_64-unknown-uefi\release\bootloader.efi os\EFI\Roselia\boot.efi
exit /b


:run
qemu-system-x86_64 -machine q35 -m 218 -drive if=pflash,format=raw,readonly=on,file="C:\Program Files\qemu\share\OVMF_CODE.fd" -drive if=pflash,format=raw,file=./os/OVMF/vars.fd -drive format=raw,file=fat:rw:os/ -rtc clock=host,base=utc -cpu qemu64,+x2apic -d int,cpu_reset,guest_errors -D qemu.log
goto :eof