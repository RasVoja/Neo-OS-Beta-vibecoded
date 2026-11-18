# Progress log

Neo OS Lite is coded in small steps. Each step must build and pass its boot
test before the next one starts. This file says where the work stopped, so the
next session can pick it up without asking.

## Done

### Step 1: Fortress boots on x86_64 (2026-11-13)

- `fortress/`: a `no_std` Rust kernel for `x86_64-unknown-none`.
- Boots through the PVH protocol, so `qemu-system-x86_64 -kernel` loads it
  directly with no boot loader. `boot.S` builds identity page tables for the
  first 1 GiB, enters long mode and calls `kmain`.
- Prints a banner, the firmware memory map, usable RAM and the kernel command
  line on COM1.
- `neo.test` on the command line makes it exit QEMU through `isa-debug-exit`
  (status 33 = pass), which the boot test and CI use.
- Run: `./scripts/run-x86_64.sh`. Test: `./scripts/test-boot-x86_64.sh`.

### Step 2: CPU exceptions (2026-11-13)

- `arch/x86_64/idt.rs` and `isr.S`: an IDT with entry stubs for all 32 CPU
  exceptions. Each stub saves every register and calls `exception_dispatch`.
- A breakpoint (`int3`) is reported and execution continues. Any other
  exception prints its name, the error code, all registers (and for a page
  fault the address and the kind of access) and panics.
- The panic handler prints the message and source location and exits QEMU
  with status 35.
- Boot tests: `neo.test` now also checks that a breakpoint is caught;
  `neo.test=pagefault` reads unmapped memory and checks the report.
- Known gap: no TSS or separate double-fault stack yet, so a kernel stack
  overflow still resets the machine instead of being reported. That comes
  with the kernel's own GDT in step 3 or 4.

### Step 3: physical memory (2026-11-13)

- `mem/frame.rs`: a bitmap frame allocator (one bit per 4 KiB frame), filled
  from the firmware memory map. The first 1 MiB, the kernel image and the
  loader's data (start info, memory map, command line) are kept out.
- Hands out the lowest free frame first; freeing a frame twice panics.
- `sync.rs`: a minimal spin lock that keeps the allocator's state behind a
  safe interface.
- Self-test: three frames are distinct, page aligned, outside the kernel and
  writable; a freed frame is handed out again; the free count returns to
  where it started.
- Limit for now: only RAM below 1 GiB is managed, because only that is
  mapped. Checked with `-m 3G`: it boots and reports 1022 MiB managed.

### Step 4: the kernel's own page tables, GDT and TSS (2026-11-13)

- `arch/x86_64/gdt.rs`: the kernel's GDT with a TSS; interrupt stack 1 is a
  separate 16 KiB stack used only by the double-fault handler.
- `arch/x86_64/paging.rs`: 4-level page tables built from allocator frames.
  Physical memory is identity mapped up to 4 GiB or the end of RAM,
  whichever is higher (2 MiB pages; 4 KiB pages around the kernel image).
  `translate()` walks the tables.
- The page below the boot stack is left unmapped, so a stack overflow hits
  the guard page and is reported as a double fault instead of resetting the
  machine or silently overwriting memory.
- Fixed a latent bug: the linker put `.got` after `__kernel_end`, so the frame
  allocator could hand out (and page-table code zero) the kernel's GOT. `.got`
  now lives inside `.data`.
- Boot tests: page-table self-test (guard unmapped, kernel and local APIC
  mapped) and `neo.test=stackoverflow`. Also checked by hand with `-m 6G`.
- Not yet: no NX or read-only kernel text; everything is mapped writable.

### Step 5: all RAM and a kernel heap (2026-11-13)

- The frame bitmap is now sized from the memory map and stored in RAM (16 KiB
  for 512 MiB, 256 KiB for 8 GiB) instead of a fixed 1 GiB array. Frames are
  handed out only below a limit: 1 GiB at boot, raised to everything the
  kernel's page tables map once they are active.
- `mem/heap.rs`: the kernel heap, a first-fit free list sorted by address
  that merges neighbouring blocks on release. It starts empty at
  `0xFFFF_9000_0000_0000` and grows by mapping fresh frames (64 KiB at a time,
  up to 256 MiB). `Box`, `Vec` and `String` now work in the kernel.
- Heap self-test: a 100 000-element `Vec`, 1000 boxes freed and refilled in a
  checkerboard, a page-aligned allocation, a `String`; at the end every byte
  must be free again in a single block.
- Checked by hand: `-m 6G` (6142 MiB free, frames above 4 GiB usable) and a
  debug build (overflow checks and debug assertions on).

### Step 6: timer and preemptive scheduler (2026-11-13)

- `arch/x86_64/apic.rs`: the legacy PICs are remapped to 0xF0-0xFF and
  masked; the local APIC timer is calibrated against PIT channel 2 and runs
  periodically at 100 Hz on vector 32.
- The IDT now has stubs for all 256 vectors; vectors from 32 up go to the
  device interrupt dispatcher (timer, spurious, unexpected).
- `sync.rs`: the spin lock now turns interrupts off while held, so a timer
  interrupt can never switch away from a thread holding the heap or frame
  lock and deadlock the single core.
- `sched.rs`: kernel threads with 16 KiB heap stacks, a round-robin scheduler
  with a 2-tick (20 ms) time slice, `spawn`, `yield_now` and `exit`. The
  `kmain` context becomes thread 0. Ended threads' stacks are freed on the
  next switch. The context switch saves only callee-saved registers; a
  preempted thread's interrupt frame stays on its own stack.
- Self-test: three busy threads that never yield all make progress for
  0.3 s (preemption works), then stop and are cleaned up. Repeated 5 times
  in release and 3 times in a debug build.

### Step 7: sleeping, parking and an idle thread (2026-11-13)

- `sched::sleep(ticks)`: the thread is not scheduled again until the timer
  reaches that tick. The timer wakes sleepers and leaves idle at once.
- `sched::park()` / `unpark(id)`: block until another thread wakes you. An
  unpark that arrives early is remembered, so a wake-up is never lost. This
  is the building block for wait queues and message ports.
- An idle thread halts the CPU (`sti; hlt`) and runs only when nothing else
  is ready, so a waiting system no longer spins.
- Self-test: a 10-tick sleep takes 10 ticks and lets idle run; yielding with
  nothing else ready does not switch; a parked thread is woken once by an
  unpark, and a second unpark sent before it parks again is not lost.

### Step 8: message ports, the seed of Exec-N (2026-11-13)

- `ipc.rs`: a `Port` is a bounded message queue. `send` waits while it is
  full, `recv` waits while it is empty; waiting threads park and the other
  side unparks them. `call` sends a request with a fresh reply port and waits
  for the answer.
- Ports can be published under a name (`publish`, `find`, `withdraw`), so a
  client finds a service by name, Amiga style.
- A `Message` is a tag, three words of payload and an optional reply port.
- Self-test: 1000 messages through a 4-slot port arrive complete and in
  order (both sides block in turn); a small "adder" service publishes its
  port, answers 20 calls, then quits and withdraws its name.
- Kernel threads only for now; real Exec-N passes messages between address
  spaces over shared-memory rings.

### Step 9: programs in user mode and system calls (2026-11-13)

- GDT gains user code and data segments (ring 3) in the order `syscall` and
  `sysret` need; the TSS `rsp0` and the syscall entry stack are set to the
  running thread's kernel stack on every switch.
- `paging.rs`: an address space per program. It shares the kernel mappings
  (supervisor-only, so the program cannot touch them) and has its own user
  part in PML4 slots 1-255; `destroy_user` frees all of it.
- `process.rs`: loads a raw machine-code image read-only at
  `0x80_0000_0000`, a 16 KiB stack below `0x80_1000_0000`, and drops to ring 3.
  The thread switch also switches the address space.
- `syscall.rs`: `exit`, `print`, `find_port`, `send`, `sleep` through the
  `syscall` instruction. Every pointer is checked against the program's own
  mapped memory; caller-saved registers come back zeroed.
- A fault in user mode stops only that program, with a report; the kernel
  carries on.
- `programs/`: four tiny test programs in assembly. The self-test checks
  that `hello` prints, finds a port by name and sends a message; `badptr`
  gets an error instead of reading kernel memory through `print`; `peek`
  (reads kernel memory) and `halt` (privileged instruction) are stopped;
  every thread, page table and frame is given back afterwards.
- Not yet: programs are raw images (no ELF loader), no NX bit, one thread
  per program, ports are found by name with no access rights yet.

### Step 10: capabilities and a first service in user mode (2026-11-13)

- A program's handle table now holds capabilities: a port plus rights
  (send, receive). A program starts with only the handles its creator gives
  it; it gains more by receiving a reply port in a message (send right only)
  or by looking a name up, which needs the `lookup` permission.
- New system calls: `recv` (blocks; fills tag, two words, sender and reply
  handle into the program's buffer, which must be its own writable memory),
  `call` (send and wait for the answer) and `close`. `send` now takes a tag.
  Errors: `ERR_DENIED` for a missing right, `ERR_BAD_HANDLE` for no handle.
- Test programs: `doubler`, a service in user mode that answers requests
  on the port it was given; `client`, which holds only the send right to it.
  The client checks that receiving, looking up names and using a handle it
  does not have are all refused, then calls the service and gets 42. The
  kernel calls the same service too, then tells it to quit.

### Step 11: programs written in Rust, loaded from ELF (2026-11-13)

- `user/`: a separate Cargo workspace for programs. `neo-rt` is their
  runtime: `_start`, safe wrappers for every system call (`Result` with an
  `Error` enum), `print!`/`println!` and a panic handler. Programs are
  compiled position-independent and linked at `0x80_0000_0000` with
  `user.ld` (one page-aligned segment per kind of access).
- `user/hello-rust`: the first Rust program. It prints, computes fib(50),
  checks that receiving, an unknown handle and name lookup are all refused,
  sends its result on the handle it was given and exits with 7.
- `fortress/build.rs` builds the `user/` workspace on every kernel build and
  the kernel embeds the ELF files (`include_bytes!`).
- `elf.rs`: a bounds-checked ELF64 reader. `process::spawn_elf` maps each
  loadable segment with its own rights (read-only code, writable data),
  zero-fills the rest, and refuses segments outside the program's part of
  memory, overlapping or unaligned segments and an entry point outside code.
- Self-test: hello-rust runs and reports; four damaged files (bad magic,
  wrong machine, a segment aimed at the kernel, truncated headers) are
  refused; no frame is leaked.

How to write a program: copy `user/hello-rust`, add it to `user/Cargo.toml`
and to the list in `fortress/build.rs`, then start it with
`process::spawn_elf`.

### Step 12: the console as a service in user mode (2026-11-14)

- A second kind of capability: `Cap::Io { base, len }`, a range of x86 I/O
  ports. System calls 8 `io_out` and 9 `io_in` need such a handle and an
  offset inside its range; a port handle gives `Denied`, a missing one
  `BadHandle`.
- `user/console`: the console service. It is given a receive right on its
  request port (handle 0) and the eight UART registers of COM1 (handle 1),
  and is the only program that touches the serial port. It waits for the
  transmitter to be free (LSR bit 5) and writes each byte (THR), turning
  `\n` into `\r\n`. It checks that offset 8 is refused.
- Protocol (in `neo-rt`): tag `1 | len << 8` carries up to 16 bytes packed
  little-endian in a0/a1; tag 2 stops the service, which exits with the
  number of characters printed. `neo_rt::console_write` and
  `ConsoleClient` (a `fmt::Write`) split text into such messages.
- `user/hello-console`: prints two lines through the service with only a
  send right, and checks it cannot use I/O itself.
- Self-test: both programs run, the text appears, the service stops on
  request, all threads end and no frame is leaked.

The kernel's own `print` call stays for now (early programs and panics);
new programs should print through the console service.

### Step 13: Fortress boots on AArch64 (2026-11-14)

- `arch/aarch64`: entry for the QEMU `virt` machine. Core 0 boots and the
  others wait. If started in EL2 it drops to EL1. It turns on FP/SIMD, zeroes
  `.bss`, and switches the MMU on with a boot identity map of the first
  4 GiB in 1 GiB blocks (the first as device memory, the rest as cached RAM).
  The kernel is linked at 0x4020_0000.
- `vectors.S`: all 16 exception vectors save the registers and call Rust. A
  `brk` is counted and skipped. Anything else is reported (kind, cause, ESR,
  FAR, ELR, SPSR, x0-x30) and stops the kernel.
- `fdt.rs`: a small device tree reader with no heap. It finds RAM, the
  reserved areas and the command line (`/chosen/bootargs`). The tree comes
  in x0, or from the start of RAM where QEMU puts it for ELF kernels.
- The serial console is the PL011 UART. QEMU is left through semihosting,
  with the same exit statuses as on x86_64 (33 pass, 35 fail).
- The frame allocator is shared with x86_64. `main.rs` now holds only what
  both architectures share; the x86_64 boot path moved to
  `start_x86_64.rs`, and the new one is `start_aarch64.rs`.
- `scripts/test-boot-aarch64.sh` boots three times: the normal self-test,
  an undefined instruction that must be reported, and a 2-core, 1 GiB
  machine started in EL2. CI runs it next to the x86_64 test.

Build for AArch64 with `cargo build --release --target aarch64-unknown-none`.

### Step 14: AArch64 gets its own page tables and a heap (2026-11-14)

- `arch/aarch64/paging.rs`: 4 KiB pages, four levels, 48-bit addresses.
  As on x86_64, memory is identity mapped up to 4 GiB or the end of RAM,
  with 2 MiB blocks except around the kernel, where 4 KiB pages leave the
  page under the boot stack unmapped as a guard. RAM regions from the
  device tree are normal cached memory; everything else is device memory
  that can never be executed.
- One top-level table serves both halves (TTBR0 and TTBR1), the way x86_64
  uses one PML4, so the kernel heap at `0xFFFF_9000_0000_0000` works on
  AArch64 without changes.
- Kernel stack overflow: every exception taken in the kernel first checks
  (with `AT S1E1W`) that its register frame fits on the current stack. If
  not, it switches to a small emergency stack, so the overflow is still
  reported instead of looping.
- Data aborts are reported as page faults (address, read or write, missing
  or protected page, kernel or user mode).
- The heap test and the stack overflow helper are shared by both
  architectures now (`main.rs`).
- Boot test: page tables and heap self-tests, plus a page fault and a stack
  overflow boot, as on x86_64.

### Step 15: timer, interrupts and threads on AArch64 (2026-11-14)

- `arch/aarch64/gic.rs`: the GICv2 interrupt controller of QEMU `virt` and
  the EL1 virtual timer (interrupt 27) at 100 Hz. When started in EL2, the
  entry code lets EL1 use the timer and zeroes the virtual offset.
- Every exception now also saves the FP/SIMD registers (q0-q31, FPSR,
  FPCR), because the kernel itself uses them; `switch_context` keeps x19-x30
  and d8-d15, the registers a function call must preserve.
- The scheduler, sleeping, parking and message ports from x86_64 run
  unchanged on AArch64. `sched.rs` only leaves out user programs there.
  The thread, sleep and port self-tests moved to `thread_tests.rs`, shared
  by both architectures.
- New self-test on both: two threads do floating-point work while the timer
  preempts them, and their results must match the same work done without
  interruption. Without the FP save in the exception entry it fails.
- The AArch64 boot test checks the timer, scheduler, sleep, ports and FP
  tests, also on the 2-core machine started in EL2.

### Step 16: programs in user mode on AArch64 (2026-11-14)

- Programs, capabilities, the ELF loader and system calls (`process.rs`,
  `syscall.rs`, `elf.rs`) are now shared by both architectures. Only the
  I/O-port calls 8 and 9 stay x86_64-only.
- AArch64 address spaces: each program has its own top-level table in
  TTBR0 that shares the kernel's entries (EL1 only) and adds slots 1..255,
  the same program addresses as on x86_64. Writable program pages are
  never executable, and the kernel can execute none of them. Switching
  threads loads TTBR0 and drops stale translations.
- Entering EL0 clears every general and FP/SIMD register. `svc #0` with the
  call number in x8 and arguments in x0-x3 reaches the same
  `syscall_dispatch`. Any other exception from EL0 stops only that program.
  The exception frame now keeps the program's stack pointer (SP_EL0).
- `neo-rt` has an AArch64 `_start` and system call stub; `fortress/build.rs`
  builds the user programs for the kernel's own processor.
- `user/crash`: a program that reads kernel memory. On both architectures
  it is stopped, the kernel carries on and no frame leaks.
- `hello-rust` and its checks run unchanged on AArch64. The Rust program
  tests moved to `program_tests.rs`, shared by both.

### Step 17: a boot image with the programs (2026-11-14)

- `bootfs.rs`: a read-only file system in memory, the boot image the
  loader puts next to the kernel (QEMU `-initrd`). The format is a plain
  ustar archive. Every header checksum and size is checked, so a damaged
  image is refused instead of being read past its end.
- x86_64 finds the image as the first PVH module. AArch64 finds it in the
  device tree (`linux,initrd-start` / `-end`). Both keep its memory away
  from the frame allocator.
- AArch64 kernels now start with the 64-byte header of the arm64 Linux
  boot protocol, and `scripts/elf2image.py` turns the ELF file into the raw
  Image. QEMU passes the device tree and the boot image only to such
  images, and U-Boot and the Raspberry Pi firmware use the same protocol.
  The ELF file still boots.
- `scripts/make-bootfs.sh x86_64|aarch64` builds the user programs and
  packs them under `bin/`. The boot tests pass the image. The self-test
  lists it, runs `bin/hello-rust` from it, and checks that two damaged
  copies are refused.
- AArch64 user programs are linked with 4 KiB segment alignment, which
  makes them about 5 times smaller.

### Step 18: programs read files and start programs (2026-11-14)

- Two new kinds of capability: `BootDir` (may list the boot image and open
  its files) and `File` (one read-only file: read it, or start it as a
  program).
- System calls 10-15: `open`, `read`, `list`, `spawn`, `wait`,
  `file_size`. A program's file name in the image becomes its thread name.
  `spawn` passes the new program a copy of one handle as its handle 0.
  `wait` works only on programs the caller started, and returns -255 for
  one the kernel had to stop. Every buffer is checked against the
  program's own memory.
- `neo-rt` wraps all of them.
- `user/runner`, started from the boot image with only a `BootDir` handle:
  it lists the image, reads README in two pieces, starts `bin/crash` and
  waits for it to be stopped, and checks that a missing file, a file used
  as a directory, a directory read as a file, waiting for someone else's
  program and a kernel buffer are all refused. The self-test checks its
  exit code on both architectures.

### Step 19: Neo DOS, the first command line (2026-11-14)

- Keyboard input over the serial console on both architectures
  (`arch::getc`). A new `Console` capability, and system call 16
  `read_input`, which waits until something is typed. There is no serial
  interrupt yet, so it looks every tick.
- `user/neodos`: Neo DOS 0.1, in the spirit of AmigaDOS. It has `DIR`,
  `TYPE`, `RUN` (a program's name alone works too, with or without `bin/`),
  `ECHO`, `VERSION`, `HELP` and `ENDCLI`. It echoes typed text and handles
  backspace. A started program gets the boot image as its handle 0. When
  the system stops a misbehaving program, Neo DOS says so and carries on.
- `neo.shell` on the kernel command line starts Neo DOS from the boot image
  with only the image and the console as handles. QEMU ends with status 33
  when it leaves with `ENDCLI`.
- Both boot tests type a short session (help, dir, type, a crashing
  program, an unknown command, echo, endcli) and check the answers.

Try it: `qemu-system-x86_64 -machine q35 -m 512M -nographic -kernel
target/x86_64-unknown-none/release/fortress -initrd "$(scripts/make-bootfs.sh
x86_64)" -append neo.shell`.

### Step 20: a disk (virtio block device) (2026-11-14)

- `virtio_blk.rs`: a legacy virtio block driver. It handles PCI on x86_64
  (`virtio-blk-pci,disable-modern=on`, registers on I/O ports, found by
  scanning PCI bus 0) and memory-mapped registers on AArch64 (the `virt`
  machine's 32 virtio-mmio slots). Only register access differs between
  the two; the request queue code is shared.
- One queue, one request at a time, waiting by polling. Reads of up to 8
  sectors are checked against the disk's size and the device's status.
  The driver is in the kernel as a bootstrap and will move to a driver
  process in user mode later.
- The frame allocator can now hand out physically contiguous frames,
  which the queue needs.
- PCI configuration access and 16- and 32-bit port I/O on x86_64.
- `diskfs.rs`: the same ustar file system as the boot image, read from the
  disk sector by sector. Listing reads only the headers, and a file's bytes
  are read when it is opened.
- Self-test, with the boot image copied onto the disk: list the disk,
  check that README is the same on disk and in the boot image, refuse a
  missing file and a read past the end, and run `bin/hello-rust` read from
  the disk. No frame may leak.

### Step 21: Neo DOS volumes, BOOT: and DH0: (2026-11-14)

- A directory handle now names its volume: the boot image (`BOOT:`) or the
  disk (`DH0:`). `open`, `read`, `list` and `spawn` work the same on both;
  a file read from the disk is held in memory while a handle refers to it.
- The disk now has its own contents (README "Neo OS Lite disk DH0:" and the
  programs), built by `make-bootfs.sh ARCH disk`.
- Neo DOS 0.2 gets the disk as handle 2 when there is one. New commands:
  `CD vol:` (or just `vol:`), `INFO` (volumes and sizes), and `DIR vol:`.
  The prompt shows the current volume, `BOOT:>` or `DH0:>`.
- The boot tests on both architectures switch to `DH0:`, list it, show its
  README and run a crashing program from the disk; Neo DOS carries on.

### Step 22: typed input by interrupt (2026-11-14)

- The serial port now interrupts when a byte arrives: on x86_64 through
  the I/O APIC (ISA IRQ 4, with OUT2 set on the UART), on AArch64 as
  shared interrupt 33 of the GIC (PL011 receive interrupts).
- `input.rs`, shared by both: the handler moves the bytes into a 256-byte
  buffer and wakes the reading thread. A reader with nothing to read parks,
  so a waiting Neo DOS no longer wakes up every tick. A byte that arrives
  between the check and the park leaves an unpark behind and is not lost.
- On AArch64 the UART's interrupt is cleared before the bytes are read, not
  after: clearing it afterwards lost a byte that came in meanwhile, and
  input stopped after the first key.
- When Neo DOS ends, the kernel reports how many input interrupts it took;
  the boot tests on both architectures require at least one.

### Step 23: a keyboard driver in user mode (2026-11-14)

- Two new kinds of capability. An interrupt handle (`Cap::Irq`) lets a
  driver wait on one interrupt line with the new `irq_wait` call; the
  kernel's handler only counts the interrupt and wakes the driver. A
  typing handle (`Cap::Typing`) lets a driver add bytes to what the
  console reads, with the new `type_byte` call. The kernel can take a line
  back (`irq::revoke`), and the driver's wait then fails so it can end.
- `bin/keyboard` (x86_64): the PS/2 keyboard driver. It gets the
  controller's five ports, IRQ 1 and a typing handle, turns on the
  keyboard interrupt and scan code translation, and turns set 1 scan codes
  into characters (US layout, Shift and Caps Lock).
- The I/O APIC now routes ISA interrupts generally (line n to vector
  32 + n); IRQ 1 goes to the driver.
- With `neo.shell` the kernel starts the keyboard service next to Neo DOS,
  so typing in the QEMU window works as well as typing on the serial line.
- Self-test: the PS/2 controller's command 0xD2 presses keys with no one at
  the keyboard (Shift+H, I, space, 4, 2, Enter); Neo DOS's input must read
  `Hi 42` and a newline, the driver must end with 6 when its line is taken
  back, and no frame may leak.
- A fifth boot test types `echo Typed on PS/2!` and `endcli` with QEMU's
  monitor (`scripts/sendkeys.py`), through the real emulated keyboard.

### Step 24: a keyboard on AArch64 too (virtio-input) (2026-11-14)

- `virtio_input.rs`: the virtio keyboard of the `virt` machine
  (`-device virtio-keyboard-device`), legacy MMIO, by interrupt. The driver
  offers 64 event buffers; the device writes 8-byte input events into them
  and raises its interrupt (shared interrupt 16 + slot, INTID 48 + slot).
  The handler turns key presses into characters for the console input and
  offers the buffers again.
- `keymap.rs`: Linux key codes and PS/2 set 1 scan codes agree for the
  usual keys, so one US table with Shift and Caps Lock serves both.
- Like the disk driver, this one lives in the kernel as a bootstrap; on
  x86_64 the keyboard driver already runs in user mode. It moves out once
  a driver can be given device memory and DMA buffers as capabilities.
- A seventh AArch64 boot types `echo Typed on virtio!` and `endcli` with
  QEMU's monitor, and Neo DOS runs them.

### Step 25: the first window, drawn from user mode (2026-11-14)

- A new capability for device memory (`Cap::Mem`) and the `map_device`
  call (19): a driver maps a device's memory into its own address space,
  in a 1 GiB window above its code. Such pages are marked as device memory
  in the page tables (a free bit on both architectures), so they never go
  back to the frame allocator when the program ends.
- Port I/O calls now take a width (1, 2 or 4 bytes); the whole access must
  be inside the program's range.
- `display.rs` (x86_64) finds the PC's standard VGA card under QEMU (the
  Bochs display, PCI 1234:1111) and its framebuffer.
- `bin/display`: the display service. It gets the card's mode registers,
  its framebuffer and the clock chip's ports; it sets 800 x 600 at 32 bits
  and draws the desk: wallpaper, the Bar, a first window (in Serbian and
  English) and the Clock tile with the time from the clock chip. Text uses
  an 8x16 font made from DejaVu Sans Mono by `scripts/make-font.py`, with
  the Serbian Latin letters.
- `neo.shell` draws the desk before Neo DOS starts.
- Self-test: the service draws, ends with 0, and the kernel finds the right
  colours at five places in the framebuffer; no frame leaks.

### Step 26: the clock ticks (a display service that keeps running) (2026-11-14)

- `recv` (call 5) can now poll: with the flag set it returns at once with
  "not found" when the port is empty. `neo-rt` has `try_recv`.
- `bin/display` is now a service that keeps running. It draws the desk once,
  then reads the clock chip a few times a second and redraws only the Clock
  tile's face, the digits and the time on the Bar when the second changes.
  A message on its control port (tag 2) stops it; it ends with the number
  of redraws.
- `neo.shell` starts the display service beside Neo DOS instead of waiting
  for it, and stops it when Neo DOS ends ("Display service stopped after N
  clock redraws.").
- Self-test: the kernel lets the service run for 2.5 s, checks the pixels
  while it is still running, stops it and expects at least two redraws and
  no frame leaks.

### Step 27: the screen on AArch64 too (ramfb) (2026-11-14)

- `display.rs` is now shared by both architectures. On AArch64 the kernel
  finds QEMU's firmware interface (fw_cfg), looks up `etc/ramfb`, takes
  469 contiguous pages of RAM for an 800 x 600 framebuffer and hands its
  address over with one DMA write. It does this once at boot ("Screen:
  ramfb 800x600 at ...").
- The same `bin/display` program runs on both. Its handles are now: 0 the
  framebuffer, 1 its control port, 2 the clock (CMOS ports on a PC, the
  PL031 chip's page on AArch64), and on a PC 3 the card's mode registers.
- `neo.shell` starts the display service on AArch64 as well; the boot image
  there has 6 files.
- Self-test on AArch64 (with `-device ramfb`): the same five pixels, at
  least two clock redraws, no frame leaks. Without a screen it is skipped.
- Screenshot: `docs/img/first-window-aarch64.png`.

### Step 28: the mouse and the pointer (2026-11-14)

- `pointer.rs`: the kernel keeps the pointer's position (inside the screen)
  and buttons, and tells the display service of each change on its control
  port (tag 3), without waiting: if the service is behind, a move is
  dropped and the next one carries the new position.
- A new pointer capability (`Cap::Pointer`) and call 20 (`pointer_move`).
- Call 17 (`irq_wait`) can wait on two interrupt lines at once
  (`irq_wait_either`). The PC's PS/2 controller driver (bin/keyboard) uses
  it to run the mouse too, on IRQ 12: it turns 3-byte packets into moves.
- AArch64: virtio-input now drives up to four devices; mice (relative) and
  tablets (absolute) move the pointer, keyboards type as before.
- The display service draws an arrow at the pointer, keeping what it covers,
  and lifts it while the clock is redrawn. It now looks for work every
  20 ms.
- Self-test on x86_64: three mouse packets injected through the controller
  (command 0xD3) put the pointer at (440, 330) with the left button down and
  the arrow drawn there. Shell boots on both architectures move the mouse
  through QEMU's monitor and check where the pointer ends up.
- Screenshot: `docs/img/pointer.png`.

### Step 29: windows that move (2026-11-14)

- The display service keeps its desk as a small scene (wallpaper, the
  Clock tile, the first window, the Bar on top) and repaints only a
  rectangle of it, back to front, with clipping: when the window moves,
  the area it left and the area it covers; when the clock ticks, the tile
  and the Bar's time (under the window, if the window is over them).
- Press the left button on the window's title bar and move: the window
  follows the pointer. Its title bar stays on the desk, above the Bar.
- Self-test on x86_64: injected mouse packets grab the title bar and drag
  the window from (90, 100) to (190, 250); the kernel finds the title bar
  at the new place and the wallpaper at the old one.
- Screenshot: `docs/img/window-dragged.png`.

### Step 30: a contributor guide (2026-11-14)

- `CONTRIBUTING.md` (English) and `CONTRIBUTING.sr.md` (Serbian): what is
  needed, building and testing, the code's layout, the rules a change must
  keep, and how to add a program, a service and a system call, with first
  tasks.
- `rustfmt.toml` (lines up to 120 characters): all code formatted with it,
  and CI checks the formatting.
- The system call table at the top of `syscall.rs` now lists calls 17-20
  and the newer arguments.

### Step 31: windows for programs, through shared memory (2026-11-14)

- Shared memory: call 21 (`share`) makes a block of zeroed, contiguous
  pages (`Cap::Shared`), which `map_device` maps as ordinary memory. The
  block is freed when the last handle and the last mapping are gone; a
  mapping keeps it alive even if the handle is closed. `file_size` on such
  a handle gives its size.
- Handles in messages: call 22 (`send_handle`) sends a copy of one of the
  sender's handles with a message; `recv` now fills six words, the sixth
  being the handle that came along (or none).
- The display service publishes its port as "display". A program sends
  `WINDOW_TITLE` (8 bytes of title), then `WINDOW_OPEN` with its pixels'
  shared memory and size, and `WINDOW_DRAWN` after drawing. The service
  checks the size against the block, maps it, frames it with a title bar
  and copies it to the screen inside the clip. Up to three program
  windows, each dragged like the first, topmost first.
- Programs started by Neo DOS may look ports up when Neo DOS may, so
  `hello-window` (new) runs from the command line.
- Self-test on both architectures: hello-window opens its window; the
  kernel finds its pixels and title bar on the screen, then stops the
  service and checks that no frame leaked (the shared block included).
- Screenshot: `docs/img/program-window.png`.

### Step 32: clicks reach windows; a click raises a window (2026-11-14)

- Programs can make message ports (call 23, `new_port`) and send without
  waiting (call 24, `try_send`). A port passed in a message keeps only its
  send right, so the receiver can send to it but not read it.
- A window's program may pass an event port (`WINDOW_EVENTS`) before
  `WINDOW_OPEN`. Clicks inside the window's pixels reach it as
  `EVENT_CLICK` (position inside the window and the buttons just
  pressed); the display service never waits on a slow program. When the
  service stops, every such program gets `EVENT_QUIT`.
- Pressing any button on a window raises it in front of the others; the
  title bar still drags it. `WINDOW_CLOSE` takes a program's window away.
- hello-window answers clicks: left turns its disc amber (or white again),
  right closes the window; it also ends on `EVENT_QUIT` or after 30 s.
- Self-test on x86_64: injected mouse moves and clicks turn the disc and
  then close the window; the program ends with 1 change.
- Screenshot: `docs/img/window-raised.png`.

### Step 33: keys go to the window in focus (2026-11-14)

- A key focus capability (`Cap::KeyFocus`) and call 25 (`focus_keys`):
  the holder says which port keyboard bytes go to. While a window has the
  focus, what the PS/2 or virtio keyboard types reaches its program as
  `EVENT_KEY` (never waiting on it) instead of Neo DOS; the serial console
  always types into Neo DOS.
- The display service holds it (handle 3; the PC's mode registers moved to
  handle 4). A new program window gets the focus; a click gives it to the
  window clicked, or back to the console when the click is on the desk or
  the first window; closing the window or stopping the service gives it
  back too.
- hello-window: A turns the disc amber or white, Q closes the window.
- Self-test on x86_64: after a click, A and Q typed on the PS/2 keyboard
  reach the window and not the console; after the window closes, X reaches
  the console again.

### Step 34: neo-gfx, drawing and text for every program (2026-11-14)

- `user/neo-gfx` (new library): `Canvas` over 32-bit pixels with a clip
  rectangle: rectangles, outlines, discs, rings, lines and text in the Neo
  8x16 font (which moved here from the display service; `make-font.py`
  writes it here now). `Window` opens a window through the display service
  in one call and turns its messages into events (`Click`, `Key`, `Quit`).
- The display service draws the desk with the same canvas.
- hello-window uses it: a greeting at the top, typed keys appear on a line
  (Backspace removes one), Esc or a right click closes the window, a left
  click turns the disc. It ends with ten times the colour changes plus the
  line's length.
- Self-test on x86_64: "hi" typed on the PS/2 keyboard shows in the window,
  Esc closes it, the program ends with 12.
- Screenshot: `docs/img/typing-in-window.png`.

### Step 35: Neo DOS in a window (the terminal) (2026-11-14)

- The kernel can copy what programs print on the console to a port
  (`console::set_tee`), 16 bytes per message, never waiting on it.
- `bin/terminal` (new): opens an 80 x 25 window through the display
  service, draws the console text as it comes (with scrolling, backspace
  and an amber cursor), and passes keys typed in its window on to the
  console (call 18 with a third argument of 1: straight into the console,
  not back to the window in focus).
- `neo.shell` starts the terminal beside the display service, so Neo DOS
  runs in a window on the desk as well as on the serial console. The
  display service now allows windows up to 640 x 400 and moves a large one
  left and up until it fits above the Bar.
- Self-test on both architectures: text printed shows in the window, a key
  typed there reaches the console, nothing leaks.
- Fixed: the x86_64 self-test's final result had stopped including the
  click test (a formatting change had hidden it); it includes every test
  again.
- Screenshot: `docs/img/terminal.png`.

### Step 36: the Serbian Latin keyboard layout (2026-11-14)

- `user/neo-keys` (new): one keyboard translation shared by the kernel's
  virtio keyboard driver and the PS/2 keyboard service, in place of
  `fortress/src/keymap.rs` and the service's own tables. It knows the US
  layout and Serbian Latin (QWERTZ: z and y swap places; č, ć, ž, š, đ sit
  right of L and P, as on a Serbian keyboard). Left Alt+Shift switches
  between them.
- Typed letters travel as UTF-8 all the way: the drivers type each
  character's bytes, the console lets bytes from 0x80 up through, Neo DOS
  reads and echoes whole characters (Backspace removes a whole letter),
  and the terminal keeps a character per cell. The window font already
  had the Serbian letters.
- Programs get a 64 KiB stack (was 16 KiB): the terminal's screen of
  characters no longer fits in 16.
- Tests: the PS/2 self-test now also switches layout and types č and z;
  on both architectures a Neo DOS session typed through QEMU's monitor
  switches layout with Alt+Shift and echoes "čšđ".
- Screenshot: `docs/img/serbian-keys.png`.

### Step 37: arrows, Home, End, Delete and line editing (2026-11-14)

- `neo-keys` turns the arrows, Home, End and Delete into the escape
  sequences a VT100 terminal sends (`ESC [ D` for Left, `ESC [ 3 ~` for
  Delete), so a program reads them the same way from either keyboard
  driver and from the serial line. The PS/2 service understands the 0xE0
  prefix those keys come with.
- Neo DOS edits the line: Left and Right move the cursor, Home and End
  jump to the ends, Backspace and Delete remove the letter before or under
  the cursor, and typing in the middle inserts. Serbian letters count as
  one letter each.
- Tests on both architectures: a serial session uses the escape sequences
  (Home, Left, Backspace, End, Delete), and a session typed through QEMU's
  monitor presses the real keys on PS/2 or virtio.
- Screenshot: `docs/img/line-editing.png`.

### Step 38: command history in Neo DOS (2026-11-15)

- Neo DOS 0.3 keeps the last 16 lines typed (not empty ones, not the same
  line twice in a row). Up and Down step through them; Down past the
  newest brings back the line that was being typed.
- `HISTORY` lists them, numbered; `HELP` mentions the editing keys.
- Tests on both architectures: Up twice runs an earlier line again,
  `HISTORY` shows lines 14 and 15 as expected, and Down restores the
  unfinished line.

### Step 39: the keyboard layout on the Bar (2026-11-15)

- The kernel keeps the layout in use (`input::layout`), so both keyboard
  drivers and the Bar agree. Call 26, `keyboard_layout`, sets, switches
  or reads it; it needs the typing capability (a keyboard driver) or the
  key focus capability (the display service). A change is sent to the
  display service as `LAYOUT_CHANGED` (tag 9).
- The drivers read the layout before each key and report it when
  Alt+Shift switched it.
- The Bar shows a white "US" or a teal "SR" box left of the time; a click
  on it switches the layout and leaves the keyboard focus where it was.
- Self-test on both architectures (it runs where there is a screen): a
  driver's switch shows "SR", a pointer click on the box switches back to
  "US", nothing leaks.
- Screenshot: `docs/img/layout-on-bar.png`.

### Step 40: shared-memory rings for large messages (2026-11-15)

- `user/neo-ring` (new, used by programs and the kernel): a one-way byte
  ring laid over a shared memory block. A 64-byte header holds the bytes
  written and the bytes read, each moved by one side only (Release and
  Acquire ordering, no lock). A port message is the doorbell, so the
  reader sleeps in `recv` while the ring is empty.
- `bin/ring-send` (embedded for the self-test): makes a 64 KiB block,
  passes it to the kernel, and writes one megabyte of a test pattern in
  odd-sized pieces that wrap around the ring's end, waiting a tick when
  the ring is full.
- Self-test on both architectures: the kernel reads the ring from the
  other side and checks every byte, then checks that the block's frames
  come back.
- This closes the last open "programs and messages" target of M1.

### Step 41: decision record on Fortress and Genode (2026-11-15)

- `docs/decisions/0001-fortress-and-genode.md` and its Serbian twin:
  the options, the proposed choice (keep Fortress as the bootstrap, keep
  programs on `neo-rt` only, and switch when three named triggers hold),
  how each `neo-rt` call maps onto Genode, and what that means for the
  hours report. Status: proposed, for the project owner to accept.
- The licence is decided by the project owner: GPL version 3 or later for
  this repository and the public one. CONTRIBUTING says so in both
  languages.

### Step 42: twelve starter tasks (2026-11-15)

- `docs/STARTER-TASKS.md` and its Serbian twin: twelve first tasks (Neo
  DOS, keyboard and text, desk and windows, kernel and tests), each with
  its size and the test that proves it done. They become issues on the
  public repository when the code moves there.
- CONTRIBUTING's "Where to start" points to them.

## Step 43: the first Neo program on Genode/seL4

- neo-rt has a `genode` feature: every system call goes to one C function.
  `genode/repos/neo` implements exit and print with Genode services; the
  unchanged program in `genode/neo-hello` runs on Genode 24.11 + seL4
  under QEMU (`scripts/test-genode-x86_64.sh`, needs a Genode tree).
- Decision 0001 accepted, with the Genode trial started at once.
- The code moved to the public repository; the starter tasks are issues
  #3 to #14. M1 is closed.

## Step 44: a bootable PC image (GRUB, BIOS and UEFI)

- Fortress also carries a Multiboot2 header. The entry code passes the
  loader's magic to `kmain`, which turns the Multiboot2 information
  (command line, first module, memory map) into the PVH start info the
  rest of the kernel already reads (`pvh::BootInfo::from_multiboot2`).
- `scripts/make-usb-image.sh` builds `target/neo-lite-x86_64.iso`, a hybrid
  ISO from grub-mkrescue for CD or USB stick. Its menu starts the desk and
  Neo DOS, or the self-test.
- `scripts/test-usb-x86_64.sh` boots it with SeaBIOS and with OVMF (UEFI);
  the self-test must pass both times. CI runs it in a new `pc-image` job.
- Screenshot: `docs/img/uefi-desk.png`, the desk booted from the ISO under
  UEFI.

## Step 45: the screen on real PCs (UEFI GOP and VBE)

- Fortress's Multiboot2 header asks GRUB for 800 x 600 at 32 bits. GRUB
  sets it through UEFI GOP or VBE and reports the framebuffer (tag 8);
  `pvh::boot_framebuffer` keeps it and the frame allocator never hands
  its memory out (UEFI may place it in ordinary RAM).
- When that framebuffer fits the desk (800 x 600, 32 bits, 0x00RRGGBB, no
  padding), the display service gets it without the Bochs mode registers
  and keeps the mode it finds. Otherwise the Bochs card is used as before.
- `scripts/test-usb-x86_64.sh` now boots three times: BIOS, UEFI, and UEFI
  with no VGA card at all (only ramfb behind GOP), and checks that the
  desk is drawn on the loader's framebuffer each time.
- Screenshot: `docs/img/gop-desk.png`, the desk on a GOP-only screen.

## Step 46: ports and messages on Genode

- The Genode side of neo-rt is now a library, `neo_genode`, that every Neo
  component links. Besides exit and print it maps sleep to the Timer and
  the port calls (find_port, new_port, send, try_send, recv, close) to a
  new server, `neo_ports`, which holds the ports of all Neo components by
  name. Starting handles come from the component's config.
- Two new unchanged neo-rt programs, `neo-echo-server` and
  `neo-echo-client`, exchange 100 messages through two ports on Genode
  and seL4 under QEMU (`run/neo_echo.run`); all 100 answers are right.
- The Genode programs share one Cargo workspace (`genode/Cargo.toml`).

## Step 47: USB, part 1: the xHCI controller

- `fortress/src/xhci.rs` finds an xHCI controller on the PCI bus, resets
  it and gives it the device context table (with scratchpad pages when it
  asks for them), a command ring and an event ring. It starts it, checks
  the rings with a No-Op command, and reports which root ports have a
  device plugged in. It polls; MSI comes later.
- `arch::delay_us` for short device waits.
- A sixth x86_64 boot runs with `qemu-xhci`, a USB keyboard and a USB
  mouse, and checks that both devices are seen.

## Step 48: USB, part 2: setting devices up

- `fortress/src/usb.rs` sets up each device the controller sees: it resets
  the port (USB 2), takes a slot (Enable Slot), gives the device an
  address with a context for its control endpoint (Address Device), and
  reads its device and configuration descriptors through control
  transfers. A HID boot keyboard or mouse is configured and switched to
  the boot protocol; its interrupt endpoint is noted for reading reports.
- The boot log lists each port: vendor and product, speed, slot, and what
  the device is. The USB boot test now plugs in a keyboard, a mouse and a
  tablet (not a boot device) and checks all three.

## Step 49: USB, part 3: typing and pointing

- For every boot keyboard and mouse, `usb.rs` adds the interrupt IN
  endpoint to the device's slot (Configure Endpoint) and keeps eight
  transfers queued on it. A kernel thread, `usb`, polls the event ring a
  hundred times a second, hands each report on and queues its buffer
  again.
- Keyboard reports are compared with the previous one; modifier bits and
  keys that went down or up become key events through neo-keys (both
  layouts, Alt+Shift, arrows), into the same console input as PS/2 and
  virtio. Mouse reports move the pointer.
- A seventh x86_64 boot types a Neo DOS command on the USB keyboard and
  moves the pointer with the USB mouse, with no PS/2 input at all.

With steps 44 to 49, a PC that boots the ISO through UEFI gets the desk on
its screen and works with USB keyboard and mouse: what is left for a first
boot on real hardware is trying it on real machines and fixing what they
show.

## Step 50: every PCI bus, and registers above 4 GiB

- New `pci.rs` walks the PCI tree once at boot: every device and function
  on bus 0, then through each bridge to the buses behind it, as the
  firmware numbered them. The xHCI, virtio disk and Bochs display drivers
  ask it instead of looking at bus 0 only; on a real PC the USB
  controller usually sits behind a PCI Express root port.
- A device's memory BAR is sized and mapped. UEFI firmware puts 64-bit
  BARs high (OVMF at 768 GiB), above the identity map, so those registers
  are mapped uncached into a device window in the kernel half. Before
  this, the USB controller was not usable under UEFI.
- The sixth x86_64 boot puts the USB controller behind a root port (bus
  1); the PC image test has a USB controller in all three boots.

## Step 51: USB hubs

- A hub is set up like any device, then told to the controller as a hub
  (Configure Endpoint with the hub bit, its port count and, at high
  speed, the translator think time). Its ports are powered and, one at a
  time, reset and set up: each device behind it gets the route through
  the hubs in its slot context, and low- and full-speed devices behind a
  high-speed hub name the hub's transaction translator.
- Hubs nest up to the five tiers USB allows. The boot log names a port
  by its path, `5.3` for port 3 of the hub on root port 5. USB 3 hubs
  are listed but not walked yet; keyboards and mice show on their USB 2
  side.
- Full-speed devices may use 8 to 64 bytes per packet on endpoint 0: the
  first 8 bytes of the device descriptor are read and the controller is
  told the real size (Evaluate Context) before anything longer. QEMU does
  not mind; real keyboards do.
- An eighth x86_64 boot puts the USB keyboard and mouse behind a hub and
  types a Neo DOS command through it.

## Step 52: ACPI tables

- New `acpi.rs` finds the root pointer (from PVH, from GRUB's Multiboot2
  tags 14 and 15, or in the BIOS area) and reads the static tables.
- From the MADT: the number of processors, where the I/O APIC is, and
  the interrupt source overrides. ISA interrupts (keyboard, mouse,
  serial) are routed to the GSI the PC says, with its polarity and
  trigger mode, instead of assuming QEMU's wiring.
- From the FADT and the DSDT's `_S5_` object: the power management
  registers and sleep type. When Neo DOS ends on a real PC, Fortress
  switches the machine off (asking the firmware for ACPI mode first if
  needed) instead of halting.
- Boot 1 runs with two processors and checks the count; a ninth boot
  runs without QEMU's test exit device and expects QEMU to be switched
  off. The PC image test checks the tables under BIOS (ACPI 1.0) and UEFI
  (ACPI 2.0, XSDT).

With steps 50 to 52 the parts a real PC most often differs from QEMU in
are covered; next is trying the image on real machines.

## Next steps (planned, in order)

1. First boots on real PCs from the USB stick, and the fixes they show
   (a real-hardware test guide; USB 3 hubs; MSI interrupts for USB).
2. On Genode: wake waiters with a signal instead of polling, shared
   memory and handles in messages, then Neo DOS there.
