#!/bin/sh
# Boot tests for Fortress on AArch64 (QEMU virt) under QEMU.
#   1. Normal boot: banner, RAM from the device tree, self-tests (breakpoint,
#      frame allocator, page tables, heap, timer and scheduler, sleeping,
#      message ports, FP registers across thread switches, Rust programs in
#      EL0, a faulty program stopped), exit 33 (pass).
#   2. Undefined instruction: must be reported with registers, then exit 35.
#   3. Page fault: reading unmapped memory must be reported, then exit 35.
#   4. Stack overflow: must hit the guard page and still be reported.
#   5. The same normal boot, started in EL2 on a 2-core machine with 1 GiB,
#      from the ELF file without a boot image (the others boot the raw
#      Image made by scripts/elf2image.py, the first with a boot image).
#   6. Neo DOS: a short session typed on the serial console.
set -e
cd "$(dirname "$0")/.."
input=""
cargo build --release --target aarch64-unknown-none
elf=target/aarch64-unknown-none/release/fortress
image=target/fortress-aarch64.img
python3 scripts/elf2image.py "$elf" "$image"
bootfs=$(./scripts/make-bootfs.sh aarch64)
# The disk (DH0:) holds the same programs with its own README.
disk=$(./scripts/make-bootfs.sh aarch64 disk)
withdisk="-drive file=$disk,format=raw,if=none,id=d0,readonly=on -device virtio-blk-device,drive=d0"
log=$(mktemp)
sock=$(mktemp -u)
trap 'rm -f "$log" "$sock"' EXIT

boot() { # $1 = kernel command line, $2 = extra QEMU options; sets $status
    set +e
    # shellcheck disable=SC2086
    # Typed input (for neo.shell) comes from $input, after a second.
    (sleep 1; printf "$input") | timeout 30 qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 512M $2 \
        -display none -serial stdio -no-reboot -semihosting \
        -append "$1" >"$log" 2>&1
    status=$?
    set -e
    echo "--- boot: $1 $2 (exit $status)"
    cat "$log"
}
fail() { echo "BOOT TEST FAILED: $1"; exit 1; }
expect() { grep -q "$1" "$log" || fail "missing \"$1\""; }

boot "neo.test" "-kernel $image -initrd $bootfs $withdisk -device ramfb"
[ "$status" -eq 33 ] || fail "exit status $status, expected 33"
expect "Neo Boing Platinum OS Lite: Fortress .* (aarch64)"
expect "Running in EL1 (started in EL1)"
expect "Exception handlers installed."
expect "Device tree at 0x4"
expect "mem 0x0040000000..0x0060000000 usable"
expect "Usable RAM: 512 MiB"
expect "Free memory 51[01] MiB"
expect "Fortress is up."
expect "Self-test: breakpoint caught"
expect "Self-test: frame allocator ok"
expect "Kernel page tables active: 4 GiB identity mapped"
expect "Self-test: page tables ok"
expect "Self-test: heap ok"
expect "Timer running at 100 Hz"
expect "Self-test: scheduler ok"
expect "Self-test: sleep and park ok"
expect "Self-test: message ports ok"
expect "Self-test: FP registers across switches ok"
expect "Hello from Rust in user mode!"
expect "fib(50) = 12586269025"
expect "Self-test: Rust programs ok"
expect "Program crash stopped: data abort"
expect "Boot image at 0x[0-9a-f]*: 8 files"
expect "Self-test: boot image ok (8 files"
expect "runner: README says: Neo Boing Platinum OS Lite boot image"
expect "runner: bin/crash was stopped, as it should be"
expect "Disk: virtio, [0-9]* sectors"
expect "Self-test: disk ok (8 files on [0-9]* KiB, programs match the boot image, hello-rust ran from the disk, 0 frames leaked)"
expect "Self-test: faulty program stopped ok"
expect "Screen: ramfb 800x600 at 0x4[0-9a-f]*"
expect "Self-test: terminal window ok (text shown in its window, a key typed there reached the console, it showed 13 bytes, 0 frames leaked)"
expect "Self-test: shared-memory ring ok (1024 KiB through a 64 KiB ring, 0 bytes wrong, [0-9]* doorbells, writer waited [0-9]* times, 0 frames leaked)"
expect "Self-test: layout switch on the Bar ok (Bar showed US, showed SR after the driver switched, a click switched back, 0 frames leaked)"
expect "Self-test: a program's window ok (hello-window ended with 0, 4 of 4 pixels of its window right, 0 frames leaked)"
expect "Self-test: display service ok (800x600 drawn from user mode at 0x4[0-9a-f]*, 6 of 6 pixels right, Dijana [0-9]* pixels and moving (1 tip after an error), clock redrawn [2-9][0-9]* times, 0 frames leaked)"

boot "neo.test=undefined" "-kernel $image"
[ "$status" -eq 35 ] || fail "exit status $status, expected 35"
expect "CPU EXCEPTION: unknown instruction (synchronous) from EL1"
expect "esr 0x02000000"
expect "FORTRESS PANIC: unhandled CPU exception"

boot "neo.test=pagefault" "-kernel $image"
[ "$status" -eq 35 ] || fail "exit status $status, expected 35"
expect "CPU EXCEPTION: data abort (synchronous) from EL1"
expect "page fault at 0x0000004000000000: read on a missing page, kernel mode"

boot "neo.test=stackoverflow" "-kernel $image"
[ "$status" -eq 35 ] || fail "exit status $status, expected 35"
expect "write on a missing page, kernel mode"
expect "kernel stack overflow"

boot "neo.test" "-kernel $elf -machine virtualization=on -cpu max -smp 2 -m 1G"
[ "$status" -eq 33 ] || fail "exit status $status, expected 33"
expect "Running in EL1 (started in EL2)"
expect "Usable RAM: 1024 MiB"
expect "Self-test: scheduler ok"
expect "Self-test: FP registers across switches ok"
expect "Self-test: Rust programs ok"
expect "Self-test: boot image ok (skipped: no boot image given)"
expect "Self-test: disk ok (skipped: no disk)"
expect "Self-test: display service ok (skipped: no screen)"
expect "Self-test: a program's window ok (skipped: no screen)"
expect "Self-test: terminal window ok (skipped: no screen)"

# Neo DOS: a short session typed on the serial console, on both volumes.
input='help\rinfo\rdir\rtype README\rtype DH0:README\rcd dh0:\rcrash\rnope\rboot:\recho Zdravo, Neo!\rcho Strelice rade\033[He\recho Brisanje Xradi\033[D\033[D\033[D\033[D\010\033[F!\recho DeleteX\033[D\033[3~ radi\r\033[A\033[A\rhistory\recho Nazad na nacrt\033[A\033[A\033[B\033[B\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $withdisk -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "exit status $status, expected 33"
expect "Neo DOS 0.3. Type HELP for the commands."
expect "DIR .vol:., TYPE <file>, RUN <program>"
expect "  BOOT:  *8 files"
expect "  DH0:  *8 files"
expect "  bin/neodos "
expect "8 files on BOOT:"
expect "Neo Boing Platinum OS Lite boot image"
expect "Neo Boing Platinum OS Lite disk DH0:"
expect "DH0:> crash"
expect "crash was stopped by the system; Neo DOS carries on"
expect "nope: unknown command or program"
expect "BOOT:> echo Zdravo, Neo!"
# Line editing over the serial line (VT100 keys): Home, Left, Backspace, End, Delete.
expect "^Strelice rade"
expect "^Brisanje radi!"
expect "^Delete radi"
# History: Up twice runs "echo Brisanje radi!" again; HISTORY lists the lines;
# Down past the newest brings back the line being typed.
[ "$(grep -c "^Brisanje radi!" "$log")" -eq 2 ] || fail "Up did not bring back an earlier line"
expect "^ 14  echo Brisanje radi!"
expect "^ 15  history"
expect "^Nazad na nacrt"
expect "Neo DOS ended (code 0), [1-9][0-9]* input interrupts."
expect "Display service stopped after [1-9][0-9]* clock redraws; Dijana gave 2 tips and said good night."
expect "Terminal showed [1-9][0-9]* bytes."

# Neo DOS typed on a virtio keyboard: QEMU's monitor presses the keys, the
# virtio-input driver turns them into characters.
python3 scripts/sendkeys.py "$sock" 2 'echo Typed on virtio!\rcho Strelice na virtio{sendkey home}e{sendkey end}!\recho AXB{sendkey left}{sendkey left}{sendkey delete}C\r{sendkey alt-shift}echo ;[]\r{sendkey alt-shift}{mouse_move 30 20}{mouse_move 10 5}{mouse_button 1}{mouse_button 0}bootlog fortress\rendcli\r' &
keys=$!
boot "neo.shell" "-kernel $image -initrd $bootfs -device virtio-keyboard-device -device virtio-mouse-device -device ramfb -monitor unix:$sock,server,nowait"
wait $keys || true
[ "$status" -eq 33 ] || fail "exit status $status, expected 33"
expect "Keyboard: virtio-input (interrupt [0-9]*)"
expect "Pointer: virtio-input (interrupt [0-9]*)"
expect "Pointer: [1-9][0-9]* moves, at (440, 325)."
expect "BOOT:> echo Typed on virtio!"
expect "^Typed on virtio!"
# Alt+Shift switched to Serbian Latin: the keys right of L and P typed č, š, đ.
expect "BOOT:> echo čšđ"
expect "^čšđ"
# Arrows, Home, End and Delete on the virtio keyboard.
expect "^Strelice na virtio!"
expect "^ACB"
# BOOTLOG finds the kernel's banner and "Fortress is up." in its log.
expect "^2 lines with \"fortress\"."
expect "Neo DOS ended (code 0)"

echo "BOOT TEST PASSED"
