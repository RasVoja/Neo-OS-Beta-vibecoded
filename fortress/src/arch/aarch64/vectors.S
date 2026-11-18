// Exception vectors (VBAR_EL1). Every entry saves x0-x30, ELR, SPSR and
// the FP/SIMD registers in a frame on the stack and calls
// exception(frame, kind) in Rust; kind is the entry number 0..15 (4 groups
// of sync, IRQ, FIQ, SError). The kernel itself uses FP/SIMD registers, so
// an interrupt must keep them as they were.

.set FRAME, 816                     // see Frame in mod.rs
.set FP_AT, 288

.macro ENTRY kind
.balign 0x80
    sub     sp, sp, #FRAME
    stp     x0, x1, [sp, #(0 * 8)]
    mov     x1, #\kind
    b       save_rest
.endm

// For exceptions taken in the kernel (EL1, SP_EL1): if the frame would not
// fit on the current stack (it ran into a guard page), switch to the
// emergency stack first, so the overflow can still be reported.
.macro ENTRY_KERNEL kind
.balign 0x80
    msr     tpidr_el1, x0
    mov     x0, sp
    sub     x0, x0, #FRAME
    at      s1e1w, x0
    isb
    mrs     x0, par_el1
    tbz     x0, #0, 1f
    adrp    x0, emergency_stack_top
    add     x0, x0, :lo12:emergency_stack_top
    mov     sp, x0
1:  mrs     x0, tpidr_el1
    sub     sp, sp, #FRAME
    stp     x0, x1, [sp, #(0 * 8)]
    mov     x1, #\kind
    b       save_rest
.endm

.pushsection .text.vectors, "ax"
.balign 2048
.global exception_vectors
exception_vectors:
    ENTRY 0
    ENTRY 1
    ENTRY 2
    ENTRY 3
    ENTRY_KERNEL 4
    ENTRY_KERNEL 5
    ENTRY_KERNEL 6
    ENTRY_KERNEL 7
    ENTRY 8
    ENTRY 9
    ENTRY 10
    ENTRY 11
    ENTRY 12
    ENTRY 13
    ENTRY 14
    ENTRY 15

save_rest:
    stp     x2, x3, [sp, #(2 * 8)]
    stp     x4, x5, [sp, #(4 * 8)]
    stp     x6, x7, [sp, #(6 * 8)]
    stp     x8, x9, [sp, #(8 * 8)]
    stp     x10, x11, [sp, #(10 * 8)]
    stp     x12, x13, [sp, #(12 * 8)]
    stp     x14, x15, [sp, #(14 * 8)]
    stp     x16, x17, [sp, #(16 * 8)]
    stp     x18, x19, [sp, #(18 * 8)]
    stp     x20, x21, [sp, #(20 * 8)]
    stp     x22, x23, [sp, #(22 * 8)]
    stp     x24, x25, [sp, #(24 * 8)]
    stp     x26, x27, [sp, #(26 * 8)]
    stp     x28, x29, [sp, #(28 * 8)]
    mrs     x2, elr_el1
    mrs     x3, spsr_el1
    stp     x30, x2, [sp, #(30 * 8)]
    mrs     x5, sp_el0              // the program's stack pointer
    stp     x3, x5, [sp, #(32 * 8)]
    add     x4, sp, #FP_AT
    stp     q0, q1, [x4, #(0 * 32)]
    stp     q2, q3, [x4, #(1 * 32)]
    stp     q4, q5, [x4, #(2 * 32)]
    stp     q6, q7, [x4, #(3 * 32)]
    stp     q8, q9, [x4, #(4 * 32)]
    stp     q10, q11, [x4, #(5 * 32)]
    stp     q12, q13, [x4, #(6 * 32)]
    stp     q14, q15, [x4, #(7 * 32)]
    stp     q16, q17, [x4, #(8 * 32)]
    stp     q18, q19, [x4, #(9 * 32)]
    stp     q20, q21, [x4, #(10 * 32)]
    stp     q22, q23, [x4, #(11 * 32)]
    stp     q24, q25, [x4, #(12 * 32)]
    stp     q26, q27, [x4, #(13 * 32)]
    stp     q28, q29, [x4, #(14 * 32)]
    stp     q30, q31, [x4, #(15 * 32)]
    mrs     x5, fpsr
    mrs     x6, fpcr
    str     x5, [x4, #(16 * 32)]
    str     x6, [x4, #(16 * 32 + 8)]
    mov     x0, sp
    bl      exception
    add     x4, sp, #FP_AT
    ldp     q0, q1, [x4, #(0 * 32)]
    ldp     q2, q3, [x4, #(1 * 32)]
    ldp     q4, q5, [x4, #(2 * 32)]
    ldp     q6, q7, [x4, #(3 * 32)]
    ldp     q8, q9, [x4, #(4 * 32)]
    ldp     q10, q11, [x4, #(5 * 32)]
    ldp     q12, q13, [x4, #(6 * 32)]
    ldp     q14, q15, [x4, #(7 * 32)]
    ldp     q16, q17, [x4, #(8 * 32)]
    ldp     q18, q19, [x4, #(9 * 32)]
    ldp     q20, q21, [x4, #(10 * 32)]
    ldp     q22, q23, [x4, #(11 * 32)]
    ldp     q24, q25, [x4, #(12 * 32)]
    ldp     q26, q27, [x4, #(13 * 32)]
    ldp     q28, q29, [x4, #(14 * 32)]
    ldp     q30, q31, [x4, #(15 * 32)]
    ldr     x5, [x4, #(16 * 32)]
    ldr     x6, [x4, #(16 * 32 + 8)]
    msr     fpsr, x5
    msr     fpcr, x6
    ldp     x30, x2, [sp, #(30 * 8)]
    ldp     x3, x5, [sp, #(32 * 8)]
    msr     elr_el1, x2
    msr     spsr_el1, x3
    msr     sp_el0, x5
    ldp     x0, x1, [sp, #(0 * 8)]
    ldp     x2, x3, [sp, #(2 * 8)]
    ldp     x4, x5, [sp, #(4 * 8)]
    ldp     x6, x7, [sp, #(6 * 8)]
    ldp     x8, x9, [sp, #(8 * 8)]
    ldp     x10, x11, [sp, #(10 * 8)]
    ldp     x12, x13, [sp, #(12 * 8)]
    ldp     x14, x15, [sp, #(14 * 8)]
    ldp     x16, x17, [sp, #(16 * 8)]
    ldp     x18, x19, [sp, #(18 * 8)]
    ldp     x20, x21, [sp, #(20 * 8)]
    ldp     x22, x23, [sp, #(22 * 8)]
    ldp     x24, x25, [sp, #(24 * 8)]
    ldp     x26, x27, [sp, #(26 * 8)]
    ldp     x28, x29, [sp, #(28 * 8)]
    add     sp, sp, #FRAME
    eret
.popsection
