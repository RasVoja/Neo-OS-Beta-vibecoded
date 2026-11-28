//! Self-tests for programs in user mode written in Rust. They run on every
//! architecture that has user mode.

use super::*;
use core::sync::atomic::Ordering;
use thread_tests::wait_until;

/// Runs hello-rust from `file` with one send-only handle and checks what
/// it reports and how it ends.
fn run_hello_rust(file: &[u8]) -> bool {
    use process::{Cap, LAST_EXIT, RIGHT_SEND};
    let port = ipc::Port::new(2);
    LAST_EXIT.store(i64::MIN, Ordering::Relaxed);
    match process::spawn_elf("hello-rust", file, alloc::vec![Cap::new(port.clone(), RIGHT_SEND)], false) {
        Ok(id) => {
            let mut ok = wait_until(100, || {
                port.try_recv().is_some_and(|m| m.tag == 0x200 && m.args == [12_586_269_025, 50, id])
            });
            ok &= wait_until(100, || LAST_EXIT.load(Ordering::Relaxed) == 7);
            if LAST_EXIT.load(Ordering::Relaxed) != 7 {
                kprintln!("  hello-rust exit code: {}", LAST_EXIT.load(Ordering::Relaxed));
            }
            ok
        }
        Err(e) => {
            kprintln!("  hello-rust refused: {}", e);
            false
        }
    }
}

/// Frames in use beyond what the heap grew by since `free`/`heap`.
fn leaked_since(free: usize, heap: usize) -> i64 {
    let heap_frames = (mem::heap::stats().mapped - heap) as u64 / mem::PAGE_SIZE;
    free as i64 - mem::frame::free_frames() as i64 - heap_frames as i64
}

/// The boot image: lists it and runs hello-rust from it, the way programs
/// will be found from now on.
pub fn bootfs_test() -> (bool, alloc::string::String) {
    let Some(fs) = bootfs::image() else {
        return (true, alloc::string::String::from("skipped: no boot image given"));
    };
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let mut files = 0;
    for f in fs.files() {
        kprintln!("  {} ({} bytes)", f.name(), f.data.len());
        files += 1;
    }
    let mut ok = match fs.find("bin/hello-rust") {
        Some(file) => run_hello_rust(file),
        None => {
            kprintln!("  bin/hello-rust is missing");
            false
        }
    };
    ok &= fs.find("bin/no-such-program").is_none();

    // A program that uses the file calls: it gets the image as handle 0,
    // starts bin/crash itself and exits with the number of files it saw.
    let runner = match fs.find("bin/runner") {
        Some(file) => {
            match process::spawn_elf("runner", file, alloc::vec![process::Cap::Dir(process::Volume::Boot)], false) {
                Ok(id) => process::wait(id),
                Err(e) => {
                    kprintln!("  runner refused: {}", e);
                    -1
                }
            }
        }
        None => -1,
    };
    if runner != files as i64 {
        kprintln!("  runner exit code: {}", runner);
        ok = false;
    }

    // Damaged copies are refused: a changed header byte breaks the
    // checksum, and a file size beyond the image's end is caught.
    let mut refused = 0;
    let first = |data: &[u8]| data[..512].to_vec();
    let mut bad = first(fs.data());
    bad[0] ^= 1;
    bad.extend_from_slice(&[0; 1024]);
    refused += bootfs::BootFs::new(&bad).is_err() as u32;
    let mut bad = first(fs.data());
    bad[124..136].copy_from_slice(b"77777777777\0");
    let sum: u32 = bad.iter().enumerate().map(|(i, &b)| if (148..156).contains(&i) { 32 } else { b as u32 }).sum();
    bad[148..156].copy_from_slice(alloc::format!("{:06o}\0 ", sum).as_bytes());
    refused += bootfs::BootFs::new(&bad).is_err() as u32;
    ok &= refused == 2;
    ok &= wait_until(100, || sched::thread_count() == threads);
    let leaked = leaked_since(free, heap);
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "{} files in {} KiB, hello-rust and runner ran from it, {} damaged images refused, {} frames leaked",
            files,
            fs.size() / 1024,
            refused,
            leaked
        ),
    )
}

/// Writes, rewrites and deletes files on the disk, as a program would:
/// returns what went wrong, if anything. Leaves a few hidden copies behind
/// (the archive only grows), so it runs only in the self-test.
fn disk_write_test() -> Result<(), &'static str> {
    let before = diskfs::free_bytes()?;
    diskfs::write_file("selftest/note.txt", b"first")?;
    if diskfs::read_file("selftest/note.txt")? != b"first" {
        return Err("a new file reads back wrong");
    }
    diskfs::write_file("selftest/note.txt", b"second version, longer than the first")?;
    if diskfs::read_file("selftest/note.txt")? != b"second version, longer than the first" {
        return Err("a rewritten file reads back wrong");
    }
    if diskfs::list()?.iter().filter(|e| e.name == "selftest/note.txt").count() != 1 {
        return Err("a rewritten file is listed twice");
    }
    // Several requests' worth, and a size that is not a whole sector.
    let big: alloc::vec::Vec<u8> = (0..20_000u32).map(|i| (i * 7 + i / 251) as u8).collect();
    diskfs::write_file("selftest/big.bin", &big)?;
    if diskfs::read_file("selftest/big.bin")? != big {
        return Err("a large file reads back wrong");
    }
    diskfs::delete_file("selftest/big.bin")?;
    diskfs::delete_file("selftest/note.txt")?;
    if diskfs::read_file("selftest/note.txt").is_ok() || diskfs::list()?.iter().any(|e| e.name.starts_with("selftest/"))
    {
        return Err("a deleted file is still there");
    }
    if diskfs::delete_file("selftest/note.txt").is_ok() {
        return Err("deleting a missing file worked");
    }
    // The programs are not to be replaced, and names are checked.
    if diskfs::write_file("bin/crash", b"x").is_ok() || diskfs::delete_file("bin/crash").is_ok() {
        return Err("bin/ was changed");
    }
    for bad in ["", "/abs", "a//b", "a/../b", "dir/"] {
        if diskfs::write_file(bad, b"x").is_ok() {
            return Err("a bad file name was accepted");
        }
    }
    if diskfs::free_bytes()? >= before {
        return Err("free space did not shrink");
    }
    // What the test left must not hide the real files.
    diskfs::read_file("README").map(|_| ())
}

/// The disk holds the same kind of archive as the boot image: list it,
/// compare a file with the boot image's copy, run a program read from the
/// disk, and write to it when the host allows.
pub fn disk_test() -> (bool, alloc::string::String) {
    let Some(sectors) = virtio_blk::sectors() else {
        return (true, alloc::string::String::from("skipped: no disk"));
    };
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let files = match diskfs::list() {
        Ok(list) => list,
        Err(e) => return (false, alloc::format!("cannot read the disk: {}", e)),
    };
    let mut ok = !files.is_empty();
    // The disk and the boot image hold the same programs.
    let same = match (diskfs::read_file("bin/crash"), bootfs::image().and_then(|fs| fs.find("bin/crash"))) {
        (Ok(disk), Some(mem)) => disk == mem,
        (Ok(_), None) => true,
        _ => false,
    };
    ok &= diskfs::read_file("README").is_ok_and(|r| r.starts_with(b"Neo Boing Platinum OS Lite disk"));
    ok &= same;
    ok &= diskfs::read_file("bin/no-such-program").is_err();
    ok &= virtio_blk::read(sectors, &mut [0u8; 512]).is_err();
    let ran = match diskfs::read_file("bin/hello-rust") {
        Ok(file) => run_hello_rust(&file),
        Err(_) => false,
    };
    ok &= ran;
    // Writing: refused on a read-only disk, working on a writable one.
    let writing = if virtio_blk::writable() {
        match disk_write_test() {
            Ok(()) => "written, rewritten and deleted files",
            Err(e) => {
                ok = false;
                e
            }
        }
    } else {
        ok &= diskfs::write_file("selftest.txt", b"x").is_err();
        "disk is read-only, writes refused"
    };
    ok &= wait_until(100, || sched::thread_count() == threads);
    let leaked = leaked_since(free, heap);
    ok &= leaked == 0;
    (
        ok,
        alloc::format!(
            "{} files on {} KiB, programs {}, hello-rust {}, {}, {} frames leaked",
            files.len(),
            sectors / 2,
            if same { "match the boot image" } else { "DIFFERS" },
            if ran { "ran from the disk" } else { "FAILED" },
            writing,
            leaked
        ),
    )
}

pub fn elf_test() -> (bool, alloc::string::String) {
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let file = programs::hello_rust_elf();

    // A Rust program loaded from its ELF file, with one send-only handle.
    let mut ok = run_hello_rust(file);

    // Broken or hostile files are refused before anything runs.
    let mut refused = 0;
    let mut bad = alloc::vec::Vec::from(file);
    bad[0] = 0; // magic
    refused += process::spawn_elf("bad", &bad, alloc::vec![], false).is_err() as u32;
    let mut bad = alloc::vec::Vec::from(file);
    bad[18] = 0x28; // machine: not x86_64
    refused += process::spawn_elf("bad", &bad, alloc::vec![], false).is_err() as u32;
    // A segment aimed at the kernel's memory.
    let mut bad = alloc::vec::Vec::from(file);
    let phoff = u64::from_le_bytes(bad[32..40].try_into().unwrap()) as usize;
    bad[phoff + 16..phoff + 24].copy_from_slice(&0x10_0000u64.to_le_bytes());
    refused += process::spawn_elf("bad", &bad, alloc::vec![], false).is_err() as u32;
    // Cut short in the middle of the program headers.
    refused += process::spawn_elf("bad", &file[..phoff + 20], alloc::vec![], false).is_err() as u32;
    ok &= refused == 4;

    ok &= wait_until(100, || sched::thread_count() == threads);
    let leaked = leaked_since(free, heap);
    ok &= leaked == 0;

    (ok, alloc::format!("ELF of {} bytes ran, {} bad files refused, {} frames leaked", file.len(), refused, leaked))
}

/// A program that reads kernel memory is stopped; the kernel carries on and
/// gets every frame back.
pub fn crash_test() -> (bool, alloc::string::String) {
    use process::{LAST_EXIT, STOPPED};
    let threads = sched::thread_count();
    let free = mem::frame::free_frames();
    let heap = mem::heap::stats().mapped;
    let stopped = STOPPED.load(Ordering::Relaxed);
    LAST_EXIT.store(i64::MIN, Ordering::Relaxed);

    let mut ok = process::spawn_elf("crash", programs::crash_elf(), alloc::vec![], false).is_ok();
    ok &= wait_until(100, || STOPPED.load(Ordering::Relaxed) == stopped + 1);
    ok &= wait_until(100, || sched::thread_count() == threads);
    // It was stopped, not ended by itself.
    ok &= LAST_EXIT.load(Ordering::Relaxed) == i64::MIN;

    let leaked = leaked_since(free, heap);
    ok &= leaked == 0;
    (ok, alloc::format!("1 program stopped, {} frames leaked", leaked))
}
