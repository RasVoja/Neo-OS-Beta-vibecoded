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
rwdisk=$(mktemp)
trap 'rm -f "$log" "$sock" "$rwdisk"' EXIT

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
expect "Self-test: disk ok (8 files on [0-9]* KiB, programs match the boot image, hello-rust ran from the disk, disk is read-only, writes refused, 0 frames leaked)"
expect "Self-test: faulty program stopped ok"
expect "Screen: ramfb 800x600 at 0x4[0-9a-f]*"
expect "Self-test: terminal window ok (text shown in its window, a key typed there reached the console, it showed 13 bytes, 0 frames leaked)"
expect "Self-test: shared-memory ring ok (1024 KiB through a 64 KiB ring, 0 bytes wrong, [0-9]* doorbells, writer waited [0-9]* times, 0 frames leaked)"
expect "Self-test: layout switch on the Bar ok (Bar showed US, showed SR after the driver switched, a click switched back, 0 frames leaked)"
expect "Self-test: a program's window ok (hello-window ended with 0, 4 of 4 pixels of its window right, 0 frames leaked)"
expect "Self-test: display service ok (800x600 drawn from user mode at 0x4[0-9a-f]*, 8 of 8 pixels right, Dijana [0-9]* pixels and moving (1 tip after an error), clock redrawn [2-9][0-9]* times, 0 frames leaked)"

# The same on a writable copy of the disk.
cp "$disk" "$rwdisk"
boot "neo.test" "-kernel $image -initrd $bootfs -drive file=$rwdisk,format=raw,if=none,id=d0,cache=writethrough -device virtio-blk-device,drive=d0 -device ramfb"
[ "$status" -eq 33 ] || fail "writable disk: exit status $status, expected 33"
expect "Self-test: disk ok (8 files on [0-9]* KiB, programs match the boot image, hello-rust ran from the disk, written, rewritten and deleted files, 0 frames leaked)"
tar -tf "$rwdisk" >/dev/null 2>&1 || fail "tar cannot read the disk after writing"

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
expect "Neo DOS 0.5. Type HELP for the commands."
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

# Neo DOS writing to the disk (WRITE, COPY, DELETE), and that what it wrote
# is still there in a second boot of the same disk. BOOT: stays read only.
cp "$disk" "$rwdisk"
rwdrive="-drive file=$rwdisk,format=raw,if=none,id=d0,cache=writethrough -device virtio-blk-device,drive=d0"
input='info\rwrite dh0:notes/hello.txt Zdravo disk\rcopy boot:README dh0:copy.txt\rtype dh0:notes/hello.txt\rdelete dh0:copy.txt\rwrite boot:x no\rdelete boot:README\rwrite dh0:bin/x no\rdelete dh0:nope\rinfo\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
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
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "scripts: exit status $status, expected 33"
expect "Wrote 23 bytes to s/startup-sequence"
expect "Added 11 bytes to s/startup-sequence"
expect "^DH0:> echo Zdravo iz skripte"
expect "^Zdravo iz skripte"
expect "^DH0:> info"
expect "EXECUTE: scripts do not run scripts"
expect "nope: no such file"
input='dir dh0:\rtype dh0:notes/hello.txt\rtype dh0:copy.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
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
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
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
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "HEAD and TAIL: exit status $status, expected 33"
expect "^uno"
expect "^tri"
expect "^cet"
if grep -q "^dva" "$log"; then fail "HEAD or TAIL shows a line it should not"; fi
# WC, on a file of two lines and three words.
input='echo uno dva > dh0:w.txt\recho tri >> dh0:w.txt\rwc dh0:w.txt\rdelete dh0:w.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "WC: exit status $status, expected 33"
expect "^2 lines, 3 words, 12 bytes"
# FIND, files by a part of their names.
input='write dh0:f/aaa.txt x\rwrite dh0:f/bbb.txt y\rfind AAA dh0:\rdelete dh0:f/aaa.txt\rdelete dh0:f/bbb.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "FIND: exit status $status, expected 33"
expect "^DH0:f/aaa.txt"
expect "^1 of [0-9]* files on DH0:"
if grep -q "^DH0:f/bbb.txt" "$log"; then fail "FIND lists a file it should not"; fi
# CMP, the same files and different ones.
input='write dh0:c1 abcd\rwrite dh0:c2 abcd\rwrite dh0:c3 abXd\rwrite dh0:c4 abc\rcmp dh0:c1 dh0:c2\rcmp dh0:c1 dh0:c3\rcmp dh0:c1 dh0:c4\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "CMP: exit status $status, expected 33"
expect "^The files are the same."
expect "^The files differ at byte 3 (line 1)."
expect "^The files differ in size: 4 and 3 bytes."
# SORT, three lines out of order.
input='echo cc > dh0:s.txt\recho aa >> dh0:s.txt\recho bb >> dh0:s.txt\rsort dh0:s.txt\rdelete dh0:s.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "SORT: exit status $status, expected 33"
expect "^aa"
[ "$(sed -n '/> sort /,/> delete /p' "$log" | grep -E '^(aa|bb|cc)$' | tr -d '\r\n')" = "aabbcc" ] || fail "SORT does not put the lines in order"
# HEXDUMP, a three-byte file.
input='write dh0:h.txt ABC\rhexdump dh0:h.txt\rdelete dh0:h.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "HEXDUMP: exit status $status, expected 33"
expect "^00000000  41 42 43 .*|ABC|"
# CALC, precedence and brackets, and a mistake.
input='calc 2+3*4\rcalc (2+3)*4-6/2\rcalc 7/0\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "CALC: exit status $status, expected 33"
expect "^14"
expect "^17"
expect "^CALC <sum>, for example"
# UNIQ, repeated lines in a row.
input='echo aa > dh0:u.txt\recho aa >> dh0:u.txt\recho bb >> dh0:u.txt\recho aa >> dh0:u.txt\runiq dh0:u.txt\rdelete dh0:u.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "UNIQ: exit status $status, expected 33"
[ "$(sed -n '/> uniq /,/> delete /p' "$log" | grep -E '^(aa|bb)$' | tr -d '\r\n')" = "aabbaa" ] || fail "UNIQ shows the wrong lines"
# NUMBER, lines with numbers.
input='echo aa > dh0:n.txt\recho bb >> dh0:n.txt\rnumber dh0:n.txt\rdelete dh0:n.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "NUMBER: exit status $status, expected 33"
expect "^   1  aa"
expect "^   2  bb"
# REPEAT, a command three times.
input='repeat 3 echo ponovo\rrepeat 0 echo no\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "REPEAT: exit status $status, expected 33"
[ "$(grep -c '^ponovo' "$log")" -eq 3 ] || fail "REPEAT does not run the command three times"
expect "^REPEAT <1 to 20>"
# TAC, lines backwards.
input='echo aa > dh0:t.txt\recho bb >> dh0:t.txt\recho cc >> dh0:t.txt\rtac dh0:t.txt\rdelete dh0:t.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "TAC: exit status $status, expected 33"
[ "$(sed -n '/> tac /,/> delete /p' "$log" | grep -E '^(aa|bb|cc)$' | tr -d '\r\n')" = "ccbbaa" ] || fail "TAC shows the lines in the wrong order"
# ALIAS, a short name with and without more words after it.
input='alias hi echo Zdravo\rhi\rhi svete\ralias\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "ALIAS: exit status $status, expected 33"
[ "$(sed -n "/> hi/,\$p" "$log" | grep -c "^Zdravo")" -ge 2 ] || fail "the alias does not run"
expect "^Zdravo svete"
expect "^hi = echo Zdravo"
# WAIT, two seconds by the clock.
input='wait 2\rwait 0\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "WAIT: exit status $status, expected 33"
expect "^Waited 2 seconds."
expect "^WAIT <1 to 60> seconds"
# DIR with a pattern.
input='write dh0:p/ab.txt 1\rwrite dh0:p/cd.dat 2\rdir dh0: p/*.TXT\rdir dh0:p/?d.*\rdelete dh0:p/ab.txt\rdelete dh0:p/cd.dat\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "DIR pattern: exit status $status, expected 33"
[ "$(sed -n '/> dir dh0: p/,/> delete /p' "$log" | grep -c '^  p/')" -eq 2 ] || fail "DIR with a pattern lists the wrong files"
expect "^1 files on DH0:"
# TOUCH, a new empty file and an old file that keeps its text.
input='touch dh0:e.txt\rwrite dh0:o.txt Zdravo\rtouch dh0:o.txt\rdir dh0: e.txt\rtype dh0:o.txt\rdelete dh0:e.txt\rdelete dh0:o.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "TOUCH: exit status $status, expected 33"
expect "^Made dh0:e.txt"
expect "^Touched dh0:o.txt"
expect "^  e.txt  *0 bytes"
expect "^Zdravo"
# REPLACE, a word changed twice in a file.
input='write dh0:r.txt mir i mir\rreplace dh0:r.txt mir svet\rtype dh0:r.txt\rreplace dh0:r.txt xyz a\rdelete dh0:r.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "REPLACE: exit status $status, expected 33"
expect "^Replaced 2 times in dh0:r.txt"
expect "^svet i svet"
expect "^dh0:r.txt: nothing to replace"
# SET, a variable used with $, an unset one, and the list.
input='set kome svete\recho Zdravo $kome$\recho [$nema]\rset\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "SET: exit status $status, expected 33"
expect "^Zdravo svete"
expect "^\\[\\]"
expect "^kome = svete"
# IF, with a file and with two words (one of them from a variable).
input='set a x\rif exists boot:README echo ima\rif not exists boot:nema echo nema\rif $a = x echo isto\rif $a = y echo NIJE\rif x\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "IF: exit status $status, expected 33"
expect "^ima"
expect "^nema"
expect "^isto"
expect "^IF \\[NOT\\] EXISTS"
if grep -q "^NIJE" "$log"; then fail "IF ran a command it should not"; fi
# FOR, a command for each word, and a wrong line.
input='for x in a b c do echo slovo $x\rfor x in a\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "FOR: exit status $status, expected 33"
expect "^slovo a"
expect "^slovo b"
expect "^slovo c"
expect "^FOR <name> IN"
# EDIT, a new file made with A and I, listed, saved and shown.
input='edit dh0:ed.txt\ra uno\ra tri\ri 2 dva\rr 3 TRI\rd 1\rl\rq\rw\rq\rtype dh0:ed.txt\rdelete dh0:ed.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "EDIT: exit status $status, expected 33"
expect "^   1  dva"
expect "^   2  TRI"
expect "^2 lines."
expect "^There are changes."
expect "^Saved 2 lines to dh0:ed.txt"
[ "$(sed -n '/> type dh0:ed.txt/,/> delete /p' "$log" | grep -E '^(dva|TRI)' | tr -d '\r\n')" = "dvaTRI" ] || fail "EDIT saved the wrong text"
# COPY with a pattern, into a folder.
input='write dh0:c/a.txt 1\rwrite dh0:c/b.txt 2\rwrite dh0:c/z.dat 3\rcopy dh0:c/*.txt dh0:bk/\rcopy dh0:c/*.txt dh0:bk\rdir dh0:bk/*\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "COPY pattern: exit status $status, expected 33"
expect "^2 files copied."
expect "^COPY with a pattern needs a place"
[ "$(sed -n '/> dir dh0:bk/,$p' "$log" | grep -c '^  bk/')" -eq 2 ] || fail "COPY with a pattern copied the wrong files"
# DELETE with a pattern.
input='write dh0:d/a.tmp 1\rwrite dh0:d/b.tmp 2\rwrite dh0:d/c.txt 3\rdelete dh0:d/*.tmp\rdir dh0:d/*\rdelete dh0:d/c.txt\rdelete boot:*\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "DELETE pattern: exit status $status, expected 33"
expect "^2 files deleted."
expect "^1 files on DH0:"
expect "boot:\\*: that volume is read only"
# MORE, a file of 25 lines: the first page, a key, the rest.
input='for x in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 do echo $x>>dh0:m.txt\rfor x in 16 17 18 19 20 21 22 23 24 25 do echo $x>>dh0:m.txt\rmore dh0:m.txt\r delete dh0:m.txt\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "MORE: exit status $status, expected 33"
expect "^20"
expect "^-- more (20 of 25 lines; Q stops) --"
expect "^25"
# TREE, two folders with a file or two each.
input='write dh0:u1/a 1\rwrite dh0:u1/b 22\rwrite dh0:u2/c 333\rtree dh0: u?/*\rdelete dh0:u*\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "TREE: exit status $status, expected 33"
expect "^DH0:u1/"
expect "^  a  *1 bytes"
expect "^  b  *2 bytes"
expect "^DH0:u2/"
expect "^  c  *3 bytes"
expect "^3 files in 2 folders on DH0:"
# MOVE with a pattern, and COPY into a folder its own pattern matches.
input='write dh0:v/a.txt 1\rwrite dh0:v/b.txt 2\rcopy dh0:v/*.txt dh0:v/bk/\rmove dh0:v/bk/*.txt dh0:w/\rmove dh0:v/*.txt dh0:w/\rtree dh0: w/*\rdelete dh0:w/*\rdelete dh0:v/*\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "MOVE: exit status $status, expected 33"
expect "^2 files copied."
expect "^2 files moved."
expect "dh0:w/a.txt: already there, left alone"
expect "^0 files moved."
expect "^2 files in 1 folders on DH0:"
# PROMPT, a changed prompt and the plain one again.
input='prompt <%%v>\recho a\rprompt\recho b\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "PROMPT: exit status $status, expected 33"
expect "^<BOOT:> echo a"
expect "^BOOT:> echo b"
# !n and !!, earlier lines run again.
input='echo uno\recho dva\r!1\r!!\r!9\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "recall: exit status $status, expected 33"
[ "$(grep -c '^uno' "$log")" -eq 2 ] || fail "!1 does not run the first line again"
[ "$(grep -c '^dva' "$log")" -eq 2 ] || fail "!! does not run the line before again"
expect "^!<number> runs a line of HISTORY"
# The clock: DATE shows the machine's time (QEMU's is the host's), and a
# file written to the disk carries it as its modification time.
input='date\rendcli\r'
boot "neo.shell" "-kernel $image -initrd $bootfs $rwdrive -device ramfb"
input=""
[ "$status" -eq 33 ] || fail "DATE: exit status $status, expected 33"
today=$(date -u +%Y-%m-%d)
expect "^Clock: $today [0-9][0-9]:[0-9][0-9]:[0-9][0-9] UTC"
expect "^$today [0-9][0-9]:[0-9][0-9]:[0-9][0-9] UTC"
tar -tvf "$rwdisk" notes/hello.txt | grep -q "$today" || fail "a written file does not carry today's date"
# Neo DOS typed on a virtio keyboard: QEMU's monitor presses the keys, the
# virtio-input driver turns them into characters.
python3 scripts/sendkeys.py "$sock" 2 'echo Typed on virtio!\rcho Strelice na virtio{sendkey home}e{sendkey end}!\recho AXB{sendkey left}{sendkey left}{sendkey delete}C\r{sendkey alt-shift}echo ;[]\r{sendkey alt-shift}{mouse_move 30 20}{mouse_move 10 5}{mouse_button 1}{mouse_button 0}bootlog fortress\rpci\rendcli\r' &
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
# PCI: AArch64 (QEMU virt) has its devices in the device tree.
expect "^No PCI bus here."
# BOOTLOG finds the kernel's banner and "Fortress is up." in its log.
expect "^2 lines with \"fortress\"."
expect "Neo DOS ended (code 0)"

echo "BOOT TEST PASSED"
