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
#  12. Shut down from the Neo menu on the Bar, with the mouse only.
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
rwdisk=$(mktemp)
trap 'rm -f "$log" "$sock" "$rwdisk"' EXIT
# Boots use the disk read-only, unless a test asks for a writable copy
# (fresh_disk), which the boot may then change.
diskopt="readonly=on"
diskfile="$disk"
diskdev="-device virtio-blk-pci,drive=d0,disable-modern=on"
fresh_disk() { cp "$disk" "$rwdisk"; diskfile="$rwdisk"; diskopt="cache=writethrough"; }
read_only_disk() { diskfile="$disk"; diskopt="readonly=on"; }

boot() { # $1 = kernel command line; sets $status, output in $log
    set +e
    # Typed input (for neo.shell) comes from $input, after a second.
    (sleep 1; printf "$input") | timeout 30 qemu-system-x86_64 -machine q35 -m 512M -display none -serial stdio -no-reboot \
        $exitdev \
        -drive file="$diskfile",format=raw,if=none,id=d0,$diskopt \
        $diskdev \
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
expect "Self-test: disk ok (11 files on [0-9]* KiB, programs match the boot image, hello-rust ran from the disk, disk is read-only, writes refused, 0 frames leaked)"
expect "Self-test: faulty program stopped ok"
expect "Hello through the console service!"
expect "6 x 7 = 42 (a longer line, sent in pieces of 16 bytes)"
expect "Self-test: console service ok (client exit 0"
expect "Screen: VGA (Bochs display) 800x600 at 0xfd000000"
expect "Self-test: mouse and pointer ok (3 packets through IRQ 12, pointer at (440, 330) with button 1, arrow drawn, window dragged, 0 frames leaked)"
expect "Self-test: clicks and keys in a window ok (click turned the disc, typing showed in the window, the Bar listed it, it minimized and came back, its close box closed it, program ended with 12, keys reach the console after, Welcome closed and came back, 0 frames leaked)"
expect "Self-test: terminal window ok (text shown in its window, a key typed there reached the console, it showed 13 bytes, 0 frames leaked)"
expect "Self-test: shared-memory ring ok (1024 KiB through a 64 KiB ring, 0 bytes wrong, [0-9]* doorbells, writer waited [0-9]* times, 0 frames leaked)"
expect "Self-test: layout switch on the Bar ok (Bar showed US, showed SR after the driver switched, a click switched back, 0 frames leaked)"
expect "Self-test: a program's window ok (hello-window ended with 0, 4 of 4 pixels of its window right, 0 frames leaked)"
expect "Self-test: Calculator tile ok (7\*6= shows 42, C clears, 1/0= gave 1 tip, 0 frames leaked)"
expect "Self-test: display service ok (800x600 drawn from user mode at 0x[0-9a-f]*, 8 of 8 pixels right, Dijana [0-9]* pixels and moving (1 tip after an error), clock redrawn [2-9][0-9]* times, 0 frames leaked)"
expect "Self-test: keyboard service ok (28 keys typed .Hi 42.n.*z.n. through IRQ 1, driver ended with 10, 0 frames leaked)"

# The same self-test on a writable copy of the disk: files are written,
# rewritten and deleted, and the host's tar still reads the image.
fresh_disk
boot "neo.test"
[ "$status" -eq 33 ] || fail "writable disk: exit status $status, expected 33"
expect "Self-test: disk ok (11 files on [0-9]* KiB, programs match the boot image, hello-rust ran from the disk, written, rewritten and deleted files, 0 frames leaked)"
tar -tf "$rwdisk" >/dev/null 2>&1 || fail "tar cannot read the disk after writing"
[ "$(tar -tf "$rwdisk" | grep -c '^selftest/note.txt$')" -eq 3 ] || fail "the disk does not hold the three entries of note.txt (first, second, deleted)"
[ "$(tar -xOf "$rwdisk" selftest/note.txt | head -c 5)" = "first" ] || fail "tar does not read the first copy"
read_only_disk

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
expect "Neo DOS 0.4. Type HELP for the commands."
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

# The same disk behind an NVMe controller: read, written and kept as before;
# a disk that is not a Neo disk is read but never written.
fresh_disk
diskdev="-device nvme,drive=d0,serial=neo0001"
boot "neo.test"
[ "$status" -eq 33 ] || fail "NVMe disk: exit status $status, expected 33"
expect "^Disk: NVMe, [0-9]* sectors (4 MiB)"
expect "Self-test: disk ok (11 files on [0-9]* KiB, programs match the boot image, hello-rust ran from the disk, written, rewritten and deleted files, 0 frames leaked)"
tar -tf "$rwdisk" >/dev/null 2>&1 || fail "tar cannot read the NVMe disk after writing"
head -c 4194304 /dev/zero > "$rwdisk"
input='info\rwrite dh0:x no\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "NVMe disk of someone else's: exit status $status, expected 33"
expect "^Disk: NVMe, [0-9]* sectors (4 MiB), read only (not a Neo disk)"
expect "(disk, read only)"
expect "dh0:x: that volume is read only"
[ "$(tr -d '\0' < "$rwdisk" | wc -c)" -eq 0 ] || fail "something was written to a disk that is not a Neo disk"
diskdev="-device virtio-blk-pci,drive=d0,disable-modern=on"
read_only_disk

# Neo DOS writing to the disk (WRITE, COPY, DELETE), and that what it wrote
# is still there in a second boot of the same disk. BOOT: stays read only.
fresh_disk
input='info\rwrite dh0:notes/hello.txt Zdravo disk\rcopy boot:README dh0:copy.txt\rtype dh0:notes/hello.txt\rdelete dh0:copy.txt\rwrite boot:x no\rdelete boot:README\rwrite dh0:bin/x no\rdelete dh0:nope\rinfo\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "writing from Neo DOS: exit status $status, expected 33"
expect "(disk, read and write, [0-9]* KiB free)"
expect "Wrote 11 bytes to dh0:notes/hello.txt"
expect "Copied [0-9]* bytes to dh0:copy.txt"
expect "^Zdravo disk"
expect "Deleted dh0:copy.txt"
expect "boot:x: that volume is read only"
expect "boot:README: that volume is read only"
expect "dh0:bin/x: that volume is read only (or bin/ is kept)"
expect "dh0:nope: no such file"
tar -tf "$rwdisk" >/dev/null 2>&1 || fail "tar cannot read the disk after Neo DOS wrote to it"
[ "$(tar -xOf "$rwdisk" notes/hello.txt)" = "Zdravo disk" ] || fail "tar does not read the file Neo DOS wrote"
# Scripts: ECHO > and >> make one a line at a time, EXECUTE runs it, and a
# startup sequence on the disk runs by itself at the next boot.
input='cd dh0:\recho echo Zdravo iz skripte > s/startup-sequence\recho ; komentar >> s/startup-sequence\recho info >> s/startup-sequence\rexecute s/startup-sequence\rwrite s/nest execute s/startup-sequence\rexecute s/nest\rexecute nope\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "scripts: exit status $status, expected 33"
expect "Wrote 23 bytes to s/startup-sequence"
expect "Added 11 bytes to s/startup-sequence"
expect "^DH0:> echo Zdravo iz skripte"
expect "^Zdravo iz skripte"
expect "^DH0:> info"
expect "EXECUTE: scripts do not run scripts"
expect "nope: no such file"
# The same disk, booted again: the file is still there, the deleted one is not.
input='dir dh0:\rtype dh0:notes/hello.txt\rtype dh0:copy.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "second boot of the written disk: exit status $status, expected 33"
expect "  notes/hello.txt "
expect "  notes/hello.txt  *11 bytes  $(date -u +%Y-%m-%d) [0-9][0-9]:[0-9][0-9]"
expect "^Running DH0:s/startup-sequence"
expect "^BOOT:> echo Zdravo iz skripte"
expect "^Zdravo disk"
expect "dh0:copy.txt: no such file"
if grep -q "  copy.txt " "$log"; then fail "the deleted file is listed after a reboot"; fi
# RENAME and SEARCH, on the same disk.
input='write dh0:r/a.txt Zdravo svete\rrename dh0:r/a.txt dh0:r/b.txt\rrename dh0:r/b.txt boot:x\rtype dh0:r/b.txt\rtype dh0:r/a.txt\rrename boot:README boot:z\rsearch ZDRAVO dh0:\rsearch Neo boot:\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "rename and search: exit status $status, expected 33"
expect "Renamed dh0:r/a.txt to dh0:r/b.txt"
expect "RENAME stays on one volume"
expect "^Zdravo svete"
expect "dh0:r/a.txt: no such file"
expect "boot:README: that volume is read only"
expect "^notes/hello.txt:1: Zdravo disk"
expect "^r/b.txt:1: Zdravo svete"
expect "^s/startup-sequence:1: echo Zdravo iz skripte"
expect "^3 lines in 3 files on DH0:"
expect "^README:1: Neo Boing Platinum OS Lite boot image"
tar -tf "$rwdisk" >/dev/null 2>&1 || fail "tar cannot read the disk after RENAME"
# HEAD and TAIL, on a file of four lines.
input='echo uno > dh0:l.txt\recho dva >> dh0:l.txt\recho tri >> dh0:l.txt\recho cet >> dh0:l.txt\rhead dh0:l.txt 1\rtail dh0:l.txt 2\rdelete dh0:l.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "HEAD and TAIL: exit status $status, expected 33"
expect "^uno"
expect "^tri"
expect "^cet"
if grep -q "^dva" "$log"; then fail "HEAD or TAIL shows a line it should not"; fi
# WC, on a file of two lines and three words.
input='echo uno dva > dh0:w.txt\recho tri >> dh0:w.txt\rwc dh0:w.txt\rdelete dh0:w.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "WC: exit status $status, expected 33"
expect "^2 lines, 3 words, 12 bytes"
# FIND, files by a part of their names.
input='write dh0:f/aaa.txt x\rwrite dh0:f/bbb.txt y\rfind AAA dh0:\rdelete dh0:f/aaa.txt\rdelete dh0:f/bbb.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "FIND: exit status $status, expected 33"
expect "^DH0:f/aaa.txt"
expect "^1 of [0-9]* files on DH0:"
if grep -q "^DH0:f/bbb.txt" "$log"; then fail "FIND lists a file it should not"; fi
# CMP, the same files and different ones.
input='write dh0:c1 abcd\rwrite dh0:c2 abcd\rwrite dh0:c3 abXd\rwrite dh0:c4 abc\rcmp dh0:c1 dh0:c2\rcmp dh0:c1 dh0:c3\rcmp dh0:c1 dh0:c4\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "CMP: exit status $status, expected 33"
expect "^The files are the same."
expect "^The files differ at byte 3 (line 1)."
expect "^The files differ in size: 4 and 3 bytes."
# SORT, three lines out of order.
input='echo cc > dh0:s.txt\recho aa >> dh0:s.txt\recho bb >> dh0:s.txt\rsort dh0:s.txt\rdelete dh0:s.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "SORT: exit status $status, expected 33"
expect "^aa"
[ "$(sed -n '/> sort /,/> delete /p' "$log" | grep -E '^(aa|bb|cc)$' | tr -d '\r\n')" = "aabbcc" ] || fail "SORT does not put the lines in order"
# HEXDUMP, a three-byte file.
input='write dh0:h.txt ABC\rhexdump dh0:h.txt\rdelete dh0:h.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "HEXDUMP: exit status $status, expected 33"
expect "^00000000  41 42 43 .*|ABC|"
# CALC, precedence and brackets, and a mistake.
input='calc 2+3*4\rcalc (2+3)*4-6/2\rcalc 7/0\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "CALC: exit status $status, expected 33"
expect "^14"
expect "^17"
expect "^CALC <sum>, for example"
# UNIQ, repeated lines in a row.
input='echo aa > dh0:u.txt\recho aa >> dh0:u.txt\recho bb >> dh0:u.txt\recho aa >> dh0:u.txt\runiq dh0:u.txt\rdelete dh0:u.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "UNIQ: exit status $status, expected 33"
[ "$(sed -n '/> uniq /,/> delete /p' "$log" | grep -E '^(aa|bb)$' | tr -d '\r\n')" = "aabbaa" ] || fail "UNIQ shows the wrong lines"
# NUMBER, lines with numbers.
input='echo aa > dh0:n.txt\recho bb >> dh0:n.txt\rnumber dh0:n.txt\rdelete dh0:n.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "NUMBER: exit status $status, expected 33"
expect "^   1  aa"
expect "^   2  bb"
# REPEAT, a command three times.
input='repeat 3 echo ponovo\rrepeat 0 echo no\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "REPEAT: exit status $status, expected 33"
[ "$(grep -c '^ponovo' "$log")" -eq 3 ] || fail "REPEAT does not run the command three times"
expect "^REPEAT <1 to 20>"
# TAC, lines backwards.
input='echo aa > dh0:t.txt\recho bb >> dh0:t.txt\recho cc >> dh0:t.txt\rtac dh0:t.txt\rdelete dh0:t.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "TAC: exit status $status, expected 33"
[ "$(sed -n '/> tac /,/> delete /p' "$log" | grep -E '^(aa|bb|cc)$' | tr -d '\r\n')" = "ccbbaa" ] || fail "TAC shows the lines in the wrong order"
# ALIAS, a short name with and without more words after it.
input='alias hi echo Zdravo\rhi\rhi svete\ralias\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "ALIAS: exit status $status, expected 33"
[ "$(sed -n "/> hi/,\$p" "$log" | grep -c "^Zdravo")" -ge 2 ] || fail "the alias does not run"
expect "^Zdravo svete"
expect "^hi = echo Zdravo"
# WAIT, two seconds by the clock.
input='wait 2\rwait 0\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "WAIT: exit status $status, expected 33"
expect "^Waited 2 seconds."
expect "^WAIT <1 to 60> seconds"
# DIR with a pattern.
input='write dh0:p/ab.txt 1\rwrite dh0:p/cd.dat 2\rdir dh0: p/*.TXT\rdir dh0:p/?d.*\rdelete dh0:p/ab.txt\rdelete dh0:p/cd.dat\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "DIR pattern: exit status $status, expected 33"
[ "$(sed -n '/> dir dh0: p/,/> delete /p' "$log" | grep -c '^  p/')" -eq 2 ] || fail "DIR with a pattern lists the wrong files"
expect "^1 files on DH0:"
# TOUCH, a new empty file and an old file that keeps its text.
input='touch dh0:e.txt\rwrite dh0:o.txt Zdravo\rtouch dh0:o.txt\rdir dh0: e.txt\rtype dh0:o.txt\rdelete dh0:e.txt\rdelete dh0:o.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "TOUCH: exit status $status, expected 33"
expect "^Made dh0:e.txt"
expect "^Touched dh0:o.txt"
expect "^  e.txt  *0 bytes"
expect "^Zdravo"
# REPLACE, a word changed twice in a file.
input='write dh0:r.txt mir i mir\rreplace dh0:r.txt mir svet\rtype dh0:r.txt\rreplace dh0:r.txt xyz a\rdelete dh0:r.txt\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "REPLACE: exit status $status, expected 33"
expect "^Replaced 2 times in dh0:r.txt"
expect "^svet i svet"
expect "^dh0:r.txt: nothing to replace"
# SET, a variable used with $, an unset one, and the list.
input='set kome svete\recho Zdravo $kome$\recho [$nema]\rset\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "SET: exit status $status, expected 33"
expect "^Zdravo svete"
expect "^\\[\\]"
expect "^kome = svete"
# The clock: DATE shows the machine's time (QEMU's is the host's), and a
# file written to the disk carries it as its modification time.
input='date\rendcli\r'
boot "neo.shell"
input=""
[ "$status" -eq 33 ] || fail "DATE: exit status $status, expected 33"
today=$(date -u +%Y-%m-%d)
expect "^Clock: $today [0-9][0-9]:[0-9][0-9]:[0-9][0-9] UTC"
expect "^$today [0-9][0-9]:[0-9][0-9]:[0-9][0-9] UTC"
tar -tvf "$rwdisk" notes/hello.txt | grep -q "$today" || fail "a written file does not carry today's date"
read_only_disk

# 12. The Neo menu: a click on "Neo" on the Bar opens it, "About" makes
# Dijana answer, then "Shut down" ends Neo DOS (as ENDCLI) and the system
# stops with Dijana's good night. Nothing is typed at all.
extra="-monitor unix:$sock,server,nowait"
click='{sleep 0.2}{mouse_button 1}{sleep 0.2}{mouse_button 0}{sleep 0.3}'
python3 scripts/sendkeys.py "$sock" 4 "{mouse_move -360 280}$click{mouse_move 60 -94}$click{mouse_move -60 94}$click{mouse_move 60 -42}$click" &
keys=$!
boot "neo.shell"
wait $keys || true
extra=""
[ "$status" -eq 33 ] || fail "Neo menu: exit status $status, expected 33"
expect "^Bye."
expect "Neo DOS ended (code 0)"
expect "Dijana gave 0 tips and said good night."

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
