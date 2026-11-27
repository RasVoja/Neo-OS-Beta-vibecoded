#!/bin/sh
# Boot tests for Fortress on x86_64 under QEMU.
#   1. Normal boot: banner, memory map, self-tests (exceptions, frames, paging,
#      heap, timer and scheduler), exit status 33 (pass).
#   2. Page fault: reading unmapped memory must be reported, then exit 35 (kernel panic).
#   3. Stack overflow: must hit the guard page and be reported as a double fault.
#   4. Neo DOS: a short session typed on the serial console.
#   5. Neo DOS typed on the PS/2 keyboard (QEMU's sendkey), through the
#      keyboard service in user mode.
#   6. USB: an xHCI controller with a USB keyboard and mouse plugged in.
set -e
cd "$(dirname "$0")/.."
input=""
extra=""
cargo build --release
kernel=target/x86_64-unknown-none/release/fortress
bootfs=$(./scripts/make-bootfs.sh x86_64)
# The disk (DH0:) holds the same programs with its own README.
disk=$(./scripts/make-bootfs.sh x86_64 disk)
log=$(mktemp)
sock=$(mktemp -u)
trap 'rm -f "$log" "$sock"' EXIT

boot() { # $1 = kernel command line; sets $status, output in $log
    set +e
    # Typed input (for neo.shell) comes from $input, after a second.
    (sleep 1; printf "$input") | timeout 30 qemu-system-x86_64 -machine q35 -m 512M -display none -serial stdio -no-reboot \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
        -drive file="$disk",format=raw,if=none,id=d0,readonly=on \
        -device virtio-blk-pci,drive=d0,disable-modern=on \
        -kernel "$kernel" -initrd "$bootfs" $extra -append "$1" >"$log" 2>&1
    status=$?
    set -e
    echo "--- boot: $1 (exit $status)"
    cat "$log"
}
fail() { echo "BOOT TEST FAILED: $1"; exit 1; }
expect() { grep -q "$1" "$log" || fail "missing \"$1\""; }

boot "neo.test"
[ "$status" -eq 33 ] || fail "exit status $status, expected 33"
expect "NeoOS Lite: Fortress"
expect "Exception handlers installed."
expect "Usable RAM: 5[01][0-9] MiB"
expect "Fortress is up."
expect "Self-test: breakpoint caught"
expect "Free memory 5[01][0-9] MiB"
expect "Self-test: frame allocator ok"
expect "Kernel page tables active: 4 GiB identity mapped"
expect "Self-test: page tables ok"
expect "Self-test: heap ok"
expect "Timer running at 100 Hz"
expect "Self-test: scheduler ok"
expect "Self-test: sleep and park ok"
expect "Self-test: message ports ok"
expect "Self-test: FP registers across switches ok"
expect "Hello from ring 3!"
expect "Program peek stopped: page fault"
expect "Program halt stopped: general protection fault"
expect "Self-test: user mode ok (6 programs"
expect "Hello from Rust in user mode!"
expect "fib(50) = 12586269025"
expect "Self-test: Rust programs ok"
expect "Program crash stopped: general protection fault\|Program crash stopped: page fault"
expect "Boot image at 0x[0-9a-f]*: 11 files"
expect "bin/hello-rust ("
expect "Self-test: boot image ok (11 files"
expect "runner: README says: Neo OS Lite boot image"
expect "runner: bin/crash was stopped, as it should be"
expect "Disk: virtio, [0-9]* sectors"
expect "Self-test: disk ok (11 files on [0-9]* KiB, programs match the boot image, hello-rust ran from the disk, 0 frames leaked)"
expect "Self-test: faulty program stopped ok"
expect "Hello through the console service!"
expect "6 x 7 = 42 (a longer line, sent in pieces of 16 bytes)"
expect "Self-test: console service ok (client exit 0"
expect "Screen: VGA (Bochs display) 800x600 at 0xfd000000"
expect "Self-test: mouse and pointer ok (3 packets through IRQ 12, pointer at (440, 330) with button 1, arrow drawn, window dragged, 0 frames leaked)"
expect "Self-test: clicks and keys in a window ok (click turned the disc, typing showed in the window, Esc closed it, program ended with 12, keys reach the console after, 0 frames leaked)"
expect "Self-test: terminal window ok (text shown in its window, a key typed there reached the console, it showed 13 bytes, 0 frames leaked)"
expect "Self-test: shared-memory ring ok (1024 KiB through a 64 KiB ring, 0 bytes wrong, [0-9]* doorbells, writer waited [0-9]* times, 0 frames leaked)"
expect "Self-test: layout switch on the Bar ok (Bar showed US, showed SR after the driver switched, a click switched back, 0 frames leaked)"
expect "Self-test: a program's window ok (hello-window ended with 0, 4 of 4 pixels of its window right, 0 frames leaked)"
expect "Self-test: display service ok (800x600 drawn from user mode at 0x[0-9a-f]*, 5 of 5 pixels right, clock redrawn [2-9][0-9]* times, 0 frames leaked)"
expect "Self-test: keyboard service ok (28 keys typed .Hi 42.n.*z.n. through IRQ 1, driver ended with 10, 0 frames leaked)"

boot "neo.test=pagefault"
[ "$status" -eq 35 ] || fail "exit status $status, expected 35"
expect "CPU EXCEPTION 14: page fault"
expect "page fault at 0x0000004000000000: read on a missing page, kernel mode"
expect "FORTRESS PANIC: unhandled CPU exception 14"

boot "neo.test=stackoverflow"
[ "$status" -eq 35 ] || fail "exit status $status, expected 35"
expect "CPU EXCEPTION 8: double fault"
expect "kernel stack overflow"

# Neo DOS: a short session typed on the serial console, on both volumes.
input='help\rinfo\rdir\rtype README\rtype DH0:README\rcd dh0:\rcrash\rnope\rboot:\recho Zdravo, Neo!\rcho Strelice rade\033[He\recho Brisanje Xradi\033[D\033[D\033[D\033[D\010\033[F!\recho DeleteX\033[D\033[3~ radi\r\033[A\033[A\rhistory\recho Nazad na nacrt\033[A\033[A\033[B\033[B\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "exit status $status, expected 33"
expect "Neo DOS 0.3. Type HELP for the commands."
expect "DIR .vol:., TYPE <file>, RUN <program>"
expect "  BOOT:  *11 files"
expect "  DH0:  *11 files"
expect "  bin/neodos "
expect "11 files on BOOT:"
expect "Neo OS Lite boot image"
expect "Neo OS Lite disk DH0:"
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
expect "Display service stopped after [1-9][0-9]* clock redraws."
expect "Terminal showed [1-9][0-9]* bytes."

# Neo DOS typed on the PS/2 keyboard: QEMU's monitor presses the keys, the
# keyboard service in user mode turns them into characters.
extra="-monitor unix:$sock,server,nowait"
python3 scripts/sendkeys.py "$sock" 2 'echo Typed on PS/2!\rcho Strelice na PS/2{sendkey home}e{sendkey end}!\recho AXB{sendkey left}{sendkey left}{sendkey delete}C\r{sendkey alt-shift}echo ;[]\r{sendkey alt-shift}{mouse_move 30 20}{mouse_move 10 5}endcli\r' &
keys=$!
boot "neo.shell"
wait $keys || true
extra=""
[ "$status" -eq 33 ] || fail "exit status $status, expected 33"
expect "BOOT:> echo Typed on PS/2!"
expect "Pointer: [1-9][0-9]* moves, at (4[0-9][0-9], 3[0-9][0-9])."
expect "^Typed on PS/2!"
# Alt+Shift switched to Serbian Latin: the keys right of L and P typed č, š, đ.
expect "BOOT:> echo čšđ"
expect "^čšđ"
# Arrows, Home, End and Delete on the PS/2 keyboard.
expect "^Strelice na PS/2!"
expect "^ACB"
expect "Neo DOS ended (code 0)"

# 6. USB: the controller starts, its rings work, both devices are seen.
input=""
extra="-device qemu-xhci -device usb-kbd -device usb-mouse"
boot "neo.test"
[ "$status" -eq 33 ] || fail "USB boot: exit status $status, expected 33"
expect "USB: xHCI 1\.[0-9], [0-9]* ports, 2 with a device plugged in, [0-9]* scratchpad pages; command ring ok"
extra=""

echo "BOOT TEST PASSED"
