//! The PVH boot protocol's start info (Xen `hvm_start_info`, version 1).

pub const MAGIC: u32 = 0x336e_c578;
pub const E820_RAM: u32 = 1;

#[repr(C)]
pub struct StartInfo {
    magic: u32,
    version: u32,
    flags: u32,
    nr_modules: u32,
    modlist_paddr: u64,
    cmdline_paddr: u64,
    rsdp_paddr: u64,
    memmap_paddr: u64,
    memmap_entries: u32,
    reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemEntry {
    pub addr: u64,
    pub size: u64,
    pub kind: u32,
    reserved: u32,
}

impl MemEntry {
    pub fn kind_name(&self) -> &'static str {
        match self.kind {
            1 => "usable",
            2 => "reserved",
            3 => "ACPI reclaimable",
            4 => "ACPI NVS",
            5 => "bad",
            _ => "other",
        }
    }
}

/// One module the loader put in memory (QEMU: the -initrd file).
#[repr(C)]
struct ModEntry {
    paddr: u64,
    size: u64,
    cmdline_paddr: u64,
    reserved: u64,
}

pub struct BootInfo(&'static StartInfo);

impl BootInfo {
    /// # Safety
    /// `addr` must be what the PVH loader passed in %ebx, inside identity-mapped memory.
    pub unsafe fn from_addr(addr: usize) -> Option<Self> {
        if addr == 0 {
            return None;
        }
        let si = &*(addr as *const StartInfo);
        (si.magic == MAGIC).then_some(BootInfo(si))
    }

    pub fn memory_map(&self) -> &'static [MemEntry] {
        if self.0.version < 1 || self.0.memmap_paddr == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.0.memmap_paddr as *const MemEntry, self.0.memmap_entries as usize) }
    }

    /// The first module (the boot image), as a physical range.
    pub fn module(&self) -> Option<(u64, u64)> {
        if self.0.nr_modules == 0 || self.0.modlist_paddr == 0 {
            return None;
        }
        let m = unsafe { &*(self.0.modlist_paddr as *const ModEntry) };
        (m.paddr != 0 && m.size != 0).then_some((m.paddr, m.paddr + m.size))
    }

    /// Memory the loader's data occupies: start info, memory map, command
    /// line, module list, the first module and the loader's framebuffer
    /// (which UEFI may place in ordinary RAM). The frame allocator must not
    /// hand these out while they are in use. Missing ones are (0, 0).
    pub fn occupied(&self) -> [(u64, u64); 6] {
        let si = self.0 as *const StartInfo as u64;
        let map = self.0.memmap_paddr;
        let map_len = self.0.memmap_entries as u64 * core::mem::size_of::<MemEntry>() as u64;
        let cmd = self.0.cmdline_paddr;
        let mods = self.0.modlist_paddr;
        let mods_len = self.0.nr_modules as u64 * core::mem::size_of::<ModEntry>() as u64;
        [
            (si, si + core::mem::size_of::<StartInfo>() as u64),
            (map, map + map_len),
            (cmd, cmd + self.cmdline().len() as u64 + 1),
            (mods, mods + mods_len),
            self.module().unwrap_or((0, 0)),
            boot_framebuffer().map_or((0, 0), |fb| (fb.addr, fb.addr + fb.pitch as u64 * fb.height as u64)),
        ]
    }

    /// The ACPI root pointer the loader gave, or 0.
    pub fn rsdp(&self) -> u64 {
        self.0.rsdp_paddr
    }

    pub fn cmdline(&self) -> &'static str {
        let p = self.0.cmdline_paddr as *const u8;
        if p.is_null() {
            return "";
        }
        unsafe {
            let mut len = 0;
            while *p.add(len) != 0 && len < 4096 {
                len += 1;
            }
            core::str::from_utf8(core::slice::from_raw_parts(p, len)).unwrap_or("")
        }
    }
}

/// What a Multiboot2 loader (GRUB) leaves in %eax.
pub const MULTIBOOT2_MAGIC: u32 = 0x36d7_6289;

/// Room for the converted memory map; PCs report a few dozen entries at most.
const MB2_MAX_ENTRIES: usize = 128;

/// The screen a Multiboot2 loader set up (UEFI GOP or VBE), from tag 8.
#[derive(Clone, Copy, Debug)]
pub struct Framebuffer {
    pub addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    /// 1 = direct RGB colour.
    pub kind: u8,
    /// Bit positions of red, green and blue (kind 1 only).
    pub shifts: (u8, u8, u8),
}

static mut BOOT_FB: Option<Framebuffer> = None;

/// The framebuffer the boot loader set up, if it did.
pub fn boot_framebuffer() -> Option<Framebuffer> {
    unsafe { *core::ptr::addr_of!(BOOT_FB) }
}

struct Mb2Store {
    /// A copy of the RSDP from tag 14 or 15 (the loader's copy may be reused).
    rsdp: [u8; 36],
    info: StartInfo,
    map: [MemEntry; MB2_MAX_ENTRIES],
    module: ModEntry,
}

static mut MB2: Mb2Store = Mb2Store {
    rsdp: [0; 36],
    info: StartInfo {
        magic: MAGIC,
        version: 1,
        flags: 0,
        nr_modules: 0,
        modlist_paddr: 0,
        cmdline_paddr: 0,
        rsdp_paddr: 0,
        memmap_paddr: 0,
        memmap_entries: 0,
        reserved: 0,
    },
    map: [MemEntry { addr: 0, size: 0, kind: 0, reserved: 0 }; MB2_MAX_ENTRIES],
    module: ModEntry { paddr: 0, size: 0, cmdline_paddr: 0, reserved: 0 },
};

impl BootInfo {
    /// Turns a Multiboot2 information block into PVH start info, so the rest
    /// of the kernel sees one format. Takes the command line (tag 1), the
    /// first module (tag 3, the boot image), the memory map (tag 6), whose
    /// entries have the same layout as PVH's, and the ACPI RSDP (tags 14, 15). The framebuffer (tag 8) is
    /// kept for the display (`boot_framebuffer`).
    ///
    /// # Safety
    /// `addr` must be what the loader passed in %ebx, inside identity-mapped
    /// memory; call once, before anything else reads the boot info.
    pub unsafe fn from_multiboot2(addr: usize) -> Option<Self> {
        if addr == 0 || addr % 8 != 0 {
            return None;
        }
        let store = &mut *core::ptr::addr_of_mut!(MB2);
        let total = *(addr as *const u32) as usize;
        let mut off = 8;
        while off + 8 <= total {
            let tag = addr + off;
            let kind = *(tag as *const u32);
            let size = *((tag + 4) as *const u32) as usize;
            if kind == 0 || size < 8 {
                break;
            }
            match kind {
                1 => store.info.cmdline_paddr = (tag + 8) as u64,
                3 if store.info.nr_modules == 0 => {
                    let start = *((tag + 8) as *const u32) as u64;
                    let end = *((tag + 12) as *const u32) as u64;
                    store.module =
                        ModEntry { paddr: start, size: end - start, cmdline_paddr: (tag + 16) as u64, reserved: 0 };
                    store.info.modlist_paddr = core::ptr::addr_of!(store.module) as u64;
                    store.info.nr_modules = 1;
                }
                6 => {
                    let entry_size = *((tag + 8) as *const u32) as usize;
                    if entry_size >= core::mem::size_of::<MemEntry>() {
                        let mut p = tag + 16;
                        let mut n = 0;
                        while p + entry_size <= tag + size && n < MB2_MAX_ENTRIES {
                            let e = &*(p as *const MemEntry);
                            store.map[n] = MemEntry { addr: e.addr, size: e.size, kind: e.kind, reserved: 0 };
                            n += 1;
                            p += entry_size;
                        }
                        store.info.memmap_paddr = store.map.as_ptr() as u64;
                        store.info.memmap_entries = n as u32;
                    }
                }
                // ACPI RSDP, old (20 bytes) or new (36): the new one wins.
                14 | 15 if size >= 28 => {
                    let n = (size - 8).min(36);
                    if kind == 15 || store.info.rsdp_paddr == 0 {
                        store.rsdp = [0; 36];
                        core::ptr::copy_nonoverlapping((tag + 8) as *const u8, store.rsdp.as_mut_ptr(), n);
                        store.info.rsdp_paddr = store.rsdp.as_ptr() as u64;
                    }
                }
                8 if size >= 32 => {
                    let rd = |o: usize| *((tag + o) as *const u8);
                    let fb = Framebuffer {
                        addr: *((tag + 8) as *const u64),
                        pitch: *((tag + 16) as *const u32),
                        width: *((tag + 20) as *const u32),
                        height: *((tag + 24) as *const u32),
                        bpp: rd(28),
                        kind: rd(29),
                        shifts: if size >= 38 { (rd(32), rd(34), rd(36)) } else { (0, 0, 0) },
                    };
                    *core::ptr::addr_of_mut!(BOOT_FB) = Some(fb);
                }
                _ => {}
            }
            off += (size + 7) & !7;
        }
        Some(BootInfo(&store.info))
    }
}
