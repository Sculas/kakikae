use core::{fmt::Write, mem::transmute};

mod ffi;

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        $crate::preloader::pl_println(format_args!($($arg)*), module_path!(), line!())
    };
}

static mut IN_PL_PHASE: bool = false;
const _: kakikae_shared::PL_PRINT = pl_println;

#[doc(hidden)]
pub fn pl_println(args: core::fmt::Arguments, module: &str, line: u32) {
    if unsafe { !IN_PL_PHASE } {
        return;
    }

    let mut buffer = heapless::String::<256>::new();
    let mut tm = ffi::RtcTime::default();
    ffi::rtc_get_time(&mut tm);
    write!(
        &mut buffer,
        "[{}/{:02}/{:02} {:02}:{:02}:{:02}][{module}:{line}] {args}\n\0",
        tm.year, tm.mon, tm.day, tm.hour, tm.min, tm.sec
    )
    .ok();

    let printf: ffi::PrintfFn = unsafe { transmute(ffi::PRINTF_PTR as usize) };
    printf(buffer.as_ptr() as _);
}

pub unsafe fn install_preloader_bldr_jump64_hook() {
    // 002229C0 LDR  R0, [SP,#0xD8+var_A8]
    // 002229C2 MOV  R1, R4
    // 002229C4 MOVW R2, #0x5920
    // 002229C8 BL   bldr_jump64 <-- this is the instruction we want to replace
    athook::install_hook_at(ffi::BLDR_JMP_HOOK_ADDR, bldr_jump64_hook as _);
}

#[inline(never)]
unsafe extern "C" fn bldr_jump64_hook(addr: u32, arg1: u32, arg2: u32) {
    IN_PL_PHASE = true;
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
    IN_PL_PHASE = false; // leaving PL, so tell S2 to switch to LK
    ffi::original_bldr_jump64(addr, arg1, arg2);
}

#[rustfmt::skip]
unsafe fn initialize_and_jump_to_s2() {
    const S2_BIN: &[u8] = include_bytes!(concat!(env!("S2_BUILD_DIR"), "/kakikae-s2.bin"));
    core::ptr::copy_nonoverlapping(S2_BIN.as_ptr(), kakikae_shared::S2_BASE_ADDR, S2_BIN.len());
    eprintln!("Jumping to S2 (0x{:08X}, {} bytes)", kakikae_shared::S2_BASE_ADDR as usize, S2_BIN.len());
    let s2_entry_point: kakikae_shared::S2_ENTRY_POINT = transmute(kakikae_shared::S2_BASE_ADDR);
    s2_entry_point(pl_println as _, &raw const IN_PL_PHASE) // call the EP and pray that we survive
}
