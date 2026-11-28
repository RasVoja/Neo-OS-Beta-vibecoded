//! The wall clock: seconds since 1 January 1970 (UTC), from the chip the
//! machine has: the CMOS clock on a PC, the PL031 on AArch64 (QEMU `virt`).
//! Used for the time of files written to the disk and for Neo DOS's DATE.

#[cfg(target_arch = "x86_64")]
/// Days from 1970-01-01 to a date in the proleptic Gregorian calendar.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(target_arch = "x86_64")]
/// Seconds since the epoch for a date and time (UTC).
pub fn to_unix(year: i64, month: i64, day: i64, hour: i64, minute: i64, second: i64) -> u64 {
    let secs = days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second;
    secs.max(0) as u64
}

/// Year, month, day, hour, minute, second of a time since the epoch.
pub fn from_unix(t: u64) -> (u64, u64, u64, u64, u64, u64) {
    let days = (t / 86_400) as i64;
    let rem = t % 86_400;
    // Civil from days (Howard Hinnant's algorithm).
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

#[cfg(target_arch = "x86_64")]
fn read_chip() -> u64 {
    use crate::arch::{inb, outb};
    let cmos = |reg: u8| unsafe {
        outb(0x70, reg);
        inb(0x71)
    };
    let snapshot = || {
        // Wait while the chip is updating, so the values belong together.
        for _ in 0..1_000_000 {
            if cmos(0x0A) & 0x80 == 0 {
                break;
            }
        }
        let b = cmos(0x0B);
        let bcd = b & 0x04 == 0;
        let fix = |v: u8| if bcd { (v & 0x0F) + (v >> 4) * 10 } else { v };
        let (s, mi, mut h) = (fix(cmos(0x00)), fix(cmos(0x02)), cmos(0x04));
        let pm = h & 0x80 != 0;
        h = fix(h & 0x7F);
        if b & 0x02 == 0 && pm {
            h = (h % 12) + 12;
        }
        let (d, mo, y) = (fix(cmos(0x07)), fix(cmos(0x08)), fix(cmos(0x09)));
        (s, mi, h, d, mo, y)
    };
    // The same twice in a row: nothing changed under our hands.
    let mut now = snapshot();
    for _ in 0..4 {
        let again = snapshot();
        if again == now {
            break;
        }
        now = again;
    }
    let (s, mi, h, d, mo, y) = now;
    to_unix(2000 + y as i64, mo as i64, d as i64, h as i64, mi as i64, s as i64)
}

#[cfg(target_arch = "aarch64")]
fn read_chip() -> u64 {
    // The PL031's data register counts seconds since the epoch.
    unsafe { core::ptr::read_volatile(0x0901_0000 as *const u32) as u64 }
}

/// Seconds since 1 January 1970 (UTC), as the machine's clock has them.
pub fn unix_seconds() -> u64 {
    read_chip()
}

/// The time as text for the log: 2026-11-28 05:14:02 UTC.
pub fn describe() -> alloc::string::String {
    let (y, mo, d, h, mi, s) = from_unix(unix_seconds());
    alloc::format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", y, mo, d, h, mi, s)
}
