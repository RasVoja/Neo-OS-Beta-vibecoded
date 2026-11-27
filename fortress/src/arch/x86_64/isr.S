/*
 * Interrupt entry stubs for all 256 vectors (0-31 are CPU exceptions). Every stub leaves the same frame:
 * a vector number and an error code (0 when the CPU pushes none) on top of
 * the CPU's own interrupt frame, then isr_common saves all general registers
 * and hands a pointer to the whole frame to exception_dispatch in Rust.
 */

    .section .text
    .code64

    .macro ISR_NOERR n
isr_\n:
    pushq $0
    pushq $\n
    jmp isr_common
    .endm

    .macro ISR_ERR n
isr_\n:
    pushq $\n
    jmp isr_common
    .endm

    .irp n, 0,1,2,3,4,5,6,7,9,15,16,18,19,20,22,23
    ISR_NOERR \n
    .endr
    .irp n, 24,25,26,27,28,31,32,33,34,35,36,37,38,39,40,41
    ISR_NOERR \n
    .endr
    .irp n, 42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57
    ISR_NOERR \n
    .endr
    .irp n, 58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73
    ISR_NOERR \n
    .endr
    .irp n, 74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89
    ISR_NOERR \n
    .endr
    .irp n, 90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105
    ISR_NOERR \n
    .endr
    .irp n, 106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121
    ISR_NOERR \n
    .endr
    .irp n, 122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137
    ISR_NOERR \n
    .endr
    .irp n, 138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153
    ISR_NOERR \n
    .endr
    .irp n, 154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169
    ISR_NOERR \n
    .endr
    .irp n, 170,171,172,173,174,175,176,177,178,179,180,181,182,183,184,185
    ISR_NOERR \n
    .endr
    .irp n, 186,187,188,189,190,191,192,193,194,195,196,197,198,199,200,201
    ISR_NOERR \n
    .endr
    .irp n, 202,203,204,205,206,207,208,209,210,211,212,213,214,215,216,217
    ISR_NOERR \n
    .endr
    .irp n, 218,219,220,221,222,223,224,225,226,227,228,229,230,231,232,233
    ISR_NOERR \n
    .endr
    .irp n, 234,235,236,237,238,239,240,241,242,243,244,245,246,247,248,249
    ISR_NOERR \n
    .endr
    .irp n, 250,251,252,253,254,255
    ISR_NOERR \n
    .endr
    .irp n, 8,10,11,12,13,14,17,21,29,30
    ISR_ERR \n
    .endr

isr_common:
    push %rax
    push %rbx
    push %rcx
    push %rdx
    push %rsi
    push %rdi
    push %rbp
    push %r8
    push %r9
    push %r10
    push %r11
    push %r12
    push %r13
    push %r14
    push %r15
    mov %rsp, %rdi
    cld
    call exception_dispatch
    pop %r15
    pop %r14
    pop %r13
    pop %r12
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rbp
    pop %rdi
    pop %rsi
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax
    add $16, %rsp           /* vector + error code */
    iretq

    .section .rodata
    .p2align 3
    .global isr_table
isr_table:
    .irp n, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15
    .quad isr_\n
    .endr
    .irp n, 16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
    .quad isr_\n
    .endr
    .irp n, 32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47
    .quad isr_\n
    .endr
    .irp n, 48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63
    .quad isr_\n
    .endr
    .irp n, 64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79
    .quad isr_\n
    .endr
    .irp n, 80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95
    .quad isr_\n
    .endr
    .irp n, 96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111
    .quad isr_\n
    .endr
    .irp n, 112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127
    .quad isr_\n
    .endr
    .irp n, 128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143
    .quad isr_\n
    .endr
    .irp n, 144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159
    .quad isr_\n
    .endr
    .irp n, 160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175
    .quad isr_\n
    .endr
    .irp n, 176,177,178,179,180,181,182,183,184,185,186,187,188,189,190,191
    .quad isr_\n
    .endr
    .irp n, 192,193,194,195,196,197,198,199,200,201,202,203,204,205,206,207
    .quad isr_\n
    .endr
    .irp n, 208,209,210,211,212,213,214,215,216,217,218,219,220,221,222,223
    .quad isr_\n
    .endr
    .irp n, 224,225,226,227,228,229,230,231,232,233,234,235,236,237,238,239
    .quad isr_\n
    .endr
    .irp n, 240,241,242,243,244,245,246,247,248,249,250,251,252,253,254,255
    .quad isr_\n
    .endr
