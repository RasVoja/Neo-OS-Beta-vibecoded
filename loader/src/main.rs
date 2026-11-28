//! Neo loader: starts Fortress from UEFI without GRUB.
//!
//! GRUB cannot start Fortress when Secure Boot is on (its shim check only
//! understands signed PE files, and Fortress is an ELF). This program is a
//! signed PE: the shim starts it, it reads `\boot\fortress` and
//! `\boot\bootfs.tar` from the stick, builds the Multiboot2 information
//! Fortress already understands (command line, boot image, memory map,
//! framebuffer, ACPI table), leaves UEFI and jumps to Fortress's 32-bit
//! entry, as GRUB would.
#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use core::arch::global_asm;
use uefi::boot::{self, AllocateType, MemoryType};
use uefi::fs::FileSystem;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::table::cfg::{ACPI2_GUID, ACPI_GUID};
use uefi::{cstr16, println};

/// The SBAT record: the shim refuses a second stage that has none.
const SBAT_TEXT: &str = "sbat,1,SBAT Version,sbat,1,https://github.com/rhboot/shim/blob/main/SBAT.md\nneo-loader,1,Neo Boing Platinum OS Lite,neo-loader,0.0.1,https://github.com/RasVoja/Neo-OS-Beta-vibecoded\n";

const fn sbat() -> [u8; SBAT_TEXT.len() + 1] {
    let mut out = [0u8; SBAT_TEXT.len() + 1];
    let b = SBAT_TEXT.as_bytes();
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i];
        i += 1;
    }
    out
}

#[link_section = ".sbat"]
#[used]
static SBAT: [u8; SBAT_TEXT.len() + 1] = sbat();

const PAGE: usize = 4096;
/// Fortress boots with the first GiB identity mapped, so everything it is
/// given has to be in it.
const LOW: u64 = 0x3FFF_FFFF;

// 32-bit code, copied below 4 GiB and entered in compatibility mode: paging
// and long mode off, then Fortress's entry with %eax = the Multiboot2 magic,
// %ebx = the information (set by the caller), %edi = the entry.
global_asm!(
    ".section .text",
    ".code32",
    ".global neo_tramp_start",
    ".global neo_tramp_end",
    "neo_tramp_start:",
    "mov $0x10, %ax",
    "mov %ax, %ds",
    "mov %ax, %es",
    "mov %ax, %fs",
    "mov %ax, %gs",
    "mov %ax, %ss",
    "mov %cr0, %eax",
    "and $0x7FFFFFFF, %eax",
    "mov %eax, %cr0",
    "mov $0xC0000080, %ecx",
    "rdmsr",
    "and $0xFFFFFEFF, %eax",
    "wrmsr",
    "mov %cr4, %eax",
    "and $0xFFFFFFDF, %eax",
    "mov %eax, %cr4",
    "mov $0x36d76289, %eax",
    "jmp *%edi",
    "neo_tramp_end:",
    ".code64",
    options(att_syntax)
);

extern "C" {
    static neo_tramp_start: u8;
    static neo_tramp_end: u8;
}

fn pages(bytes: usize) -> usize {
    bytes.div_ceil(PAGE)
}

/// Memory for `bytes`, below 1 GiB, zeroed.
fn low_alloc(bytes: usize) -> Result<*mut u8, Status> {
    let p = boot::allocate_pages(AllocateType::MaxAddress(LOW), MemoryType::LOADER_DATA, pages(bytes))
        .map_err(|e| e.status())?;
    unsafe { core::ptr::write_bytes(p.as_ptr(), 0, pages(bytes) * PAGE) };
    Ok(p.as_ptr())
}

struct Segment {
    paddr: u64,
    offset: usize,
    filesz: usize,
    memsz: usize,
}

/// The loadable segments and the entry point of a 64-bit x86 ELF.
fn parse_elf(file: &[u8]) -> Option<(u64, Vec<Segment>)> {
    let rd16 = |o: usize| Some(u16::from_le_bytes(file.get(o..o + 2)?.try_into().ok()?) as usize);
    let rd32 = |o: usize| Some(u32::from_le_bytes(file.get(o..o + 4)?.try_into().ok()?));
    let rd64 = |o: usize| Some(u64::from_le_bytes(file.get(o..o + 8)?.try_into().ok()?));
    if file.get(0..4)? != b"\x7fELF" || file[4] != 2 {
        return None;
    }
    let entry = rd64(24)?;
    let phoff = rd64(32)? as usize;
    let phentsize = rd16(54)?;
    let phnum = rd16(56)?;
    let mut out = Vec::new();
    for i in 0..phnum {
        let p = phoff + i * phentsize;
        if rd32(p)? == 1 {
            out.push(Segment {
                paddr: rd64(p + 24)?,
                offset: rd64(p + 8)? as usize,
                filesz: rd64(p + 32)? as usize,
                memsz: rd64(p + 40)? as usize,
            });
        }
    }
    Some((entry, out))
}

/// Puts the segments where they want to be. Returns the entry point.
fn load_kernel(file: &[u8]) -> Result<u64, &'static str> {
    let (entry, segments) = parse_elf(file).ok_or("boot/fortress is not a 64-bit ELF")?;
    let start = segments.iter().map(|s| s.paddr).min().ok_or("no segments")? & !(PAGE as u64 - 1);
    let end = segments.iter().map(|s| s.paddr + s.memsz as u64).max().unwrap_or(0);
    let count = pages((end - start) as usize);
    let at = boot::allocate_pages(AllocateType::Address(start), MemoryType::LOADER_DATA, count)
        .map_err(|_| "the memory Fortress wants (1 MiB) is not free")?;
    unsafe { core::ptr::write_bytes(at.as_ptr(), 0, count * PAGE) };
    for s in &segments {
        let from = file.get(s.offset..s.offset + s.filesz).ok_or("a segment is cut off")?;
        unsafe { core::ptr::copy_nonoverlapping(from.as_ptr(), s.paddr as *mut u8, from.len()) };
    }
    Ok(entry)
}

/// The screen: Full HD if the card has it, else what it is in.
struct Screen {
    addr: u64,
    pitch: u32,
    width: u32,
    height: u32,
    shifts: (u8, u8, u8),
}

fn find_screen() -> Option<Screen> {
    let handle = boot::get_handle_for_protocol::<GraphicsOutput>().ok()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(handle).ok()?;
    let want =
        gop.modes().find(|m| m.info().resolution() == (1920, 1080) && m.info().pixel_format() != PixelFormat::BltOnly);
    if let Some(mode) = want {
        let _ = gop.set_mode(&mode);
    }
    let info = gop.current_mode_info();
    let (width, height) = info.resolution();
    let shifts = match info.pixel_format() {
        PixelFormat::Bgr => (16, 8, 0),
        PixelFormat::Rgb => (0, 8, 16),
        _ => return None,
    };
    let mut fb = gop.frame_buffer();
    Some(Screen {
        addr: fb.as_mut_ptr() as u64,
        pitch: (info.stride() * 4) as u32,
        width: width as u32,
        height: height as u32,
        shifts,
    })
}

fn rsdp() -> Option<([u8; 36], usize)> {
    uefi::system::with_config_table(|t| {
        let find = |g| t.iter().find(|e| e.guid == g).map(|e| e.address as *const u8);
        let (p, len) = match (find(ACPI2_GUID), find(ACPI_GUID)) {
            (Some(p), _) => (p, 36),
            (None, Some(p)) => (p, 20),
            _ => return None,
        };
        let mut copy = [0u8; 36];
        unsafe { core::ptr::copy_nonoverlapping(p, copy.as_mut_ptr(), len) };
        Some((copy, len))
    })
}

struct Writer {
    base: *mut u8,
    at: usize,
    cap: usize,
}

impl Writer {
    fn u8(&mut self, v: u8) {
        if self.at < self.cap {
            unsafe { self.base.add(self.at).write(v) };
        }
        self.at += 1;
    }
    fn u16(&mut self, v: u16) {
        v.to_le_bytes().iter().for_each(|&b| self.u8(b));
    }
    fn u32(&mut self, v: u32) {
        v.to_le_bytes().iter().for_each(|&b| self.u8(b));
    }
    fn u64(&mut self, v: u64) {
        v.to_le_bytes().iter().for_each(|&b| self.u8(b));
    }
    fn bytes(&mut self, b: &[u8]) {
        b.iter().for_each(|&x| self.u8(x));
    }
    fn align(&mut self) {
        while self.at % 8 != 0 {
            self.u8(0);
        }
    }
    /// A tag's header; `size` is patched by `end_tag`.
    fn tag(&mut self, kind: u32) -> usize {
        self.align();
        let at = self.at;
        self.u32(kind);
        self.u32(0);
        at
    }
    fn end_tag(&mut self, at: usize) {
        let size = (self.at - at) as u32;
        if at + 8 <= self.cap {
            unsafe { (self.base.add(at + 4) as *mut u32).write_unaligned(size) };
        }
    }
}

/// Memory type for Multiboot2 (1 usable, 2 reserved, 3 ACPI, 4 NVS).
fn mb2_kind(t: MemoryType) -> u32 {
    match t {
        MemoryType::CONVENTIONAL
        | MemoryType::BOOT_SERVICES_CODE
        | MemoryType::BOOT_SERVICES_DATA
        | MemoryType::LOADER_CODE
        | MemoryType::LOADER_DATA => 1,
        MemoryType::ACPI_RECLAIM => 3,
        MemoryType::ACPI_NON_VOLATILE => 4,
        _ => 2,
    }
}

fn menu(default: &'static str, quick: bool) -> &'static str {
    println!("Neo Boing Platinum OS Lite");
    println!("  1  desk (default)");
    println!("  2  text only (no desk, shows the boot log)");
    println!("  3  boot self-test");
    println!("Starting {} in 3 seconds; press 1, 2 or 3.", default);
    let mut chosen = default;
    if quick {
        return chosen;
    }
    'wait: for _ in 0..60 {
        let key = uefi::system::with_stdin(|i| i.read_key());
        if let Ok(Some(uefi::proto::console::text::Key::Printable(c))) = key {
            match char::from(c) {
                '1' => {
                    chosen = "neo.shell";
                    break 'wait;
                }
                '2' => {
                    chosen = "neo.text";
                    break 'wait;
                }
                '3' => {
                    chosen = "neo.test";
                    break 'wait;
                }
                _ => {}
            }
        }
        boot::stall(50_000);
    }
    chosen
}

fn run() -> Result<(), &'static str> {
    let mut fs = FileSystem::new(boot::get_image_file_system(boot::image_handle()).map_err(|_| "no file system")?);
    let kernel = fs.read(cstr16!(r"\boot\fortress")).map_err(|_| "cannot read \\boot\\fortress")?;
    let bootfs = fs.read(cstr16!(r"\boot\bootfs.tar")).map_err(|_| "cannot read \\boot\\bootfs.tar")?;
    // An optional `\boot\default.txt` names the entry started when no key is pressed (the tests use it).
    let default = fs.read(cstr16!(r"\boot\default.txt")).ok().and_then(|b| {
        let line = core::str::from_utf8(&b).ok()?.lines().next()?.trim();
        ["neo.shell", "neo.text", "neo.test"].into_iter().find(|&e| e == line)
    });
    let cmdline = menu(default.unwrap_or("neo.shell"), default.is_some());
    let screen = find_screen();
    let acpi = rsdp();
    let entry = load_kernel(&kernel)?;
    drop(kernel);

    let module = low_alloc(bootfs.len()).map_err(|_| "no memory")?;
    unsafe { core::ptr::copy_nonoverlapping(bootfs.as_ptr(), module, bootfs.len()) };
    let module_len = bootfs.len();
    drop(bootfs);
    let trampoline = low_alloc(PAGE).map_err(|_| "no memory")?;
    let info_cap = 64 * 1024;
    let info = low_alloc(info_cap).map_err(|_| "no memory")?;
    let (start, end) = (core::ptr::addr_of!(neo_tramp_start) as usize, core::ptr::addr_of!(neo_tramp_end) as usize);
    let tramp_len = end - start;
    unsafe { core::ptr::copy_nonoverlapping(start as *const u8, trampoline, tramp_len) };
    // The 32-bit GDT after the code: null, code (0x08), data (0x10); then its pointer.
    let gdt = (trampoline as usize + 256) as *mut u64;
    unsafe {
        gdt.write(0);
        gdt.add(1).write(0x00CF_9A00_0000_FFFF);
        gdt.add(2).write(0x00CF_9200_0000_FFFF);
        let ptr = (trampoline as usize + 288) as *mut u8;
        (ptr as *mut u16).write_unaligned(23);
        (ptr.add(2) as *mut u32).write_unaligned(gdt as u32);
    }
    println!("Neo loader: starting Fortress ({})", cmdline);

    // The Multiboot2 information, except the memory map, which needs the
    // final map after leaving UEFI. Its place is reserved in the buffer.
    let mut w = Writer { base: info, at: 8, cap: info_cap };
    let t = w.tag(1);
    w.bytes(cmdline.as_bytes());
    w.u8(0);
    w.end_tag(t);
    let t = w.tag(3);
    w.u32(module as u32);
    w.u32(module as u32 + module_len as u32);
    w.bytes(b"bootfs\0");
    w.end_tag(t);
    if let Some(s) = &screen {
        let t = w.tag(8);
        w.u64(s.addr);
        w.u32(s.pitch);
        w.u32(s.width);
        w.u32(s.height);
        w.u8(32);
        w.u8(1);
        w.u16(0);
        w.u8(s.shifts.0);
        w.u8(8);
        w.u8(s.shifts.1);
        w.u8(8);
        w.u8(s.shifts.2);
        w.u8(8);
        w.end_tag(t);
    }
    if let Some((table, len)) = &acpi {
        let t = w.tag(if *len == 36 { 15 } else { 14 });
        w.bytes(&table[..*len]);
        w.end_tag(t);
    }
    w.align();

    // Leave UEFI; from here nothing may allocate or print.
    let map = unsafe { boot::exit_boot_services(None) };
    let t = w.tag(6);
    w.u32(24);
    w.u32(0);
    let mut last: Option<(u64, u64, u32)> = None;
    let flush = |w: &mut Writer, e: (u64, u64, u32)| {
        w.u64(e.0);
        w.u64(e.1);
        w.u32(e.2);
        w.u32(0);
    };
    for d in map.entries() {
        let (addr, len, kind) = (d.phys_start, d.page_count * PAGE as u64, mb2_kind(d.ty));
        if len == 0 {
            continue;
        }
        match last {
            Some((a, l, k)) if k == kind && a + l == addr => last = Some((a, l + len, k)),
            Some(e) => {
                flush(&mut w, e);
                last = Some((addr, len, kind));
            }
            None => last = Some((addr, len, kind)),
        }
    }
    if let Some(e) = last {
        flush(&mut w, e);
    }
    w.end_tag(t);
    w.align();
    w.u32(0); // end tag
    w.u32(8);
    if w.at > info_cap {
        return Err("the boot information is too big");
    }
    unsafe { (info as *mut u32).write(w.at as u32) };

    unsafe {
        core::arch::asm!(
            "cli",
            "mov ebx, {info:e}",
            "lgdt [{gdt}]",
            "push 0x08",
            "push {code}",
            "retfq",
            gdt = in(reg) trampoline as usize + 288,
            code = in(reg) trampoline,
            info = in(reg) info as u32,
            in("edi") entry as u32,
            options(noreturn)
        );
    }
}

#[entry]
fn main() -> Status {
    let _ = uefi::helpers::init();
    if let Err(e) = run() {
        println!("Neo loader: {}", e);
        boot::stall(10_000_000);
        return Status::LOAD_ERROR;
    }
    Status::SUCCESS
}
