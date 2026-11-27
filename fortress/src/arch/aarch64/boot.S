// Entry point on AArch64. QEMU starts every core here; core 0 boots, the
// others wait. x0 holds the device tree address when the loader passes one
// (Linux boot protocol); it is kept untouched until kmain(x0, x1).
//
// The kernel starts with the 64-byte header of the arm64 Linux boot
// protocol ("Image" format), so that loaders such as QEMU, U-Boot and the
// Raspberry Pi firmware can boot the raw binary (scripts/elf2image.py) and
// pass the device tree and a boot image (initrd). The ELF file still boots
// too: its first instruction just jumps over the header.

.pushsection .text.boot, "ax"
.global _start
_start:
    b       start                   // code0
    .long   0                       // code1
    .quad   0x200000                // text_offset: 2 MiB above RAM start
    .quad   __kernel_end - _start   // image_size, with .bss
    .quad   0x2                     // flags: little endian, 4 KiB pages
    .quad   0, 0, 0                 // reserved
    .ascii  "ARM\x64"              // magic
    .long   0                       // reserved

start:
    mrs     x9, mpidr_el1
    and     x9, x9, #0xff
    cbnz    x9, park

    // Started in EL2 (virtualization on)? Drop to EL1. x1 keeps the
    // level we started in, for kmain's report.
    mrs     x1, CurrentEL
    lsr     x1, x1, #2
    cmp     x1, #2
    b.ne    in_el1
    mov     x9, #(1 << 31)          // HCR_EL2.RW: EL1 runs AArch64
    msr     hcr_el2, x9
    mov     x9, #3                  // EL1 may use the physical counter and timer
    msr     cnthctl_el2, x9
    msr     cntvoff_el2, xzr        // virtual time = physical time
    mov     x9, #0x3c5              // EL1h, all interrupts masked
    msr     spsr_el2, x9
    adr     x9, in_el1
    msr     elr_el2, x9
    eret

in_el1:
    mov     x9, #(3 << 20)          // CPACR_EL1.FPEN: FP/SIMD allowed
    msr     cpacr_el1, x9
    isb

    adrp    x9, boot_stack_top
    add     x9, x9, :lo12:boot_stack_top
    mov     sp, x9

    adrp    x9, __bss_start
    add     x9, x9, :lo12:__bss_start
    adrp    x10, __bss_end
    add     x10, x10, :lo12:__bss_end
1:  cmp     x9, x10
    b.hs    2f
    str     xzr, [x9], #8
    b       1b
2:
    // Identity map the first 4 GiB with 1 GiB blocks: the first as device
    // memory (UART, interrupt controller), the rest as normal cached RAM.
    adrp    x9, boot_l1
    add     x9, x9, :lo12:boot_l1
    mov     x10, #0x401             // block, AF, attr 0 (device)
    str     x10, [x9]
    mov     x11, #1
    mov     x12, #0x705             // block, AF, inner shareable, attr 1 (RAM)
3:  lsl     x10, x11, #30
    orr     x10, x10, x12
    str     x10, [x9, x11, lsl #3]
    add     x11, x11, #1
    cmp     x11, #4
    b.lo    3b

    mov     x10, #0xff00            // attr 0 device-nGnRnE, attr 1 write-back
    msr     mair_el1, x10
    // T0SZ 25 (39-bit, walk from level 1), 4 KiB granule, inner shareable
    // write-back walks, TTBR1 walks off, 4 GiB physical range.
    ldr     x10, =(25 | (1 << 8) | (1 << 10) | (3 << 12) | (1 << 23))
    msr     tcr_el1, x10
    msr     ttbr0_el1, x9
    dsb     ish
    isb
    mrs     x10, sctlr_el1
    orr     x10, x10, #1            // M: MMU on
    orr     x10, x10, #(1 << 2)     // C: data cache
    orr     x10, x10, #(1 << 12)    // I: instruction cache
    bic     x10, x10, #(1 << 1)     // A: no alignment checks
    msr     sctlr_el1, x10
    isb

    bl      kmain
park:
    wfe
    b       park
.ltorg
.popsection

.pushsection .bss.boot, "aw", @nobits
.balign 4096
.global boot_l1
boot_l1:
    .space 4096
// A small stack for reporting a kernel stack overflow (see vectors.S).
.balign 16
    .space 16 * 1024
.global emergency_stack_top
emergency_stack_top:
.balign 4096
.global boot_stack_guard
boot_stack_guard:                   // left unmapped by the kernel page tables
    .space 4096
boot_stack:
    .space 64 * 1024
.global boot_stack_top
boot_stack_top:
.popsection
