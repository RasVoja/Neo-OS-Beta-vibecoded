//! Uses the file system calls the way a shell will: it gets the boot image
//! as handle 0, lists it, reads README, starts bin/crash and waits for it,
//! and checks that what it may not do is refused. Exits with the number of
//! files it found, or 100 and up on a failed check.

#![no_std]
#![no_main]

use neo_rt::{println, Error, NO_HANDLE, STOPPED_CODE};

const DIR: u64 = 0;

#[no_mangle]
fn neo_main() -> i64 {
    // List the image.
    let mut files = 0;
    let mut name = [0u8; 64];
    while let Ok(n) = neo_rt::list(DIR, files, &mut name) {
        let shown = core::str::from_utf8(&name[..n.min(name.len())]).unwrap_or("?");
        println!("runner: {}", shown);
        files += 1;
    }

    // Read a file, in two pieces.
    let Ok(readme) = neo_rt::open(DIR, "README") else { return 101 };
    let Ok(size) = neo_rt::file_size(readme) else { return 102 };
    let mut text = [0u8; 128];
    let Ok(a) = neo_rt::read(readme, &mut text[..5], 0) else { return 103 };
    let Ok(b) = neo_rt::read(readme, &mut text[5..], 5) else { return 103 };
    if (a + b) as u64 != size {
        return 104;
    }
    println!("runner: README says: {}", core::str::from_utf8(&text[..a + b]).unwrap_or("?").trim_end());

    // Start a program and wait for it: bin/crash is stopped by the kernel.
    let Ok(crash) = neo_rt::open(DIR, "bin/crash") else { return 105 };
    let Ok(id) = neo_rt::spawn(crash, NO_HANDLE) else { return 106 };
    if neo_rt::wait(id) != Ok(STOPPED_CODE) {
        return 107;
    }
    println!("runner: bin/crash was stopped, as it should be");

    // What may not be done is refused.
    if neo_rt::open(DIR, "bin/no-such-program") != Err(Error::NotFound) {
        return 108;
    }
    if neo_rt::open(readme, "README") != Err(Error::Denied) {
        return 109; // a file is not a directory
    }
    if neo_rt::read(DIR, &mut text, 0) != Err(Error::Denied) {
        return 110; // a directory cannot be read as a file
    }
    if neo_rt::wait(1) != Err(Error::NotFound) {
        return 111; // not our program
    }
    let kernel = unsafe { core::slice::from_raw_parts_mut(0x40_0000 as *mut u8, 16) };
    if neo_rt::read(readme, kernel, 0) != Err(Error::BadPointer) {
        return 112;
    }
    files as i64
}
