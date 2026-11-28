//! Neo DOS, first steps: a command line in the spirit of AmigaDOS.
//!
//! Handles: 0 is the boot image (`BOOT:`), 1 the console keyboard and 2,
//! when there is a disk, the disk (`DH0:`). Commands are not case
//! sensitive; a file name may start with a volume, as in `DH0:bin/crash`:
//!
//! - `DIR [volume:]`   list the files
//! - `TYPE <file>`     show a file
//! - `RUN <file>`      start a program and wait for it (a program's name
//!                     alone works too: `crash`, `bin/crash`)
//! - `WRITE <file> <text>`  make a file on the disk (DH0:) with the text
//! - `COPY <from> <to>`     copy a file (to DH0:, from either volume)
//! - `DELETE <file>`        delete a file on the disk (BOOT: is read only)
//! - `ECHO <text> > <file>`, `>> <file>`  make a file, or add a line to one
//! - `EXECUTE <file>`  run the lines of a script (`;` starts a comment);
//!                     `s/startup-sequence` on DH0: runs at start
//! - `RENAME <from> <to>`  rename a file on the disk
//! - `REPEAT <n> <command>` runs a command n times (up to 20).
//! - `TOUCH <file>` makes an empty file or gives an old one a fresh date.
//! - `WAIT <seconds>` a pause of 1 to 60 seconds.
//! - `ALIAS <name> <text>` a short name for a command (8 at most).
//! - `TAC <file>` the lines of a file, last first.
//! - `NUMBER <file>` the lines of a file with their numbers.
//! - `UNIQ <file>` the lines of a file without repeats in a row.
//! - `CALC <sum>` a whole-number calculator.
//! - `HEXDUMP <file> [n]` the first bytes of a file as numbers and letters.
//! - `SORT <file>` the lines of a text file in order.
//! - `CMP <a> <b>` whether two files are the same, or where they differ.
//! - `FIND <part> [vol:]` the files with a part in their names.
//! - `WC <file>` the lines, words and bytes of a file.
//! - `DIR [vol:] [pattern]` lists files; the pattern may have `*` and `?`.
//! - `HEAD <file> [n]` / `TAIL <file> [n]` the first or last n lines (10).
//! - `SEARCH <word> [vol:]` the lines of the text files that have a word
//! - `DATE`, `TIME`    the clock (UTC); files written get its time
//! - `CD <volume:>`    change the current volume
//! - `INFO`            list the volumes
//! - `ECHO <text>`     print the text
//! - `BOOTLOG [n|word]` the kernel's boot messages, 20 lines a page, or
//!                     only the lines with a word (`BOOTLOG USB`): on a
//!                     real PC the screen is the only place to read them
//! - `USB`, `PCI`      what the machine has right now, one device a line
//!                     (for reports from real PCs)
//! - `VERSION`, `HELP`, `ENDCLI`
//!
//! A started program gets the current volume as its handle 0. If it
//! misbehaves, the system stops it and Neo DOS carries on.

#![no_std]
#![no_main]

use neo_rt::{print, println, Error, Handle, STOPPED_CODE};

const KEYBOARD: Handle = 1;
const LINE_MAX: usize = 120;

/// The volumes Neo DOS knows: name and directory handle.
const VOLUMES: [(&str, Handle); 2] = [("BOOT", 0), ("DH0", 2)];

/// Typed bytes not used yet: one read can bring more than one line.
struct Keyboard {
    buf: [u8; 16],
    at: usize,
    end: usize,
}

impl Keyboard {
    fn next(&mut self) -> Option<u8> {
        if self.at == self.end {
            self.end = neo_rt::read_input(KEYBOARD, &mut self.buf).ok()?;
            self.at = 0;
        }
        self.at += 1;
        Some(self.buf[self.at - 1])
    }
}

/// Reads one line, echoing what is typed. Left and Right move the cursor,
/// Home and End jump to the ends, Backspace and Delete remove the letter
/// before or under the cursor. Keys arrive as UTF-8 and VT100 escape
/// sequences (`ESC [ D` is Left), the same from the keyboard drivers and
/// from a serial terminal. Up and Down step through earlier lines. Returns
/// None when the input ends.
fn read_line<'a>(keys: &mut Keyboard, line: &'a mut [u8; LINE_MAX], hist: &mut History) -> Option<&'a str> {
    let mut ed = Editor { line, len: 0, cur: 0 };
    // How far back in the history the line shown is (0: the one being
    // typed, kept in `draft` while an older one is shown).
    let mut back = 0;
    let mut draft = ([0u8; LINE_MAX], 0);
    // A letter of more than one byte, collected until it is whole.
    let (mut part, mut have) = ([0u8; 4], 0);
    // Escape sequence: 0 none, 1 after ESC, 2 after ESC [, 3 after ESC [ 3.
    let mut esc = 0;
    loop {
        let b = keys.next()?;
        match (esc, b) {
            (0, 0x1B) => esc = 1,
            (1, b'[') => esc = 2,
            (2, b'3') => esc = 3,
            (2, b'D') => (esc, _) = (0, ed.left()),
            (2, b'C') => (esc, _) = (0, ed.right()),
            (2, b'H') => (esc, _) = (0, ed.home()),
            (2, b'F') => (esc, _) = (0, ed.end()),
            (3, b'~') => (esc, _) = (0, ed.delete()),
            (2, b'A') if back < hist.count => {
                if back == 0 {
                    draft.0[..ed.len].copy_from_slice(&ed.line[..ed.len]);
                    draft.1 = ed.len;
                }
                back += 1;
                (esc, _) = (0, ed.replace(hist.get(back)));
            }
            (2, b'B') if back > 0 => {
                back -= 1;
                let text = if back == 0 { &draft.0[..draft.1] } else { hist.get(back) };
                (esc, _) = (0, ed.replace(text));
            }
            (1.., _) => esc = 0, // Anything else after ESC: ignored.
            (_, b'\r' | b'\n') => {
                ed.end();
                println!();
                let len = ed.len;
                hist.add(&ed.line[..len]);
                return core::str::from_utf8(&ed.line[..len]).ok();
            }
            // End of input (Ctrl+D, or Shut down in the Neo menu): end
            // Neo DOS as ENDCLI does, whatever was typed on the line.
            (_, 0x04) => {
                println!();
                return Some("endcli");
            }
            (_, 0x08 | 0x7F) => {
                if ed.left() {
                    ed.delete();
                }
            }
            (_, 0x20..=0x7E) => ed.insert(&[b]),
            (_, 0x80..=0xF4) => {
                if b & 0xC0 != 0x80 {
                    have = 0; // A new letter begins; drop an unfinished one.
                }
                if have < 4 {
                    part[have] = b;
                    have += 1;
                }
                if core::str::from_utf8(&part[..have]).is_ok() {
                    ed.insert(&part[..have]);
                    have = 0;
                }
            }
            _ => {}
        }
    }
}

/// The line being typed and the cursor in it (byte offsets, always at the
/// start of a letter). The screen follows each change: 0x08 moves the
/// cursor left without erasing, on the serial line and in the terminal.
struct Editor<'a> {
    line: &'a mut [u8; LINE_MAX],
    len: usize,
    cur: usize,
}

/// The last lines typed, newest last, in a ring.
struct History {
    lines: [[u8; LINE_MAX]; HISTORY],
    lens: [usize; HISTORY],
    /// How many lines it holds (at most HISTORY) and where the next goes.
    count: usize,
    next: usize,
}

const HISTORY: usize = 16;

impl History {
    /// Keeps a line, unless it is empty or the same as the last one.
    fn add(&mut self, line: &[u8]) {
        if line.iter().all(|&b| b == b' ') || (self.count > 0 && self.get(1) == line) {
            return;
        }
        self.lines[self.next][..line.len()].copy_from_slice(line);
        self.lens[self.next] = line.len();
        self.next = (self.next + 1) % HISTORY;
        self.count = (self.count + 1).min(HISTORY);
    }

    /// The line `back` steps back: 1 is the last one typed.
    fn get(&self, back: usize) -> &[u8] {
        let i = (self.next + HISTORY - back) % HISTORY;
        &self.lines[i][..self.lens[i]]
    }
}

impl Editor<'_> {
    /// Shows `text` in place of the whole line, cursor at its end.
    fn replace(&mut self, text: &[u8]) {
        let old = self.text(0, self.len).chars().count();
        self.home();
        self.line[..text.len()].copy_from_slice(text);
        self.len = text.len();
        let new = self.text(0, self.len);
        print!("{}", new);
        let extra = old.saturating_sub(new.chars().count());
        for _ in 0..extra {
            print!(" ");
        }
        Self::back(extra);
        self.cur = self.len;
    }

    fn text(&self, from: usize, to: usize) -> &str {
        core::str::from_utf8(&self.line[from..to]).unwrap_or("")
    }

    fn back(n: usize) {
        for _ in 0..n {
            print!("\x08");
        }
    }

    /// Where the letter at `at` ends.
    fn next(&self, at: usize) -> usize {
        let mut i = at + 1;
        while i < self.len && self.line[i] & 0xC0 == 0x80 {
            i += 1;
        }
        i
    }

    /// Prints from the cursor to the end, `erase` blanks after it, and
    /// moves back to the cursor.
    fn redraw_tail(&self, erase: usize) {
        let tail = self.text(self.cur, self.len);
        print!("{}", tail);
        for _ in 0..erase {
            print!(" ");
        }
        Self::back(tail.chars().count() + erase);
    }

    fn insert(&mut self, bytes: &[u8]) {
        let n = bytes.len();
        if self.len + n > LINE_MAX {
            return;
        }
        self.line.copy_within(self.cur..self.len, self.cur + n);
        self.line[self.cur..self.cur + n].copy_from_slice(bytes);
        self.len += n;
        print!("{}", self.text(self.cur, self.cur + n));
        self.cur += n;
        self.redraw_tail(0);
    }

    fn left(&mut self) -> bool {
        if self.cur == 0 {
            return false;
        }
        self.cur -= 1;
        while self.cur > 0 && self.line[self.cur] & 0xC0 == 0x80 {
            self.cur -= 1;
        }
        Self::back(1);
        true
    }

    fn right(&mut self) {
        if self.cur < self.len {
            let next = self.next(self.cur);
            print!("{}", self.text(self.cur, next));
            self.cur = next;
        }
    }

    fn home(&mut self) {
        Self::back(self.text(0, self.cur).chars().count());
        self.cur = 0;
    }

    fn end(&mut self) {
        print!("{}", self.text(self.cur, self.len));
        self.cur = self.len;
    }

    fn delete(&mut self) {
        if self.cur < self.len {
            let next = self.next(self.cur);
            self.line.copy_within(next..self.len, self.cur);
            self.len -= next - self.cur;
            self.redraw_tail(1);
        }
    }
}

/// What `put` sends the system: the name and then the contents.
const PUT_BUF_LEN: usize = 48 * 1024;
static mut PUT_BUF: [u8; PUT_BUF_LEN] = [0; PUT_BUF_LEN];
/// The startup sequence on the disk, run when Neo DOS starts.
const STARTUP: &str = "s/startup-sequence";
/// Scripts (EXECUTE) and the old contents ECHO >> adds to.
const SCRIPT_LEN: usize = 4096;
static mut SCRIPT_BUF: [u8; SCRIPT_LEN] = [0; SCRIPT_LEN];
const ECHO_OLD: usize = 8192;
static mut COPY_BUF: [u8; COPY_LEN] = [0; COPY_LEN];
static mut OLD_BUF: [u8; ECHO_OLD] = [0; ECHO_OLD];
/// Largest file COPY handles.
const COPY_LEN: usize = 32 * 1024;
/// Most lines SORT sorts.
const SORT_LINES: usize = 512;
static mut SORT_INDEX: [(u16, u16); SORT_LINES] = [(0, 0); SORT_LINES];

struct Shell {
    /// Index into VOLUMES of the current volume.
    current: usize,
}

/// A volume is there when it can be listed (DH0: needs a disk).
fn present(dir: Handle) -> bool {
    let mut name = [0u8; 1];
    !matches!(neo_rt::list(dir, 0, &mut name), Err(Error::BadHandle))
}

fn close_after<T>(h: Handle, r: T) -> T {
    let _ = neo_rt::close(h);
    r
}

impl Shell {
    /// Splits `DH0:bin/crash` into the volume's handle and the path.
    fn resolve<'a>(&self, name: &'a str) -> Option<(usize, &'a str)> {
        match name.split_once(':') {
            Some((vol, path)) => {
                let v = VOLUMES.iter().position(|(n, _)| n.eq_ignore_ascii_case(vol))?;
                present(VOLUMES[v].1).then_some((v, path))
            }
            None => Some((self.current, name)),
        }
    }

    fn dir(&self, arg: &str) {
        // DIR [volume:] [pattern]: the pattern may have * and ?.
        let (first, second) = match arg.split_once(' ') {
            Some((a, b)) => (a, b.trim()),
            None => (arg, ""),
        };
        let Some((v, path)) = self.resolve(first) else {
            println!("{}: no such volume", first.trim_end_matches(':'));
            return;
        };
        let pattern = if second.is_empty() { path } else { second };
        let dir = VOLUMES[v].1;
        let mut name = [0u8; 100];
        let (mut index, mut shown_count) = (0, 0);
        while let Ok(n) = neo_rt::list(dir, index, &mut name) {
            index += 1;
            let shown = core::str::from_utf8(&name[..n.min(name.len())]).unwrap_or("?");
            if !pattern.is_empty() && !wild_match(pattern.as_bytes(), shown.as_bytes()) {
                continue;
            }
            shown_count += 1;
            let size = neo_rt::open(dir, shown).and_then(|f| close_after(f, neo_rt::file_size(f)));
            match size {
                Ok(size) => match neo_rt::file_time(dir, shown) {
                    Ok(t) if t > 0 => {
                        let (y, mo, d, h, mi, _) = from_unix(t);
                        println!("  {:<24} {:>8} bytes  {:04}-{:02}-{:02} {:02}:{:02}", shown, size, y, mo, d, h, mi)
                    }
                    _ => println!("  {:<24} {:>8} bytes", shown, size),
                },
                Err(_) => println!("  {}", shown),
            }
        }
        println!("{} files on {}:", shown_count, VOLUMES[v].0);
    }

    fn open(&self, name: &str) -> Option<Handle> {
        let (v, path) = self.resolve(name)?;
        neo_rt::open(VOLUMES[v].1, path).ok()
    }

    fn type_file(&self, name: &str) {
        let Some(f) = self.open(name) else {
            println!("{}: no such file", name);
            neo_rt::dijana_tip(neo_rt::TIP_NO_FILE);
            return;
        };
        let mut buf = [0u8; 256];
        let mut offset = 0;
        while let Ok(n @ 1..) = neo_rt::read(f, &mut buf, offset) {
            for &c in &buf[..n] {
                let c = if c == b'\n' || (0x20..0x7F).contains(&c) { c } else { b'.' };
                print!("{}", c as char);
            }
            offset += n as u64;
        }
        close_after(f, ());
    }

    /// Finds a program by its name as given, or under bin/.
    fn find_program(&self, name: &str) -> Option<Handle> {
        if let Some(f) = self.open(name) {
            return Some(f);
        }
        let (v, path) = self.resolve(name)?;
        let mut full = [0u8; LINE_MAX + 4];
        full[..4].copy_from_slice(b"bin/");
        full[4..4 + path.len()].copy_from_slice(path.as_bytes());
        let full = core::str::from_utf8(&full[..4 + path.len()]).ok()?;
        neo_rt::open(VOLUMES[v].1, full).ok()
    }

    fn run(&self, name: &str) {
        let Some(f) = self.find_program(name) else {
            println!("{}: unknown command or program", name);
            neo_rt::dijana_tip(neo_rt::TIP_UNKNOWN_COMMAND);
            return;
        };
        match neo_rt::spawn(f, VOLUMES[self.current].1) {
            Ok(id) => match neo_rt::wait(id) {
                Ok(STOPPED_CODE) => {
                    println!("{} was stopped by the system; Neo DOS carries on", name);
                    neo_rt::dijana_tip(neo_rt::TIP_PROGRAM_STOPPED);
                }
                Ok(0) => {}
                Ok(code) => {
                    println!("{} ended with code {}", name, code);
                    neo_rt::dijana_tip(neo_rt::TIP_PROGRAM_FAILED);
                }
                Err(_) => println!("{}: lost track of it", name),
            },
            Err(Error::Denied) => println!("{}: not a program", name),
            Err(e) => println!("{}: cannot start ({:?})", name, e),
        }
        close_after(f, ());
    }

    /// Why a write failed, in words.
    fn say_write_error(what: &str, e: Error) {
        match e {
            Error::Denied => println!("{}: that volume is read only (or bin/ is kept)", what),
            Error::NotFound => println!("{}: no such file", what),
            Error::BadPointer => println!("{}: bad file name", what),
            Error::Other(neo_rt::ERR_FULL) => println!("{}: no room on the disk, or too large", what),
            e => println!("{}: cannot write ({:?})", what, e),
        }
    }

    /// Writes `data` as file `name` (may start with a volume).
    fn put(&self, name: &str, data: &[u8]) -> bool {
        self.put_parts(name, &[data])
    }

    /// Writes the parts, one after the other, as file `name`.
    fn put_parts(&self, name: &str, parts: &[&[u8]]) -> bool {
        let Some((v, path)) = self.resolve(name) else {
            println!("{}: no such volume", name);
            return false;
        };
        let total: usize = parts.iter().map(|p| p.len()).sum();
        if path.len() + total > PUT_BUF_LEN {
            println!("{}: too large for Neo DOS ({} bytes at most)", name, PUT_BUF_LEN - path.len());
            return false;
        }
        // The name and then the contents, in one buffer for the system.
        // Safe: the one static buffer of this program. The parts never lie in it.
        let all = unsafe { &mut (&mut *core::ptr::addr_of_mut!(PUT_BUF))[..path.len() + total] };
        all[..path.len()].copy_from_slice(path.as_bytes());
        let mut at = path.len();
        for p in parts {
            all[at..at + p.len()].copy_from_slice(p);
            at += p.len();
        }
        match neo_rt::put(VOLUMES[v].1, all, path.len()) {
            Ok(_) => true,
            Err(e) => {
                Self::say_write_error(name, e);
                false
            }
        }
    }

    /// TOUCH <file>: makes an empty file, or writes an old one again so that
    /// its date is now (files over 32 KiB are left alone).
    fn touch(&self, arg: &str) {
        if arg.is_empty() {
            println!("TOUCH <file>");
            return;
        }
        // Safe: the one static buffer of this program.
        let data = unsafe { &mut *core::ptr::addr_of_mut!(COPY_BUF) };
        match self.read_all(arg, data) {
            None => {
                if self.put(arg, b"") {
                    println!("Made {}", arg);
                }
            }
            Some(n) if n < COPY_LEN => {
                if self.put(arg, &data[..n]) {
                    println!("Touched {}", arg);
                }
            }
            Some(_) => println!("{}: too large to touch (32 KiB at most)", arg),
        }
    }

    fn write(&self, arg: &str) {
        match arg.split_once(' ') {
            Some((name, text)) => {
                if self.put(name, text.trim().as_bytes()) {
                    println!("Wrote {} bytes to {}", text.trim().len(), name);
                }
            }
            None => println!("WRITE <file> <text>"),
        }
    }

    fn copy(&self, arg: &str) {
        match arg.split_once(' ') {
            Some((from, to)) => {
                self.copy_files(from, to.trim());
            }
            None => println!("COPY <from> <to>"),
        }
    }

    fn copy_files(&self, from: &str, to: &str) -> bool {
        let Some(f) = self.open(from) else {
            println!("{}: no such file", from);
            return false;
        };
        // Not on the stack: a script runs commands inside commands.
        // Safe: the one static buffer of this program.
        let data = unsafe { &mut *core::ptr::addr_of_mut!(COPY_BUF) };
        let mut n = 0;
        while n < COPY_LEN {
            match neo_rt::read(f, &mut data[n..], n as u64) {
                Ok(0) | Err(_) => break,
                Ok(k) => n += k,
            }
        }
        let size = neo_rt::file_size(f).unwrap_or(0);
        close_after(f, ());
        if size > n as u64 {
            println!("{}: too large for COPY ({} bytes at most)", from, COPY_LEN);
            return false;
        }
        if self.put(to, &data[..n]) {
            println!("Copied {} bytes to {}", n, to);
            return true;
        }
        false
    }

    /// Reads the file `name` into `buf` (as much as fits); None when it
    /// cannot be opened.
    fn read_all(&self, name: &str, buf: &mut [u8]) -> Option<usize> {
        let f = self.open(name)?;
        let mut got = 0;
        while got < buf.len() {
            match neo_rt::read(f, &mut buf[got..], got as u64) {
                Ok(0) | Err(_) => break,
                Ok(k) => got += k,
            }
        }
        close_after(f, ());
        Some(got)
    }

    /// Reads a text file into the shared buffer; shows an error and returns
    /// None when it is missing. A final newline is dropped.
    fn read_text(&self, name: &str) -> Option<&'static [u8]> {
        // Safe: the one static buffer of this program.
        let data = unsafe { &mut *core::ptr::addr_of_mut!(COPY_BUF) };
        let Some(got) = self.read_all(name, data) else {
            println!("{}: no such file", name);
            return None;
        };
        let mut data = &data[..got];
        if data.ends_with(b"\n") {
            data = &data[..data.len() - 1];
        }
        Some(data)
    }

    /// TAC <file>: the lines of a file, last first (the first 512 lines).
    fn tac(&self, arg: &str) {
        if arg.is_empty() {
            println!("TOUCH <file> (an empty file, or a fresh date on an old one).");
            println!("WAIT <seconds> (a pause of 1 to 60 seconds, for scripts).");
            println!("ALIAS <name> <text> (a short name for a command; ALIAS alone lists them).");
            println!("TAC <file>");
            return;
        }
        let Some(data) = self.read_text(arg) else { return };
        let index = unsafe { &mut *core::ptr::addr_of_mut!(SORT_INDEX) };
        let (mut count, mut start) = (0, 0);
        for (i, &b) in data.iter().chain(core::iter::once(&b'\n')).enumerate() {
            if b == b'\n' {
                if count == SORT_LINES {
                    println!("(only the first {} lines are shown)", SORT_LINES);
                    break;
                }
                index[count] = (start as u16, i as u16);
                count += 1;
                start = i + 1;
            }
        }
        for &(a, b) in index[..count].iter().rev() {
            show_line(&data[a as usize..b as usize]);
        }
    }

    /// NUMBER <file>: the lines of a file with their numbers.
    fn number(&self, arg: &str) {
        if arg.is_empty() {
            println!("REPEAT <n> <command> (the command n times, up to 20).");
            println!("TAC <file> (its lines, last first).");
            println!("NUMBER <file>");
            return;
        }
        let Some(data) = self.read_text(arg) else { return };
        for (i, line) in data.split(|&b| b == b'\n').enumerate() {
            print!("{:>4}  ", i + 1);
            show_line(line);
        }
    }

    /// UNIQ <file>: the lines of a file, without a line that repeats the
    /// one before it.
    fn uniq(&self, arg: &str) {
        if arg.is_empty() {
            println!("NUMBER <file> (its lines with numbers).");
            println!("UNIQ <file>");
            return;
        }
        let Some(data) = self.read_text(arg) else { return };
        let mut last: Option<&[u8]> = None;
        for line in data.split(|&b| b == b'\n') {
            if last != Some(line) {
                show_line(line);
            }
            last = Some(line);
        }
    }

    /// HEXDUMP <file> [n]: the first n bytes (64 by default, at most 512),
    /// sixteen to a row, as numbers and as letters.
    fn hexdump(&self, arg: &str) {
        let (name, count) = match arg.split_once(' ') {
            Some((f, n)) => (f, n.trim().parse::<usize>().unwrap_or(64)),
            None => (arg, 64),
        };
        if name.is_empty() {
            println!("UNIQ <file> (its lines, without repeats in a row).");
            println!("CALC <sum> (whole numbers, + - * / and brackets).");
            println!("HEXDUMP <file> [bytes]");
            return;
        }
        let mut buf = [0u8; 512];
        let want = count.min(buf.len());
        let Some(f) = self.open(name) else {
            println!("{}: no such file", name);
            return;
        };
        let got = neo_rt::read(f, &mut buf[..want], 0).unwrap_or(0);
        close_after(f, ());
        for (row, bytes) in buf[..got].chunks(16).enumerate() {
            print!("{:08x} ", row * 16);
            for i in 0..16 {
                match bytes.get(i) {
                    Some(b) => print!(" {:02x}", b),
                    None => print!("   "),
                }
            }
            print!("  |");
            for &c in bytes {
                print!("{}", if (0x20..0x7F).contains(&c) { c as char } else { '.' });
            }
            println!("|");
        }
    }

    /// SORT <file>: the lines of a text file in order (first 512 lines).
    fn sort(&self, arg: &str) {
        if arg.is_empty() {
            println!("HEXDUMP <file> [n] (bytes as numbers and letters).");
            println!("SORT <file>");
            return;
        }
        // Safe: the static buffers of this program.
        let data = unsafe { &mut *core::ptr::addr_of_mut!(COPY_BUF) };
        let index = unsafe { &mut *core::ptr::addr_of_mut!(SORT_INDEX) };
        let Some(got) = self.read_all(arg, data) else {
            println!("{}: no such file", arg);
            return;
        };
        let mut data = &data[..got];
        if data.ends_with(b"\n") {
            data = &data[..data.len() - 1];
        }
        let mut count = 0;
        let mut start = 0;
        for (i, &b) in data.iter().chain(core::iter::once(&b'\n')).enumerate() {
            if b == b'\n' {
                if count == SORT_LINES {
                    println!("(only the first {} lines are sorted)", SORT_LINES);
                    break;
                }
                index[count] = (start as u16, i as u16);
                count += 1;
                start = i + 1;
            }
        }
        let line = |e: (u16, u16)| &data[e.0 as usize..e.1 as usize];
        for i in 1..count {
            let mut j = i;
            while j > 0 && line(index[j - 1]) > line(index[j]) {
                index.swap(j - 1, j);
                j -= 1;
            }
        }
        for &e in &index[..count] {
            for &c in &line(e)[..line(e).len().min(200)] {
                print!("{}", if (0x20..0x7F).contains(&c) { c as char } else { '.' });
            }
            println!("");
        }
    }

    /// CMP <a> <b>: whether two files are the same, or where they differ.
    fn cmp(&self, arg: &str) {
        let Some((a, b)) = arg.split_once(' ') else {
            println!("SORT <file> (its lines in order).");
            println!("CMP <file> <file>");
            return;
        };
        let b = b.trim();
        // Safe: the one static buffer of this program, in two halves.
        let data = unsafe { &mut *core::ptr::addr_of_mut!(COPY_BUF) };
        let (da, db) = data.split_at_mut(COPY_LEN / 2);
        let Some(na) = self.read_all(a, da) else {
            println!("{}: no such file", a);
            return;
        };
        let Some(nb) = self.read_all(b, db) else {
            println!("{}: no such file", b);
            return;
        };
        let (da, db) = (&da[..na], &db[..nb]);
        match da.iter().zip(db.iter()).position(|(x, y)| x != y) {
            Some(at) => println!(
                "The files differ at byte {} (line {}).",
                at + 1,
                1 + da[..at].iter().filter(|&&c| c == b'\n').count()
            ),
            None if na != nb => println!("The files differ in size: {} and {} bytes.", na, nb),
            None if na == COPY_LEN / 2 => println!("The first 16 KiB are the same."),
            None => println!("The files are the same."),
        }
    }

    /// FIND <part> [volume:]: the files whose names have a word in them.
    fn find(&self, arg: &str) {
        let (word, vol) = match arg.split_once(' ') {
            Some((w, v)) => (w, v.trim()),
            None => (arg, ""),
        };
        if word.is_empty() {
            println!("FIND <part of a name> [volume:]");
            return;
        }
        let Some((v, _)) = self.resolve(vol) else {
            println!("{}: no such volume", vol.trim_end_matches(':'));
            return;
        };
        let dir = VOLUMES[v].1;
        let (mut index, mut hits) = (0, 0);
        let mut name = [0u8; 100];
        while let Ok(n) = neo_rt::list(dir, index, &mut name) {
            index += 1;
            let shown = &name[..n.min(name.len())];
            if has_word(shown, word.as_bytes()) {
                println!("{}:{}", VOLUMES[v].0, core::str::from_utf8(shown).unwrap_or("?"));
                hits += 1;
            }
        }
        println!("{} of {} files on {}:", hits, index, VOLUMES[v].0);
    }

    /// WC <file>: lines, words and bytes of a file.
    fn wc(&self, arg: &str) {
        if arg.is_empty() {
            println!("CMP <file> <file> (are two files the same?).");
            println!("FIND <part> [vol:] (files with a name part).");
            println!("WC <file>");
            return;
        }
        // Safe: the one static buffer of this program.
        let data = unsafe { &mut *core::ptr::addr_of_mut!(COPY_BUF) };
        let Some(got) = self.read_all(arg, data) else {
            println!("{}: no such file", arg);
            return;
        };
        let data = &data[..got];
        let lines = data.iter().filter(|&&b| b == b'\n').count();
        let words = data.split(|b| b.is_ascii_whitespace()).filter(|w| !w.is_empty()).count();
        let more = if got == COPY_LEN { " (only the first 32 KiB counted)" } else { "" };
        println!("{} lines, {} words, {} bytes{}", lines, words, got, more);
    }

    /// HEAD / TAIL <file> [n]: the first or last n lines (10 by default).
    fn head_tail(&self, arg: &str, tail: bool) {
        let (name, count) = match arg.split_once(' ') {
            Some((f, n)) => (f, n.trim().parse::<usize>().unwrap_or(10)),
            None => (arg, 10),
        };
        if name.is_empty() {
            println!("{} <file> [lines]", if tail { "TAIL" } else { "HEAD" });
            return;
        }
        let Some(f) = self.open(name) else {
            println!("{}: no such file", name);
            return;
        };
        // Safe: the one static buffer of this program.
        let data = unsafe { &mut *core::ptr::addr_of_mut!(COPY_BUF) };
        let mut got = 0;
        while got < COPY_LEN {
            match neo_rt::read(f, &mut data[got..], got as u64) {
                Ok(0) | Err(_) => break,
                Ok(k) => got += k,
            }
        }
        close_after(f, ());
        let mut data = &data[..got];
        if data.ends_with(b"\n") {
            data = &data[..data.len() - 1];
        }
        let total = data.split(|&b| b == b'\n').count();
        let skip = if tail { total.saturating_sub(count) } else { 0 };
        let show = if tail { total - skip } else { count };
        for line in data.split(|&b| b == b'\n').skip(skip).take(show) {
            let line = &line[..line.len().min(200)];
            for &c in line {
                print!("{}", if (0x20..0x7F).contains(&c) { c as char } else { '.' });
            }
            println!("");
        }
    }

    /// SEARCH <word> [volume:]: the lines of the text files that have the
    /// word (any case), as `file:line: text`. Programs (files with a zero
    /// byte) are skipped, and only the first 32 KiB of a file are read.
    fn search(&self, arg: &str) {
        let (word, vol) = match arg.split_once(' ') {
            Some((w, v)) => (w, v.trim()),
            None => (arg, ""),
        };
        if word.is_empty() {
            println!("SEARCH <word> [volume:]");
            return;
        }
        let Some((v, _)) = self.resolve(vol) else {
            println!("{}: no such volume", vol.trim_end_matches(':'));
            return;
        };
        let dir = VOLUMES[v].1;
        let (mut index, mut files, mut hits) = (0, 0, 0);
        let mut name = [0u8; 100];
        while let Ok(n) = neo_rt::list(dir, index, &mut name) {
            index += 1;
            let shown = core::str::from_utf8(&name[..n.min(name.len())]).unwrap_or("?");
            let Ok(f) = neo_rt::open(dir, shown) else { continue };
            // Safe: the one static buffer of this program.
            let data = unsafe { &mut *core::ptr::addr_of_mut!(COPY_BUF) };
            let mut got = 0;
            while got < COPY_LEN {
                match neo_rt::read(f, &mut data[got..], got as u64) {
                    Ok(0) | Err(_) => break,
                    Ok(k) => got += k,
                }
            }
            close_after(f, ());
            let data = &data[..got];
            if data.contains(&0) {
                continue;
            }
            let mut found = false;
            for (number, line) in data.split(|&b| b == b'\n').enumerate() {
                if has_word(line, word.as_bytes()) {
                    let line = &line[..line.len().min(100)];
                    println!("{}:{}: {}", shown, number + 1, core::str::from_utf8(line).unwrap_or("?"));
                    hits += 1;
                    found = true;
                }
            }
            files += found as usize;
        }
        println!("{} lines in {} files on {}:", hits, files, VOLUMES[v].0);
    }

    /// RENAME <from> <to>: on the disk, a copy and then a delete.
    fn rename(&self, arg: &str) {
        let Some((from, to)) = arg.split_once(' ') else {
            println!("RENAME <from> <to>");
            return;
        };
        let to = to.trim();
        let (Some((v1, p1)), Some((v2, _))) = (self.resolve(from), self.resolve(to)) else {
            println!("{}: no such volume", arg);
            return;
        };
        if v1 != v2 {
            println!("RENAME stays on one volume; use COPY and DELETE between volumes");
            return;
        }
        if self.open(to).map(|f| close_after(f, ())).is_some() {
            println!("{}: already there", to);
            return;
        }
        let Some(f) = self.open(from) else {
            println!("{}: no such file", from);
            return;
        };
        close_after(f, ());
        // COPY, then DELETE: if the copy fails the original stays.
        let before = neo_rt::volume_status(VOLUMES[v1].1).map(|r| r.0).unwrap_or(false);
        if !before {
            println!("{}: that volume is read only", from);
            return;
        }
        if self.copy_files(from, to) {
            match neo_rt::remove(VOLUMES[v1].1, p1) {
                Ok(()) => println!("Renamed {} to {}", from, to),
                Err(e) => Self::say_write_error(from, e),
            }
        }
    }

    fn delete(&self, name: &str) {
        let Some((v, path)) = self.resolve(name) else {
            println!("{}: no such volume", name);
            return;
        };
        match neo_rt::remove(VOLUMES[v].1, path) {
            Ok(()) => println!("Deleted {}", name),
            Err(e) => Self::say_write_error(name, e),
        }
    }

    /// `ECHO text`, or `ECHO text > file` (replace) and `ECHO text >> file`
    /// (add a line): the way to make a script a line at a time.
    fn echo(&self, arg: &str) {
        let Some(at) = arg.find('>') else {
            println!("{}", arg);
            return;
        };
        let (text, rest) = arg.split_at(at);
        let (append, file) = match rest.strip_prefix(">>") {
            Some(f) => (true, f),
            None => (false, &rest[1..]),
        };
        let (text, file) = (text.trim(), file.trim());
        if file.is_empty() {
            println!("ECHO <text> > <file>");
            return;
        }
        let mut old = 0;
        if append {
            if let Some(f) = self.open(file) {
                while old < ECHO_OLD {
                    // Safe: the one static buffer of this program.
                    let buf = unsafe { &mut (&mut *core::ptr::addr_of_mut!(OLD_BUF))[old..] };
                    match neo_rt::read(f, buf, old as u64) {
                        Ok(0) | Err(_) => break,
                        Ok(k) => old += k,
                    }
                }
                let size = neo_rt::file_size(f).unwrap_or(0);
                close_after(f, ());
                if size > old as u64 {
                    println!("{}: too large to add to ({} bytes at most)", file, ECHO_OLD);
                    return;
                }
            }
        }
        let old_bytes = unsafe { &(&*core::ptr::addr_of!(OLD_BUF))[..old] };
        if self.put_parts(file, &[old_bytes, text.as_bytes(), b"\n"]) {
            println!("{} {} bytes to {}", if append { "Added" } else { "Wrote" }, text.len() + 1, file);
        }
    }

    /// EXECUTE: runs the lines of a script file as commands. A line that
    /// starts with `;` is a comment; `ENDCLI` in a script ends the script.
    fn execute(&mut self, name: &str, hist: &History, depth: u8) {
        if depth > 0 {
            println!("EXECUTE: scripts do not run scripts");
            return;
        }
        let Some(f) = self.open(name) else {
            println!("{}: no such file", name);
            neo_rt::dijana_tip(neo_rt::TIP_NO_FILE);
            return;
        };
        let mut n = 0;
        while n < SCRIPT_LEN {
            // Safe: the one static buffer of this program, used by one script at a time.
            let buf = unsafe { &mut (&mut *core::ptr::addr_of_mut!(SCRIPT_BUF))[n..] };
            match neo_rt::read(f, buf, n as u64) {
                Ok(0) | Err(_) => break,
                Ok(k) => n += k,
            }
        }
        let size = neo_rt::file_size(f).unwrap_or(0);
        close_after(f, ());
        if size > n as u64 {
            println!("{}: too long for a script ({} bytes at most)", name, SCRIPT_LEN);
            return;
        }
        let script = unsafe { &(&*core::ptr::addr_of!(SCRIPT_BUF))[..n] };
        let Ok(script) = core::str::from_utf8(script) else {
            println!("{}: not text", name);
            return;
        };
        for line in script.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            println!("{}> {}", Volume(VOLUMES[self.current].0), line);
            if command(self, hist, line, 1).is_some() {
                break;
            }
        }
    }

    fn cd(&mut self, arg: &str) {
        let vol = arg.trim_end_matches(':');
        match VOLUMES.iter().position(|(n, _)| n.eq_ignore_ascii_case(vol)) {
            Some(v) if present(VOLUMES[v].1) => self.current = v,
            _ => println!("{}: no such volume", vol),
        }
    }

    fn info(&self) {
        for (name, dir) in VOLUMES {
            if !present(dir) {
                continue;
            }
            let mut count = 0;
            let mut bytes = 0;
            let mut buf = [0u8; 100];
            while let Ok(n) = neo_rt::list(dir, count, &mut buf) {
                let file = core::str::from_utf8(&buf[..n.min(buf.len())]).unwrap_or("");
                if let Ok(f) = neo_rt::open(dir, file) {
                    bytes += close_after(f, neo_rt::file_size(f)).unwrap_or(0);
                }
                count += 1;
            }
            let (writable, free) = neo_rt::volume_status(dir).unwrap_or((false, 0));
            if writable {
                println!(
                    "  {:<6} {:>3} files {:>8} bytes  (disk, read and write, {} KiB free)",
                    Volume(name),
                    count,
                    bytes,
                    free / 1024
                );
            } else {
                let kind = if dir == 0 { "boot image, read only" } else { "disk, read only" };
                println!("  {:<6} {:>3} files {:>8} bytes  ({})", Volume(name), count, bytes, kind);
            }
        }
    }
}

/// Shows a volume name with its colon ("BOOT:"), without a heap.
struct Volume<'a>(&'a str);

impl core::fmt::Display for Volume<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut buf = [0u8; 8];
        let n = self.0.len().min(7);
        buf[..n].copy_from_slice(&self.0.as_bytes()[..n]);
        buf[n] = b':';
        f.pad(core::str::from_utf8(&buf[..n + 1]).unwrap_or("?"))
    }
}

/// Lines of the boot log on one page.
const PAGE: usize = 20;

/// True if `word` is in `line`, ignoring case.
/// Whether `name` fits `pattern`, where `*` stands for any letters and `?`
/// for one letter (capital and small letters are the same).
fn wild_match(pattern: &[u8], name: &[u8]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some((b'*', rest)) => (0..=name.len()).any(|i| wild_match(rest, &name[i..])),
        Some((b'?', rest)) => !name.is_empty() && wild_match(rest, &name[1..]),
        Some((p, rest)) => name.first().is_some_and(|n| n.eq_ignore_ascii_case(p)) && wild_match(rest, &name[1..]),
    }
}

fn has_word(line: &[u8], word: &[u8]) -> bool {
    !word.is_empty() && line.windows(word.len()).any(|w| w.eq_ignore_ascii_case(word))
}

/// BOOTLOG: a page of the kernel's log (`n`, from 1), or the lines with a
/// word. Reads the log in pieces; lines longer than the buffer are cut.
fn bootlog(arg: &str) {
    let page = if arg.is_empty() { Some(1) } else { arg.parse::<usize>().ok().filter(|&p| p > 0) };
    let (from, to) = page.map_or((0, 0), |p| ((p - 1) * PAGE + 1, p * PAGE));
    let mut chunk = [0u8; 512];
    let mut line = [0u8; 160];
    let (mut len, mut lines, mut found, mut offset) = (0, 0, 0, 0u64);
    let show = |text: &[u8], number: usize, found: &mut usize| {
        let wanted = match page {
            Some(_) => (from..=to).contains(&number),
            None => has_word(text, arg.as_bytes()),
        };
        if wanted {
            *found += 1;
            println!("{}", core::str::from_utf8(text).unwrap_or("?"));
        }
    };
    loop {
        let n = match neo_rt::read_log(KEYBOARD, &mut chunk, offset) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                println!("BOOTLOG: cannot read the log ({:?})", e);
                return;
            }
        };
        offset += n as u64;
        for &b in &chunk[..n] {
            if b == b'\n' {
                lines += 1;
                show(&line[..len], lines, &mut found);
                len = 0;
            } else if len < line.len() && b != b'\r' {
                line[len] = b;
                len += 1;
            }
        }
    }
    if len > 0 {
        lines += 1;
        show(&line[..len], lines, &mut found);
    }
    match page {
        Some(p) if found > 0 => {
            print!("Lines {}-{} of {}.", from, from + found - 1, lines);
            if to < lines {
                print!(" BOOTLOG {} for more.", p + 1);
            }
            println!();
        }
        Some(_) => println!("The log has {} lines ({} pages).", lines, lines.div_ceil(PAGE)),
        None => println!("{} lines with \"{}\".", found, arg),
    }
}

/// USB or PCI: the kernel's list of what is there right now.
fn hardware(which: u64, name: &str) {
    let mut buf = [0u8; 3072];
    match neo_rt::hardware(KEYBOARD, which, &mut buf) {
        Ok(0) if which == neo_rt::HW_USB => println!("No USB devices set up."),
        Ok(0) => println!("No PCI bus here."),
        Ok(n) => {
            let shown = n.min(buf.len());
            let text = core::str::from_utf8(&buf[..shown]).unwrap_or("?");
            let lines = text.lines().count();
            for l in text.lines() {
                println!("{}", l);
            }
            if n > shown {
                println!("... and more ({} bytes not shown); BOOTLOG {} has them too.", n - shown, name);
            } else {
                println!("{} {} device{}.", lines, name, if lines == 1 { "" } else { "s" });
            }
        }
        Err(e) => println!("{}: cannot ask the kernel ({:?})", name, e),
    }
}

fn help() {
    println!("DIR [vol:], TYPE <file>, RUN <program>, CD <vol:>, INFO, ECHO <text>,");
    println!("DATE (or TIME) shows the clock, in UTC.");
    println!("WC <file> (lines, words and bytes).");
    println!("HEAD <file> [n], TAIL <file> [n] (the first or last lines of a text file).");
    println!("RENAME <from> <to>, SEARCH <word> [vol:] (the lines of the text files with a word).");
    println!("EXECUTE <script> runs commands from a file; ECHO <text> > <file> (or >>) writes one.");
    println!("WRITE <file> <text>, COPY <from> <to>, DELETE <file> (on DH0:, when it is writable),");
    println!("BOOTLOG [page|word], USB, PCI, HISTORY, VERSION, HELP, ENDCLI. A program's name");
    println!("alone runs it too.");
    println!("Arrows, Home, End and Delete edit the line; Up and Down bring back earlier ones.");
    println!("Volumes: BOOT: (the boot image) and DH0: (the disk, if there is one).");
}

/// Year, month, day, hour, minute, second of a time since 1 January 1970.
fn from_unix(t: u64) -> (u64, u64, u64, u64, u64, u64) {
    let days = (t / 86_400) as i64;
    let rem = t % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    (y as u64, m as u64, d as u64, rem / 3600, rem / 60 % 60, rem % 60)
}

/// Reads whole numbers with + - * / and brackets, left to right with the
/// usual precedence. None on a mistake or a division by zero.
struct Calc<'a> {
    text: &'a [u8],
    at: usize,
}

impl Calc<'_> {
    fn peek(&mut self) -> Option<u8> {
        while self.text.get(self.at) == Some(&b' ') {
            self.at += 1;
        }
        self.text.get(self.at).copied()
    }

    fn sum(&mut self) -> Option<i64> {
        let mut v = self.product()?;
        while let Some(op @ (b'+' | b'-')) = self.peek() {
            self.at += 1;
            let r = self.product()?;
            v = if op == b'+' { v.checked_add(r)? } else { v.checked_sub(r)? };
        }
        Some(v)
    }

    fn product(&mut self) -> Option<i64> {
        let mut v = self.atom()?;
        while let Some(op @ (b'*' | b'/')) = self.peek() {
            self.at += 1;
            let r = self.atom()?;
            v = if op == b'*' { v.checked_mul(r)? } else { v.checked_div(r)? };
        }
        Some(v)
    }

    fn atom(&mut self) -> Option<i64> {
        match self.peek()? {
            b'-' => {
                self.at += 1;
                self.atom()?.checked_neg()
            }
            b'(' => {
                self.at += 1;
                let v = self.sum()?;
                (self.peek()? == b')').then(|| self.at += 1)?;
                Some(v)
            }
            b'0'..=b'9' => {
                let mut v: i64 = 0;
                while let Some(d @ b'0'..=b'9') = self.text.get(self.at).copied() {
                    v = v.checked_mul(10)?.checked_add((d - b'0') as i64)?;
                    self.at += 1;
                }
                Some(v)
            }
            _ => None,
        }
    }
}

/// CALC <sum>: the value of a sum such as `(2+3)*4`.
fn calc(arg: &str) {
    let mut c = Calc { text: arg.as_bytes(), at: 0 };
    match c.sum() {
        Some(v) if !arg.is_empty() && c.peek().is_none() => println!("{}", v),
        _ => println!("CALC <sum>, for example CALC (2+3)*4 (whole numbers, + - * / and brackets)"),
    }
}

/// Most aliases, and the longest name and text of one.
const ALIASES: usize = 8;
const ALIAS_NAME: usize = 16;
const ALIAS_TEXT: usize = 64;
/// name, text, and the lengths of both.
#[derive(Clone, Copy)]
struct Alias {
    name: [u8; ALIAS_NAME],
    text: [u8; ALIAS_TEXT],
    name_len: u8,
    text_len: u8,
}
static mut ALIAS_TABLE: [Alias; ALIASES] =
    [Alias { name: [0; ALIAS_NAME], text: [0; ALIAS_TEXT], name_len: 0, text_len: 0 }; ALIASES];

/// ALIAS [name text...]: with nothing, lists the aliases; with a name and
/// a text, makes `name` stand for the text (the same name again changes it).
fn alias(arg: &str) {
    // Safe: only the one thread of Neo DOS uses the table.
    let table = unsafe { &mut *core::ptr::addr_of_mut!(ALIAS_TABLE) };
    if arg.is_empty() {
        let mut any = false;
        for a in table.iter().filter(|a| a.name_len > 0) {
            let (n, t) = (&a.name[..a.name_len as usize], &a.text[..a.text_len as usize]);
            println!("{} = {}", core::str::from_utf8(n).unwrap_or("?"), core::str::from_utf8(t).unwrap_or("?"));
            any = true;
        }
        if !any {
            println!("ALIAS <name> <text> makes a short name for a command.");
        }
        return;
    }
    let Some((name, text)) = arg.split_once(' ') else {
        println!("ALIAS <name> <text>");
        return;
    };
    let text = text.trim();
    if name.len() > ALIAS_NAME || text.len() > ALIAS_TEXT || text.is_empty() || name.eq_ignore_ascii_case("alias") {
        println!(
            "ALIAS: the name has up to {} letters, the text up to {}, and ALIAS itself cannot be changed.",
            ALIAS_NAME, ALIAS_TEXT
        );
        return;
    }
    let slot = table
        .iter()
        .position(|a| a.name_len > 0 && a.name[..a.name_len as usize].eq_ignore_ascii_case(name.as_bytes()))
        .or_else(|| table.iter().position(|a| a.name_len == 0));
    let Some(i) = slot else {
        println!("ALIAS: all {} places are taken.", ALIASES);
        return;
    };
    table[i].name[..name.len()].copy_from_slice(name.as_bytes());
    table[i].name_len = name.len() as u8;
    table[i].text[..text.len()].copy_from_slice(text.as_bytes());
    table[i].text_len = text.len() as u8;
}

/// Runs one command line, first replacing an alias at its start by its text.
fn command(shell: &mut Shell, hist: &History, text: &str, depth: u8) -> Option<i64> {
    let text = text.trim();
    let (cmd, rest) = match text.split_once(' ') {
        Some((c, r)) => (c, r.trim()),
        None => (text, ""),
    };
    // Safe: only the one thread of Neo DOS uses the table.
    let table = unsafe { &*core::ptr::addr_of!(ALIAS_TABLE) };
    if let Some(a) =
        table.iter().find(|a| a.name_len > 0 && a.name[..a.name_len as usize].eq_ignore_ascii_case(cmd.as_bytes()))
    {
        let mut line = [0u8; ALIAS_TEXT + 1 + LINE_MAX];
        let t = a.text_len as usize;
        line[..t].copy_from_slice(&a.text[..t]);
        let mut n = t;
        if !rest.is_empty() && n + 1 + rest.len() <= line.len() {
            line[n] = b' ';
            line[n + 1..n + 1 + rest.len()].copy_from_slice(rest.as_bytes());
            n += 1 + rest.len();
        }
        if let Ok(full) = core::str::from_utf8(&line[..n]) {
            return run_command(shell, hist, full, depth);
        }
    }
    run_command(shell, hist, text, depth)
}

/// Prints a line of a text file; unprintable bytes show as dots.
fn show_line(line: &[u8]) {
    for &c in &line[..line.len().min(200)] {
        print!("{}", if (0x20..0x7F).contains(&c) { c as char } else { '.' });
    }
    println!("");
}

/// Runs one command line. Returns the exit code when the line ends Neo DOS.
/// `depth` is 0 when typed and 1 inside a script (scripts do not nest).
fn run_command(shell: &mut Shell, hist: &History, text: &str, depth: u8) -> Option<i64> {
    let text = text.trim();
    let (cmd, arg) = match text.split_once(' ') {
        Some((c, a)) => (c, a.trim()),
        None => (text, ""),
    };
    let is = |name: &str| cmd.eq_ignore_ascii_case(name);
    if cmd.is_empty() {
    } else if is("dir") || is("ls") {
        shell.dir(arg);
    } else if is("type") || is("cat") {
        shell.type_file(arg);
    } else if is("touch") {
        shell.touch(arg);
    } else if is("write") {
        shell.write(arg);
    } else if is("copy") {
        shell.copy(arg);
    } else if is("delete") || is("del") || is("rm") {
        shell.delete(arg);
    } else if is("wait") {
        // WAIT <seconds>: pauses (1 to 60), by the clock.
        match arg.parse::<u64>() {
            Ok(n @ 1..=60) => {
                let end = neo_rt::clock() + n;
                while neo_rt::clock() < end {
                    neo_rt::sleep(10);
                }
                println!("Waited {} seconds.", n);
            }
            _ => println!("WAIT <1 to 60> seconds"),
        }
    } else if is("alias") {
        alias(arg);
    } else if is("tac") {
        shell.tac(arg);
    } else if is("number") || is("nl") {
        shell.number(arg);
    } else if is("uniq") {
        shell.uniq(arg);
    } else if is("calc") {
        calc(arg);
    } else if is("hexdump") || is("hex") {
        shell.hexdump(arg);
    } else if is("sort") {
        shell.sort(arg);
    } else if is("cmp") {
        shell.cmp(arg);
    } else if is("find") {
        shell.find(arg);
    } else if is("wc") {
        shell.wc(arg);
    } else if is("head") {
        shell.head_tail(arg, false);
    } else if is("tail") {
        shell.head_tail(arg, true);
    } else if is("search") || is("grep") {
        shell.search(arg);
    } else if is("rename") || is("ren") || is("mv") {
        shell.rename(arg);
    } else if is("repeat") {
        // REPEAT <n> <command>: the command n times (1 to 20); not inside
        // a script or another REPEAT.
        let (n, rest) = arg.split_once(' ').unwrap_or((arg, ""));
        match n.parse::<u32>() {
            Ok(n @ 1..=20) if !rest.trim().is_empty() && depth == 0 => {
                for _ in 0..n {
                    if let Some(code) = command(shell, hist, rest, 1) {
                        return Some(code);
                    }
                }
            }
            _ => println!("REPEAT <1 to 20> <command> (typed, not in a script)"),
        }
    } else if is("execute") || is("source") {
        shell.execute(arg, hist, depth);
    } else if is("run") {
        shell.run(arg);
    } else if is("cd") {
        shell.cd(arg);
    } else if is("info") {
        shell.info();
    } else if is("echo") {
        shell.echo(arg);
    } else if is("bootlog") || is("dmesg") {
        bootlog(arg);
    } else if is("usb") || is("lsusb") {
        hardware(neo_rt::HW_USB, "USB");
    } else if is("pci") || is("lspci") {
        hardware(neo_rt::HW_PCI, "PCI");
    } else if is("history") {
        for back in (1..=hist.count).rev() {
            println!("{:3}  {}", hist.count - back + 1, core::str::from_utf8(hist.get(back)).unwrap_or("?"));
        }
    } else if is("date") || is("time") {
        let (y, mo, d, h, mi, s) = from_unix(neo_rt::clock());
        println!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", y, mo, d, h, mi, s);
    } else if is("version") {
        println!("Neo DOS 0.4 on Fortress (Neo Boing Platinum OS Lite)");
    } else if is("help") {
        help();
    } else if is("endcli") || is("exit") {
        println!("Bye.");
        return Some(0);
    } else if cmd.ends_with(':') {
        shell.cd(cmd);
    } else {
        shell.run(cmd);
    }
    None
}

#[no_mangle]
fn neo_main() -> i64 {
    println!("Neo DOS 0.4. Type HELP for the commands.");
    let mut shell = Shell { current: 0 };
    let mut line = [0u8; LINE_MAX];
    let mut hist = History { lines: [[0; LINE_MAX]; HISTORY], lens: [0; HISTORY], count: 0, next: 0 };
    let mut keys = Keyboard { buf: [0; 16], at: 0, end: 0 };
    // Like AmigaDOS: the disk may hold a startup sequence.
    if present(VOLUMES[1].1) {
        if let Ok(f) = neo_rt::open(VOLUMES[1].1, STARTUP) {
            let _ = neo_rt::close(f);
            println!("Running {}:{}", VOLUMES[1].0, STARTUP);
            shell.execute("dh0:s/startup-sequence", &hist, 0);
        }
    }
    loop {
        print!("{}> ", Volume(VOLUMES[shell.current].0));
        let Some(text) = read_line(&mut keys, &mut line, &mut hist) else { return 1 };
        if let Some(code) = command(&mut shell, &hist, text, 0) {
            return code;
        }
    }
}
