use crate::pl_println;
use crate::lk_println;
use core::mem::transmute;
use kakikae_shared::{PL_BASE, PL_SIZE};

mod ffi;
pub mod log;

athook::install_hooks!(install_bldr_jump_hook; (PL_BASE, PL_SIZE) {
    "2D E9 F0 4E 04 46 91 46" @ 1,
    bldr_jump64_hook(orig: _, addr: u32, arg1: u32, arg2: u32),
});

#[inline(never)]
unsafe fn bldr_jump64_hook(orig: orig_bldr_jump64_hook, addr: u32, arg1: u32, arg2: u32) {
    pl_println!("Jumping from PL -> LK ({:#010X})", addr);

    // Force the boot reason to BR_POWER_KEY as indicated by MTK:
    // if (mtk_detect_key(PL_PMIC_PWR_KEY) && hw_check_battery()) {
    //     print("%s Power key boot!\n", MOD);
    //     rtc_mark_bypass_pwrkey();
    //     return BR_POWER_KEY;
    // }
    pl_println!("Fixing boot reason to BR_POWER_KEY");
    core::ptr::write_volatile(ffi::BOOT_REASON, 0);
    ffi::rtc_mark_bypass_pwrkey();

    // Initialize stage 2 before jumping to LK.
    pl_println!("Initializing stage 2 from PL");
    initialize_and_jump_to_s2();

    // Continue the jump to Little Kernel (LK).
    pl_println!("Jumping to LK ({:#010X}, {:#010X})", arg1, arg2);
    orig(addr, arg1, arg2);
}

#[rustfmt::skip]
unsafe fn initialize_and_jump_to_s2() {
    const S2_BIN: &[u8] = include_bytes!(concat!(env!("STAGE_BUILD_DIR"), "/kakikae-s2.bin"));
    core::ptr::copy_nonoverlapping(S2_BIN.as_ptr(), kakikae_shared::S2_BASE_ADDR as _, S2_BIN.len());
    pl_println!("Jumping to S2 ({:#010X}, {} bytes)", kakikae_shared::S2_BASE_ADDR, S2_BIN.len());
    let s2_entry_point: kakikae_shared::S2_ENTRY_POINT = transmute(kakikae_shared::S2_BASE_ADDR | 1);
    s2_entry_point(log::pl_println as _) // call the entry point and pray that we survive
}
