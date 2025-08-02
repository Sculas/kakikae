pub const BACKUP_LEN: usize = 9; // code len (4) + pointer length (4) + align(1)

pub(crate) unsafe fn write_hook(target: *mut u8, hook_fn: usize) {
    const THUMB_NOP: u16 = 0xbf00; // asm: nop
    const LDR_PC_PC: [u16; 2] = [0xf8df, 0xf000]; // asm: ldr.w pc, [pc]

    let mut target = target as *mut u16;
    if target as u32 % 4 != 0 {
        unsafe {
            target.write_unaligned(THUMB_NOP);
            target = target.offset(1);
        }
    }

    // sets the thumb mode bit depending on if this stage was compiled in thumb mode
    // if this is left unset, the CPU switches to ARM mode, which results in a data abort
    let hook_fn = if cfg!(target_feature = "thumb-mode") {
        hook_fn | 1
    } else {
        hook_fn & !1
    };

    unsafe {
        (target as *mut [u16; 2]).write_unaligned(LDR_PC_PC);
        (target.offset(2) as *mut usize).write_unaligned(hook_fn);
    }
}

#[cfg(target_arch = "arm")]
pub(crate) unsafe fn clear_cache_and_flush<T>(start: *const T, end: *const T) {
    let start_addr = start as usize;
    let end_addr = end as usize;

    // Clean and invalidate data cache by MVA (Modified Virtual Address)
    let mut addr = start_addr & !31; // Align to cache line (32 bytes for Cortex-A9)
    while addr < end_addr {
        // DCCIMVAC - Clean and Invalidate data cache line by MVA to PoC
        core::arch::asm!(
            "mcr p15, 0, {addr}, c7, c14, 1",
            addr = in(reg) addr,
            options(nomem, nostack)
        );
        addr += 32; // Cortex-A9 cache line size
    }

    // Data Synchronization Barrier - ensure cache operations complete
    core::arch::asm!("dsb", options(nomem, nostack));

    // Invalidate instruction cache
    core::arch::asm!(
        "mcr p15, 0, {}, c7, c5, 0", // Invalidate entire I-cache
        in(reg) 0,
        options(nomem, nostack)
    );

    // Instruction Synchronization Barrier - flush pipeline
    core::arch::asm!("isb", options(nomem, nostack));
}

#[cfg(not(target_arch = "arm"))]
pub(crate) unsafe fn clear_cache_and_flush<T>(_start: *const T, _end: *const T) {
    unimplemented!("Unsupported architecture!");
}
