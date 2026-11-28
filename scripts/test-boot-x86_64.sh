#!/bin/sh
# Boot tests for Fortress on x86_64 under QEMU.
#   1. Normal boot: banner, memory map, self-tests (exceptions, frames, paging,
#      heap, timer and scheduler), exit status 33 (pass).
#   2. Page fault: reading unmapped memory must be reported, then exit 35 (kernel panic).
#   3. Stack overflow: must hit the guard page and be reported as a double fault.
#   4. Neo DOS: a short session typed on the serial console (with BOOTLOG);
#      Dijana gives a tip after each error and says good night at the end.
#   5. Neo DOS typed on the PS/2 keyboard (QEMU's sendkey), through the
#      keyboard service in user mode.
#   6. USB: an xHCI controller with a USB keyboard, mouse and tablet plugged
#      in; each is set up, and the keyboard and mouse found as boot devices.
#   7. Neo DOS typed on the USB keyboard and the pointer moved by the USB
#      mouse (QEMU's sendkey and mouse_move), with no PS/2 input at all.
#      7b, 7c: typing on a controller with only MSI, and on one with no
#      message interrupts (the USB thread polls).
#   8. The same keyboard and mouse behind a USB hub, as on a PC whose
#      front ports or keyboard have a hub inside.
#   9. Without QEMU's test exit device (and Neo DOS's USB and PCI lists), as on a real PC: when Neo DOS ends
#      the kernel switches the machine off through ACPI (sleep state S5).
#  10. Hot-plug: a USB keyboard plugged in after boot types into Neo DOS,
#      then is pulled out; its slot and memory are given back. (Runs
#      before 9, which leaves out the test exit device.)
#  11. Hot-plug behind a hub: a keyboard plugged into a hub port after
#      boot is set up and pulled out; then a second hub with a keyboard on it
#      is plugged in and pulled out as a whole.
set -e
cd "$(dirname "$0")/.."
input=""
extra=""
# The test exit device; boot 9 leaves it out to see the machine switch off.
exitdev="-device isa-debug-exit,iobase=0xf4,iosize=0x04"
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
        $exitdev \
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

extra="-smp 2"
boot "neo.test"
extra=""
[ "$status" -eq 33 ] || fail "exit status $status, expected 33"
expect "Neo Boing Platinum OS Lite: Fortress"
# Two processors and the power registers, from the ACPI tables.
expect "ACPI [0-9.+]*: 2 CPUs, I/O APIC at 0xfec00000, [0-9]* interrupt overrides, power off ready."
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
expect "runner: README says: Neo Boing Platinum OS Lite boot image"
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
expect "Self-test: display service ok (800x600 drawn from user mode at 0x[0-9a-f]*, 6 of 6 pixels right, Dijana [0-9]* pixels and moving (1 tip after an error), clock redrawn [2-9][0-9]* times, 0 frames leaked)"
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

# 6. USB: the controller starts, its rings work, all three devices are set up.
# As on a real PC, the controller sits behind a PCI Express root port.
input=""
extra="-device pcie-root-port,id=rp1,chassis=1 -device qemu-xhci,bus=rp1 -device usb-kbd -device usb-mouse -device usb-tablet"
boot "neo.test"
[ "$status" -eq 33 ] || fail "USB boot: exit status $status, expected 33"
expect "PCI: [0-9]* functions on 2 buses."
expect "USB controller at PCI 01:00.0, registers at 0x"
expect "USB: xHCI 1\.[0-9], [0-9]* ports, 3 with a device plugged in, [0-9]* scratchpad pages; command ring ok; interrupts by MSI-X"
expect "USB port [0-9]*: 0627:0001, .*, keyboard (boot protocol), reports on endpoint 0x81, 8 bytes"
expect "USB port [0-9]*: 0627:0001, .*, mouse (boot protocol), reports on endpoint 0x81, 4 bytes"
expect "USB port [0-9]*: 0627:0001, .*, not a boot keyboard or mouse"
extra=""

# 7. Neo DOS on the USB keyboard and mouse.
extra="-device qemu-xhci -device usb-kbd -device usb-mouse -monitor unix:$sock,server,nowait"
python3 scripts/sendkeys.py "$sock" 3 'echo Typed on USB!\r{mouse_move 30 20}{mouse_move 10 5}endcli\r' &
keys=$!
boot "neo.shell"
wait $keys || true
extra=""
[ "$status" -eq 33 ] || fail "USB keyboard: exit status $status, expected 33"
expect "USB keyboard and mouse: reading reports."
expect "USB interrupts: MSI-X on vector 48 (a No-Op command raised one)."
expect "BOOT:> echo Typed on USB!"
expect "^Typed on USB!"
expect "Neo DOS ended (code 0), 0 input interrupts."
expect "Pointer: [1-9][0-9]* moves, at (4[0-9][0-9], 3[0-9][0-9])."
expect "USB: [1-9][0-9]* keys, [1-9][0-9]* mouse reports, [1-9][0-9]* interrupts."

# 7b and 7c. QEMU's xHCI above uses MSI-X; here a controller with only
# MSI, then one with no message interrupts at all (the USB thread polls):
# typing works both ways.
for mode in "msix=off:MSI" "msi=off,msix=off:polling"; do
    props=${mode%%:*}
    want=${mode#*:}
    extra="-device nec-usb-xhci,$props -device usb-kbd -monitor unix:$sock,server,nowait"
    python3 scripts/sendkeys.py "$sock" 3 'echo Typed with '"$want"'!\rendcli\r' &
    keys=$!
    boot "neo.shell"
    wait $keys || true
    extra=""
    [ "$status" -eq 33 ] || fail "USB keyboard ($want): exit status $status, expected 33"
    expect "command ring ok; interrupts by $want"
    expect "^Typed with $want!"
    if [ "$want" = polling ]; then
        expect "USB controller has no MSI or MSI-X; polling 100 times a second."
        expect "USB: [1-9][0-9]* keys, 0 mouse reports, 0 interrupts."
    else
        expect "USB interrupts: MSI on vector 48 (a No-Op command raised one)."
        expect "USB: [1-9][0-9]* keys, 0 mouse reports, [1-9][0-9]* interrupts."
    fi
done

# 8. Neo DOS on a USB keyboard and mouse behind a hub.
extra="-device qemu-xhci,id=xhci -device usb-hub,bus=xhci.0,port=1 -device usb-kbd,bus=xhci.0,port=1.1"
extra="$extra -device usb-mouse,bus=xhci.0,port=1.3 -monitor unix:$sock,server,nowait"
python3 scripts/sendkeys.py "$sock" 3 'echo Through the hub!\r{mouse_move 30 20}usb\rendcli\r' &
keys=$!
boot "neo.shell"
wait $keys || true
extra=""
[ "$status" -eq 33 ] || fail "USB hub: exit status $status, expected 33"
expect "USB port [0-9]*: 0409:55aa, full speed, slot 1, hub, 8 ports"
expect "USB port [0-9]*\.1: 0627:0001, .*, keyboard (boot protocol)"
expect "USB port [0-9]*\.3: 0627:0001, .*, mouse (boot protocol)"
expect "^Through the hub!"
# USB lists what is there now, in port order.
expect "^USB port [0-9]*: 0409:55aa, .*, hub, 8 ports"
expect "^USB port [0-9]*\.1: 0627:0001, .*, keyboard (boot protocol)"
expect "^3 USB devices."
expect "USB: [1-9][0-9]* keys, [1-9][0-9]* mouse reports, [1-9][0-9]* interrupts."

# 10. A USB keyboard plugged in while Neo DOS runs, used, then pulled out.
# Afterwards the PS/2 keyboard ends Neo DOS.
extra="-device qemu-xhci,id=xhci -monitor unix:$sock,server,nowait"
python3 scripts/sendkeys.py "$sock" 3 '{device_add usb-kbd,bus=xhci.0,id=hot}{sleep 2}echo Plugged in!\r{sleep 1}{device_del hot}{sleep 2}endcli\r' &
keys=$!
boot "neo.shell"
wait $keys || true
extra=""
[ "$status" -eq 33 ] || fail "USB hot-plug: exit status $status, expected 33"
expect "USB: no keyboard or mouse yet; watching the ports."
expect "Plugged in: USB port [0-9]*: 0627:0001, .*, keyboard (boot protocol)"
expect "^Plugged in!"
expect "Pulled out: USB port [0-9]* (1 device)."
expect "USB hot-plug: 1 plugged in, 1 pulled out."
expect "Neo DOS ended (code 0)"

# 11. Hot-plug behind a hub (its status change endpoint), and a hub
# plugged into a hub. The keyboard that ends Neo DOS stays on the first hub.
extra="-device qemu-xhci,id=xhci -device usb-hub,bus=xhci.0,port=1,id=h1 -device usb-kbd,bus=xhci.0,port=1.1"
extra="$extra -monitor unix:$sock,server,nowait"
python3 scripts/sendkeys.py "$sock" 3 '{device_add usb-kbd,bus=xhci.0,port=1.4,id=hot}{sleep 2}{device_del hot}{sleep 2}{device_add usb-hub,bus=xhci.0,port=1.2,id=h2}{sleep 3}{device_add usb-kbd,bus=xhci.0,port=1.2.3,id=k2}{sleep 2}{device_del h2}{sleep 2}endcli\r' &
keys=$!
boot "neo.shell"
wait $keys || true
extra=""
[ "$status" -eq 33 ] || fail "USB hot-plug behind a hub: exit status $status, expected 33"
expect "USB port [0-9]*: 0409:55aa, .*, hub, 8 ports, reports on endpoint 0x81"
expect "Plugged in: USB port [0-9]*\.4: 0627:0001, .*, keyboard (boot protocol)"
expect "Pulled out: USB port [0-9]*\.4 (1 device)."
expect "Plugged in: USB port [0-9]*\.2: 0409:55aa, .*, hub, 8 ports"
expect "Plugged in: USB port [0-9]*\.2\.3: 0627:0001, .*, keyboard (boot protocol)"
expect "Pulled out: USB port [0-9]*\.2 (2 devices)."
expect "USB hot-plug: 3 plugged in, 2 pulled out."
expect "Neo DOS ended (code 0)"

# 9. Switching off through ACPI when Neo DOS ends.
# BOOTLOG shows the kernel's messages first (as a word search and as
# pages), as a tester would read them on a real PC's screen.
exitdev=""
input="bootlog fortress\rbootlog\rbootlog 99\rusb\rpci\rendcli\r"
boot "neo.shell"
input=""
exitdev="-device isa-debug-exit,iobase=0xf4,iosize=0x04"
[ "$status" -eq 0 ] || fail "power off: exit status $status, expected 0 (QEMU switched off)"
expect "^2 lines with \"fortress\"."
expect "^Lines 1-20 of [0-9]*. BOOTLOG 2 for more."
expect "^The log has [0-9]* lines ([0-9]* pages)."
# USB and PCI: what the machine has, for reports from real PCs.
expect "^No USB devices set up."
expect "^00:00.0  8086:29c0  060000  host bridge"
expect "^00:[0-9a-f.]*  1af4:1001  010000  SCSI storage"
expect "^[0-9]* PCI devices."
expect "Neo DOS ended (code 0)"
expect "Switching off."

echo "BOOT TEST PASSED"
