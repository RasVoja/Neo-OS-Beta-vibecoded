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
//! - `CD <volume:>`    change the current volume
//! - `INFO`            list the volumes
//! - `ECHO <text>`     print the text
//! - `BOOTLOG [n|word]` the kernel's boot messages, 20 lines a page, or
//!                     only the lines with a word (`BOOTLOG USB`): on a
//!                     real PC the screen is the only place to read them
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
        let Some((v, _)) = self.resolve(arg) else {
            println!("{}: no such volume", arg.trim_end_matches(':'));
            return;
        };
        let dir = VOLUMES[v].1;
        let mut name = [0u8; 100];
        let mut index = 0;
        while let Ok(n) = neo_rt::list(dir, index, &mut name) {
            let shown = core::str::from_utf8(&name[..n.min(name.len())]).unwrap_or("?");
            let size = neo_rt::open(dir, shown).and_then(|f| close_after(f, neo_rt::file_size(f)));
            match size {
                Ok(size) => println!("  {:<24} {:>8} bytes", shown, size),
                Err(_) => println!("  {}", shown),
            }
            index += 1;
        }
        println!("{} files on {}:", index, VOLUMES[v].0);
    }

    fn open(&self, name: &str) -> Option<Handle> {
        let (v, path) = self.resolve(name)?;
        neo_rt::open(VOLUMES[v].1, path).ok()
    }

    fn type_file(&self, name: &str) {
        let Some(f) = self.open(name) else {
            println!("{}: no such file", name);
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
            return;
        };
        match neo_rt::spawn(f, VOLUMES[self.current].1) {
            Ok(id) => match neo_rt::wait(id) {
                Ok(STOPPED_CODE) => println!("{} was stopped by the system; Neo DOS carries on", name),
                Ok(0) => {}
                Ok(code) => println!("{} ended with code {}", name, code),
                Err(_) => println!("{}: lost track of it", name),
            },
            Err(Error::Denied) => println!("{}: not a program", name),
            Err(e) => println!("{}: cannot start ({:?})", name, e),
        }
        close_after(f, ());
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
            let kind = if dir == 0 { "boot image, read only" } else { "disk, read only" };
            println!("  {:<6} {:>3} files {:>8} bytes  ({})", Volume(name), count, bytes, kind);
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
    let mut show = |text: &[u8], number: usize, found: &mut usize| {
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

fn help() {
    println!("DIR [vol:], TYPE <file>, RUN <program>, CD <vol:>, INFO, ECHO <text>,");
    println!("BOOTLOG [page|word], HISTORY, VERSION, HELP, ENDCLI. A program's name");
    println!("alone runs it too.");
    println!("Arrows, Home, End and Delete edit the line; Up and Down bring back earlier ones.");
    println!("Volumes: BOOT: (the boot image) and DH0: (the disk, if there is one).");
}

#[no_mangle]
fn neo_main() -> i64 {
    println!("Neo DOS 0.3. Type HELP for the commands.");
    let mut shell = Shell { current: 0 };
    let mut line = [0u8; LINE_MAX];
    let mut hist = History { lines: [[0; LINE_MAX]; HISTORY], lens: [0; HISTORY], count: 0, next: 0 };
    let mut keys = Keyboard { buf: [0; 16], at: 0, end: 0 };
    loop {
        print!("{}> ", Volume(VOLUMES[shell.current].0));
        let Some(text) = read_line(&mut keys, &mut line, &mut hist) else { return 1 };
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
        } else if is("run") {
            shell.run(arg);
        } else if is("cd") {
            shell.cd(arg);
        } else if is("info") {
            shell.info();
        } else if is("echo") {
            println!("{}", arg);
        } else if is("bootlog") || is("dmesg") {
            bootlog(arg);
        } else if is("history") {
            for back in (1..=hist.count).rev() {
                println!("{:3}  {}", hist.count - back + 1, core::str::from_utf8(hist.get(back)).unwrap_or("?"));
            }
        } else if is("version") {
            println!("Neo DOS 0.3 on Fortress (Neo Boing Platinum OS Lite)");
        } else if is("help") {
            help();
        } else if is("endcli") || is("exit") {
            println!("Bye.");
            return 0;
        } else if cmd.ends_with(':') {
            shell.cd(cmd);
        } else {
            shell.run(cmd);
        }
    }
}
