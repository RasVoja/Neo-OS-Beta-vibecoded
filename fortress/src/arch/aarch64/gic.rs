//! The interrupt controller (GICv2, as on QEMU `virt`) and the generic
//! timer. The EL1 virtual timer (interrupt 27) ticks at `HZ`.

use core::arch::asm;

pub const HZ: u32 = 100;
pub const TIMER_INTID: u32 = 27;
const SPURIOUS: u32 = 1023;
/// The PL011 UART of QEMU `virt` (shared interrupt 1).
pub const UART_INTID: u32 = 33;

/// Distributor and CPU interface of QEMU `virt`. Later these come from the
/// device tree.
const GICD: usize = 0x0800_0000;
const GICC: usize = 0x0801_0000;

const GICD_CTLR: usize = 0x000;
const GICD_ISENABLER: usize = 0x100;
const GICD_IPRIORITYR: usize = 0x400;
const GICD_ITARGETSR: usize = 0x800;
const GICC_CTLR: usize = 0x000;
const GICC_PMR: usize = 0x004;
const GICC_IAR: usize = 0x00C;
const GICC_EOIR: usize = 0x010;

fn write(base: usize, reg: usize, v: u32) {
    unsafe { ((base + reg) as *mut u32).write_volatile(v) }
}

fn read(base: usize, reg: usize) -> u32 {
    unsafe { ((base + reg) as *const u32).read_volatile() }
}

static mut TICK_COUNT: u64 = 0;

fn frequency() -> u64 {
    let f: u64;
    unsafe { asm!("mrs {}, cntfrq_el0", out(reg) f, options(nomem, nostack)) };
    f
}

fn rearm() {
    unsafe {
        asm!("msr cntv_tval_el0, {}", in(reg) *core::ptr::addr_of!(TICK_COUNT), options(nomem, nostack));
    }
}

fn set_byte(reg: usize, intid: u32, v: u32) {
    let at = reg + (intid as usize & !3);
    let shift = (intid % 4) * 8;
    write(GICD, at, read(GICD, at) & !(0xFF << shift) | (v << shift));
}

/// Enables a shared interrupt and sends it to this CPU.
pub fn enable_spi(intid: u32) {
    set_byte(GICD_IPRIORITYR, intid, 0x80);
    set_byte(GICD_ITARGETSR, intid, 1);
    write(GICD, GICD_ISENABLER + 4 * (intid as usize / 32), 1 << (intid % 32));
}

/// Starts the periodic timer; returns the timer's input frequency in Hz.
pub fn init() -> u64 {
    let freq = frequency();
    unsafe { *core::ptr::addr_of_mut!(TICK_COUNT) = freq / HZ as u64 };

    write(GICD, GICD_CTLR, 1);
    // Interrupts 0-31 are per core; byte TIMER_INTID of the priority array.
    let prio_reg = GICD_IPRIORITYR + (TIMER_INTID as usize & !3);
    let shift = (TIMER_INTID % 4) * 8;
    let prio = read(GICD, prio_reg) & !(0xFF << shift) | (0x80 << shift);
    write(GICD, prio_reg, prio);
    write(GICD, GICD_ISENABLER, 1 << TIMER_INTID);
    write(GICC, GICC_PMR, 0xF0);
    write(GICC, GICC_CTLR, 1);

    rearm();
    unsafe { asm!("msr cntv_ctl_el0, {}", in(reg) 1u64, options(nomem, nostack)) };
    freq
}

/// Handles one interrupt (called from the exception handler, IRQs masked).
pub fn handle_irq() {
    let iar = read(GICC, GICC_IAR);
    let intid = iar & 0x3FF;
    if intid == SPURIOUS {
        return;
    }
    write(GICC, GICC_EOIR, iar);
    match intid {
        TIMER_INTID => {
            rearm();
            crate::sched::timer_tick();
        }
        UART_INTID => super::uart_irq(),
        other if crate::virtio_input::irq(other) => {}
        other => crate::kprintln!("Unexpected interrupt {}", other),
    }
}
