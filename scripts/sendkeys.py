#!/usr/bin/env python3
"""Types text on QEMU's emulated PS/2 keyboard through its monitor socket.

    sendkeys.py SOCKET DELAY TEXT

Waits DELAY seconds (for the system to boot), then sends one "sendkey" per
character. Only what the US layout has on its keys is supported; "\\r" is
Enter. Text in braces is sent as a monitor command of its own, for
example "{mouse_move 30 20}" moves the mouse; "{sleep 2}" waits two
seconds instead (for a device plugged in to be set up).
"""
import socket
import sys
import time

SHIFTED = {"!": "1", "@": "2", "#": "3", "$": "4", "%": "5", "^": "6", "&": "7",
           "*": "8", "(": "9", ")": "0", "_": "minus", "+": "equal", ":": "semicolon",
           '"': "apostrophe", "<": "comma", ">": "dot", "?": "slash"}
NAMED = {" ": "spc", "\r": "ret", "\n": "ret", "-": "minus", "=": "equal", ".": "dot",
         ",": "comma", "/": "slash", ";": "semicolon", "'": "apostrophe",
         "[": "bracket_left", "]": "bracket_right", "\\": "backslash"}


def key(c):
    if c.isupper():
        return "shift-" + c.lower()
    if c in SHIFTED:
        return "shift-" + SHIFTED[c]
    return NAMED.get(c, c)


def main():
    path, delay, text = sys.argv[1], float(sys.argv[2]), sys.argv[3]
    text = text.replace("\\r", "\r")
    time.sleep(delay)
    s = socket.socket(socket.AF_UNIX)
    for _ in range(50):
        try:
            s.connect(path)
            break
        except OSError:
            time.sleep(0.1)
    i = 0
    while i < len(text):
        if text[i] == "{":
            end = text.index("}", i)
            command = text[i + 1:end]
            if command.startswith("sleep "):
                time.sleep(float(command.split()[1]))
            else:
                s.sendall((command + "\n").encode())
            i = end + 1
        else:
            s.sendall(("sendkey %s\n" % key(text[i])).encode())
            i += 1
        time.sleep(0.05)
    time.sleep(0.5)
    s.close()


main()
