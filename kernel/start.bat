@echo off

cargo bootimage

IF EXIST target\x86_64\debug\bootimage-kernel.bin (
    DEL /S /Q "target\x86_64\debug\kernel.bin"
    rename target\x86_64\debug\bootimage-kernel.bin kernel.bin
)
qemu-system-x86_64 -drive format=raw,file=target/x86_64/debug/kernel.bin