ENTRY(module_init)

SECTIONS
{
    .text : { *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }

    .bss (NOLOAD) :
    {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    }
    .module_info : { KEEP(*(.module_info)) }
    .module_exports : { KEEP(*(.module_exports)) }
}