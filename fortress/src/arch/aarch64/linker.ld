/* Fortress on AArch64 (QEMU virt): loaded as an ELF file at 2 MiB into RAM.
   RAM starts at 1 GiB; QEMU puts the device tree at its very start. */
ENTRY(_start)

PHDRS
{
    text   PT_LOAD FLAGS(5);   /* R X */
    rodata PT_LOAD FLAGS(4);   /* R   */
    data   PT_LOAD FLAGS(6);   /* R W */
}

SECTIONS
{
    . = 0x40200000;
    __kernel_start = .;

    .text : { KEEP(*(.text.boot)) *(.text .text.*) } :text

    .rodata : ALIGN(4K) { *(.rodata .rodata.*) } :rodata

    .data : ALIGN(4K) { *(.data .data.*) *(.got .got.*) } :data

    .bss : ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*) *(COMMON)
        __bss_end = .;
    } :data

    . = ALIGN(4K);
    __kernel_end = .;

    /DISCARD/ : { *(.eh_frame*) *(.comment) }
}
