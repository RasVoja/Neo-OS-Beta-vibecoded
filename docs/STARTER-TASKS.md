# Starter tasks

*Srpski: [STARTER-TASKS.sr.md](STARTER-TASKS.sr.md)*

Twelve tasks for a first contribution. Each is small, touches one or two
places, and ends with a test that the boot scripts can check. They become
issues on the public repository the day the code moves there. Read
[CONTRIBUTING.md](../CONTRIBUTING.md) first: build, test, and the rules a
change must keep.

Sizes: **S** is an evening, **M** a weekend.

## Neo DOS

1. **`TIME` and `DATE` commands** (S). Neo DOS has no clock. Give the
   console a way to read one (the display service already reads the CMOS
   clock on a PC and the PL031 on AArch64; see `user/display`), or add a
   small system call that returns the time. *Test:* the serial session in
   both scripts types `time` and expects a line matching
   `[0-2][0-9]:[0-5][0-9]`.
2. **`MEM` command** (S). Show free memory. This needs a new system call
   (see "Adding a system call" in CONTRIBUTING) that returns the free frame
   count. *Test:* `mem` prints `Free memory: N KiB` with N above 100000,
   on both architectures.
3. **Tab completion of file names** (M). In `read_line`, Tab completes the
   word before the cursor from the current volume's files (`neo_rt::list`).
   *Test:* `type REA` followed by Tab and Enter shows the README.
4. **Ctrl+A, Ctrl+E, Ctrl+U** (S). Start of line, end of line, clear the
   line, as in most shells. `neo-keys` must turn Ctrl+letter into bytes 1
   to 26 first. *Test:* a serial session sends `\001`, `\005` and `\025`
   and checks the echoed result.
5. **`DIR` sorted by name, with sizes aligned** (S). *Test:* the first
   lines of `dir` on BOOT: come in alphabetical order.

## Keyboard and text

6. **Serbian Cyrillic** (M). Add the Cyrillic letters to the font
   (`scripts/make-font.py`, then regenerate `user/neo-gfx/src/font.rs`)
   and a third layout in `user/neo-keys`, switched with Alt+Shift in turn.
   *Test:* the PS/2 self-test types ж, and the Bar shows `СР`.
7. **Caps Lock light** (S). The PS/2 keyboard service sends command 0xED
   with the LED bits when Caps Lock changes. *Test:* a self-test reads
   back that the controller acknowledged (0xFA) both bytes.

## Desk and windows

8. **Close button** (S). The amber dot on a program's title bar sends
   `EVENT_QUIT` to that program. *Test:* in the x86 click test, a click on
   the dot ends `hello-window` with its quit code.
9. **Windows on the Bar** (M). Each program window gets a button on the
   Bar with its title; a click raises it. *Test:* with `hello-window`
   open, a pixel of its Bar button is page-white, and a click on it raises
   the window over Neo DOS.
10. **Date on the Clock tile** (S). Show the day and month under the
    time. *Test:* the display self-test checks that a pixel in the date
    row is ink-coloured.

## Kernel and tests

11. **A second test for a bad handle** (S). Through a program, call
    `send_handle` with a handle that does not exist and with one of the
    wrong kind; expect `ERR_BAD_HANDLE` and `ERR_DENIED`. *Test:* a new
    line in the Rust programs self-test.
12. **Two-way ring** (M). A pair of rings between two programs (an echo
    service): what goes in one comes back through the other. *Test:* a
    self-test sends 256 KiB, gets the same bytes back, and nothing leaks.
