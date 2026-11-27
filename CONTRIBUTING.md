# Contributing to Neo OS Lite

*Srpski: [CONTRIBUTING.sr.md](CONTRIBUTING.sr.md)*

Thank you for wanting to help. This guide covers building the system, testing it, how the code is laid out, and how to add a program or a service. It describes the code as it is today: **Fortress**, a small bootstrap kernel written in Rust. It boots on x86_64 (a PC) and on AArch64 (the QEMU `virt` machine) and runs its programs in user mode. See [docs/PROGRESS.md](docs/PROGRESS.md) for what each step added.

## What you need

- **Rust** through `rustup`. `rust-toolchain.toml` picks the stable channel and both bare-metal targets (`x86_64-unknown-none`, `aarch64-unknown-none`), and `rustup` installs them on the first build.
- **QEMU**: `qemu-system-x86` and `qemu-system-arm` (Debian and Ubuntu package names).
- **Python 3** for the small helper scripts. `scripts/make-font.py` also needs Pillow and the DejaVu Sans Mono font, but only if you change the font.
- A Unix shell. Linux is what CI uses. macOS should work.

## Build and test

```sh
./scripts/test-boot-x86_64.sh    # builds everything, boots 5 times, checks the output
./scripts/test-boot-aarch64.sh   # the same on AArch64, 7 boots
```

Each script builds the kernel and the user programs. It then boots QEMU with different kernel command lines: the self-test, deliberate crashes, and Neo DOS sessions typed through QEMU's monitor. It checks every line it expects and ends with `BOOT TEST PASSED`. CI (`.github/workflows/boot.yml`) runs both scripts on every push and pull request, and also checks formatting.

To use the system yourself:

```sh
./scripts/run-x86_64.sh neo.shell   # Neo DOS on the serial console; quit QEMU with Ctrl-A, X
```

Run Neo DOS with the boot image and a screen by passing the same QEMU options the test scripts use (`-initrd`, and on AArch64 `-device ramfb`). Add a QEMU window instead of `-display none` to see the desk and use the mouse.

Kernel command line words:

| Word | What it does |
| --- | --- |
| `neo.test` | Runs every self-test, then leaves QEMU with status 33 if all passed |
| `neo.shell` | Starts the display and keyboard services and Neo DOS |
| `neo.test=pagefault`, `=stackoverflow`, `=undefined` | Crashes on purpose, to check the crash report |

## How the code is laid out

| Path | What is there |
| --- | --- |
| `fortress/src/main.rs` | The shared start: module list, `run_shell`, the panic handler |
| `fortress/src/start_x86_64.rs`, `start_aarch64.rs` | Each architecture's boot path and self-test list |
| `fortress/src/arch/` | CPU, interrupts, timer and page tables per architecture |
| `fortress/src/mem/` | Frame allocator and kernel heap |
| `fortress/src/sched.rs`, `ipc.rs` | Threads, sleeping and parking; message ports |
| `fortress/src/process.rs`, `syscall.rs` | Programs, their capabilities (handles) and the system calls; the call table is at the top of `syscall.rs` |
| `fortress/src/irq.rs`, `input.rs`, `pointer.rs` | Interrupts handed to user-mode drivers, typed input, the mouse pointer |
| `fortress/src/display.rs`, `virtio_*.rs`, `bootfs.rs`, `diskfs.rs` | Devices and file systems |
| `fortress/src/*_tests.rs` | Self-tests shared by both architectures |
| `user/` | User programs (a separate Cargo workspace), with `neo-rt`, the runtime library, `neo-gfx`, drawing, text and windows, `neo-keys`, keyboard layouts, and `neo-ring`, rings in shared memory for large messages (the kernel uses these two too) |
| `scripts/` | Build, boot and test scripts |

The kernel's `build.rs` builds the `user/` workspace and embeds the programs the self-tests need. `scripts/make-bootfs.sh` packs the rest into the boot image (`-initrd`) and the disk.

## The rules a change must keep

1. **Both test scripts pass.** If a change alters what a boot prints, update the expected lines in the same commit.
2. **No warnings.** The builds are warning-free; keep them that way.
3. **Formatted with `cargo fmt`.** `rustfmt.toml` allows lines up to 120 characters. Run `cargo fmt` in the root and in `user/`.
4. **Nothing leaks.** Every self-test that starts a program checks that the frame count and thread count come back to where they were. Do the same in new tests.
5. **Drivers live in user mode when they can.** A driver gets exactly the capabilities it needs (ports, an interrupt line, device memory) and nothing more. Kernel drivers (`virtio_blk`, `virtio_input`) are a bootstrap until DMA buffers can be handed out as capabilities.
6. **Comments say why.** Each file starts with a `//!` paragraph on what it is for. Write in plain English that a newcomer can follow.
7. **One step per pull request,** with a short entry in `docs/PROGRESS.md`: what changed and how it is tested.

## Adding a user program

1. Copy `user/hello-rust` to `user/<name>` and change the package name in its `Cargo.toml`.
2. Add `<name>` to `members` in `user/Cargo.toml`.
3. Write `neo_main() -> i64`. For a window, use `neo_gfx::Window::open` (see `user/hello-window`). The value it returns is the exit code. `neo-rt` gives you `print`, ports, files, `sleep` and the rest. Handles are numbered from 0 in the order the kernel or the parent passed them.
4. To ship it, add it to the program list in `scripts/make-bootfs.sh`. That list is shared by both architectures, plus a list for x86_64 only. Then update the file counts the test scripts expect (`N files`).
5. Neo DOS can then start it with `RUN <name>`.

## Adding a service

A service is a program the kernel starts with capabilities. `display.rs` is the shortest complete example:

- **Capabilities.** Build its list (`Cap::Mem`, `Cap::io`, `Cap::Irq`, a port with `RIGHT_RECV` for its control port, and so on).
- **Starting it.** Start it with `process::spawn_elf`, and keep the port so the kernel can talk to it or stop it.
- **Self-test.** Write one: start the service, check what it does from outside (pixels, bytes, messages), stop it, and check that nothing leaked. Add it to the self-test list of each architecture it runs on.
- **Expected line.** Add the line the test prints to the test script.

## Adding a system call

1. Pick the next free number. Add it to the `match` in `syscall_dispatch` and to the table at the top of `syscall.rs`.
2. Check the handle's capability kind first. A wrong kind returns `ERR_DENIED`, and a missing handle returns `ERR_BAD_HANDLE`.
3. Add a wrapper to `neo-rt` with a doc comment.
4. Test it through a program, not only from the kernel.

## Where to start

The roadmap is in `docs/PROGRESS.md` under "Next steps". Twelve first tasks, each with its test, are in [docs/STARTER-TASKS.md](docs/STARTER-TASKS.md). A few of them:

- Tab completion of file names in Neo DOS, or a new layout in `user/neo-keys`.
- New Neo DOS commands in `user/neodos`, such as `TIME` or `MEM`.
- More characters in the font (`scripts/make-font.py`), for example Serbian Cyrillic.
- A second test for an error path that is not covered yet.

## Licence

This repository and the public one are both under the GNU GPL version 3 or later (see `LICENSE`). By contributing you agree that your contribution is published under that licence.
