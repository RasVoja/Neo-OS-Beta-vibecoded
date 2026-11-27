/*
 * Tiny test programs that run in user mode (ring 3). They are plain,
 * position-independent machine code: the kernel copies each one into a
 * fresh address space. System call numbers are in src/syscall.rs.
 */
    .pushsection .rodata.programs, "a"

/* hello: prints a line, finds the port "neo.test.user", sends it 42 and 7,
   sleeps a little and exits with code 0. */
    .global prog_hello_start
    .global prog_hello_end
prog_hello_start:
    lea rdi, [rip + 1f]
    lea rsi, [rip + 2f]
    sub rsi, rdi                    /* length */
    mov eax, 1                      /* print */
    syscall
    lea rdi, [rip + 3f]
    lea rsi, [rip + 4f]
    sub rsi, rdi
    mov eax, 2                      /* find_port */
    syscall
    mov rdi, rax                    /* handle */
    mov rsi, 0x100                  /* tag */
    mov rdx, 42
    mov r10, 7
    mov eax, 3                      /* send */
    syscall
    mov rdi, 2
    mov eax, 4                      /* sleep */
    syscall
    xor edi, edi
    mov eax, 0                      /* exit */
    syscall
    ud2
1:  .ascii "Hello from ring 3!\n"
2:
3:  .ascii "neo.test.user"
4:
prog_hello_end:

/* peek: tries to read the kernel's memory; must be stopped. */
    .global prog_peek_start
    .global prog_peek_end
prog_peek_start:
    mov rax, 0x100000
    mov rax, [rax]
    xor edi, edi
    mov eax, 0
    syscall
prog_peek_end:

/* halt: tries a privileged instruction; must be stopped. */
    .global prog_halt_start
    .global prog_halt_end
prog_halt_start:
    cli
    hlt
prog_halt_end:

/* badptr: asks the kernel to print from kernel memory; the call must fail
   with an error, not crash or leak. Exits with the returned value. */
    .global prog_badptr_start
    .global prog_badptr_end
prog_badptr_start:
    mov rdi, 0x100000
    mov rsi, 64
    mov eax, 1                      /* print */
    syscall
    mov rdi, rax
    mov eax, 0                      /* exit */
    syscall
prog_badptr_end:

/* doubler: a service. Handle 0 is its port (receive right). Answers
   tag 1 with twice the number, ends on tag 2. */
    .global prog_doubler_start
    .global prog_doubler_end
prog_doubler_start:
    sub rsp, 48                     /* message buffer: 5 words */
doubler_loop:
    xor edi, edi
    mov rsi, rsp
    mov eax, 5                      /* recv */
    syscall
    test rax, rax
    jnz doubler_fail
    mov rax, [rsp]                  /* tag */
    cmp rax, 2
    je doubler_quit
    cmp rax, 1
    jne doubler_loop
    mov rbx, [rsp + 32]             /* reply handle */
    cmp rbx, -1
    je doubler_loop
    mov rdi, rbx
    mov esi, 1
    mov rdx, [rsp + 8]
    add rdx, rdx
    xor r10d, r10d
    mov eax, 3                      /* send the answer */
    syscall
    mov rdi, rbx
    mov eax, 7                      /* close the reply handle */
    syscall
    jmp doubler_loop
doubler_quit:
    xor edi, edi
    xor eax, eax                    /* exit 0 */
    syscall
doubler_fail:
    mov edi, 2
    xor eax, eax
    syscall
prog_doubler_end:

/* client: handle 0 is the doubler's port with the send right only, and it
   may not look names up. Checks that it is refused what it was not given,
   then asks the doubler and exits with the answer (42), or 100x on error. */
    .global prog_client_start
    .global prog_client_end
prog_client_start:
    sub rsp, 48
    xor edi, edi                    /* recv on a send-only handle */
    mov rsi, rsp
    mov eax, 5
    syscall
    mov edi, 1001
    cmp rax, -252                   /* ERR_DENIED */
    jne client_exit
    lea rdi, [rip + client_name]    /* find_port without permission */
    lea rsi, [rip + client_name_end]
    sub rsi, rdi
    mov eax, 2
    syscall
    mov edi, 1002
    cmp rax, -252
    jne client_exit
    mov edi, 5                      /* send on a handle it does not have */
    mov esi, 1
    mov edx, 1
    xor r10d, r10d
    mov eax, 3
    syscall
    mov edi, 1003
    cmp rax, -253                   /* ERR_BAD_HANDLE */
    jne client_exit
    xor edi, edi                    /* call the doubler with 21 */
    mov esi, 1
    mov edx, 21
    xor r10d, r10d
    mov eax, 6
    syscall
    mov rdi, rax
client_exit:
    xor eax, eax
    syscall
client_name:
    .ascii "neo.test.user"
client_name_end:
prog_client_end:

    .popsection
