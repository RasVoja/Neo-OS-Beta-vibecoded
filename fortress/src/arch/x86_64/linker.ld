/* Fortress on x86_64: loaded by QEMU (or any PVH loader) at 1 MiB, identity mapped. */
ENTRY(pvh_start32)

PHDRS
{
    note   PT_NOTE;
    text   PT_LOAD FLAGS(5);   /* R X */
    rodata PT_LOAD FLAGS(4);   /* R   */
    data   PT_LOAD FLAGS(6);   /* R W */
}

SECTIONS
{
    . = 1M;
    __kernel_start = .;

    .note.pvh : { KEEP(*(.note.pvh)) } :text :note

    .text : ALIGN(4K) { KEEP(*(.text.boot)) *(.text .text.*) } :text

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
