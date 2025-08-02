use crate::eprintln;
use core::{fmt::Write, mem::transmute};

mod ffi;
pub mod log;

const PL_BASE: usize = 0x201000;
const PL_SIZE: usize = 0x050000;

athook::pattern_patch!(install_patches; (PL_BASE, PL_SIZE) {
    "BA F1 01 0F 07 D1 DF F8 38 05" @ 1,
    "?? ?? 00 ?? ?? ?? ?? ?? ?? ??" = enable_logging,
});

pub unsafe fn install_hooks() {
    // 002229C0 LDR  R0, [SP,#0xD8+var_A8]
    // 002229C2 MOV  R1, R4
    // 002229C4 MOVW R2, #0x5920
    // 002229C8 BL   bldr_jump64 <-- this is the instruction we want to replace
    athook::install_hook_at(ffi::BLDR_JMP_HOOK_ADDR, bldr_jump64_hook as _);
}

#[inline(never)]
unsafe extern "C" fn bldr_jump64_hook(addr: u32, arg1: u32, arg2: u32) {
    eprintln!("Jumping from PL -> LK (0x{:08X})", addr);

    // Force the boot reason to BR_POWER_KEY as indicated by MTK:
    // if (mtk_detect_key(PL_PMIC_PWR_KEY) && hw_check_battery()) {
    //     print("%s Power key boot!\n", MOD);
    //     rtc_mark_bypass_pwrkey();
    //     return BR_POWER_KEY;
    // }
    eprintln!("Fixing boot reason to BR_POWER_KEY");
    core::ptr::write_volatile(ffi::BOOT_REASON, 0);
    ffi::rtc_mark_bypass_pwrkey();

    // Initialize stage 2 before jumping to LK.
    eprintln!("Initializing stage 2 from PL");
    initialize_and_jump_to_s2();

    // Continue the jump to Little Kernel (LK).
    eprintln!("Jumping to LK (0x{:08X}, 0x{:08X})", arg1, arg2);
    ffi::original_bldr_jump64(addr, arg1, arg2);
}

#[rustfmt::skip]
unsafe fn initialize_and_jump_to_s2() {
    const S2_BIN: &[u8] = include_bytes!(concat!(env!("S2_BUILD_DIR"), "/kakikae-s2.bin"));
    core::ptr::copy_nonoverlapping(S2_BIN.as_ptr(), kakikae_shared::S2_BASE_ADDR, S2_BIN.len());
    eprintln!("Jumping to S2 (0x{:08X}, {} bytes)", kakikae_shared::S2_BASE_ADDR as usize, S2_BIN.len());
    let s2_entry_point: kakikae_shared::S2_ENTRY_POINT = transmute(kakikae_shared::S2_BASE_ADDR.add(1));
    s2_entry_point(log::pl_println as _) // call the EP and pray that we survive
}
