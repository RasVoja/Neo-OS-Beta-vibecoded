/*
 * Fortress x86_64 entry, PVH boot protocol.
 *
 * The loader (QEMU -kernel, or a PVH-capable boot loader) jumps to
 * pvh_start32 in 32-bit protected mode with paging off and %ebx pointing
 * at the hvm_start_info structure. We build identity page tables for the
 * first 1 GiB, switch to long mode and call the Rust entry point.
 */

    .section .note.pvh, "a"
    .p2align 2
    .long 4                 /* name size: "Xen\0" */
    .long 4                 /* descriptor size */
    .long 18                /* XEN_ELFNOTE_PHYS32_ENTRY */
    .asciz "Xen"
    .long pvh_start32

    .section .text.boot, "ax"
    .code32
    .global pvh_start32
pvh_start32:
    cli
    cld
    mov %ebx, %edi                  /* keep start_info for kmain (1st arg) */

    /* PML4[0] -> PDPT, PDPT[0] -> PD, PD = 512 x 2 MiB pages = 1 GiB */
    mov $boot_pdpt, %eax
    or  $0x3, %eax
    mov %eax, boot_pml4

    mov $boot_pd, %eax
    or  $0x3, %eax
    mov %eax, boot_pdpt

    xor %ecx, %ecx
1:  mov %ecx, %eax
    shl $21, %eax
    or  $0x83, %eax                 /* present | writable | huge */
    mov %eax, boot_pd(,%ecx,8)
    inc %ecx
    cmp $512, %ecx
    jne 1b

    mov %cr4, %eax                  /* PAE */
    or  $(1 << 5), %eax
    mov %eax, %cr4

    mov $boot_pml4, %eax
    mov %eax, %cr3

    mov $0xC0000080, %ecx           /* EFER.LME */
    rdmsr
    or  $(1 << 8), %eax
    wrmsr

    mov %cr0, %eax                  /* paging + protection */
    or  $0x80000001, %eax
    mov %eax, %cr0

    lgdt boot_gdt_ptr
    ljmp $0x08, $long_mode

    .code64
long_mode:
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    xor %ax, %ax
    mov %ax, %fs
    mov %ax, %gs

    mov $boot_stack_top, %rsp
    xor %rbp, %rbp

    /* zero .bss (page tables live in their own section, not .bss) */
    mov %rdi, %r12
    mov $__bss_start, %rdi
    mov $__bss_end, %rcx
    sub %rdi, %rcx
    xor %eax, %eax
    rep stosb
    mov %r12, %rdi

    call kmain
2:  hlt
    jmp 2b

    .section .rodata
    .p2align 3
boot_gdt:
    .quad 0
    .quad 0x00AF9A000000FFFF        /* 0x08: 64-bit code */
    .quad 0x00CF92000000FFFF        /* 0x10: data */
boot_gdt_end:
boot_gdt_ptr:
    .word boot_gdt_end - boot_gdt - 1
    .long boot_gdt

    .section .data.boot, "aw"
    .p2align 12
boot_pml4: .fill 512, 8, 0
boot_pdpt: .fill 512, 8, 0
boot_pd:   .fill 512, 8, 0
    /* Unmapped once the kernel has its own page tables, so running off the
       bottom of the boot stack faults instead of overwriting memory. */
    .global boot_stack_guard
boot_stack_guard:
    .fill 4096, 1, 0
boot_stack:
    .fill 16384, 1, 0
boot_stack_top:
