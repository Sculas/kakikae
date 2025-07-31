mod ffi;

use crate::hooks::install_hook_at;
use core::fmt::{self, Write};

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        $crate::preloader::println(format_args!($($arg)*), module_path!(), line!())
    };
}

static mut PASSED_PL_STAGE: bool = false;

#[doc(hidden)]
#[rustfmt::skip]
pub fn println(args: fmt::Arguments, module: &str, line: u32) {
    if unsafe { !PASSED_PL_STAGE } { return; }
    let mut buffer = heapless::String::<256>::new();
    let mut tm = ffi::RtcTime::default();
    ffi::rtc_get_time(&mut tm);
    write!(
        &mut buffer,
        "[{}/{}/{} {}:{}:{}][{module}:{line}] {args}\n\0",
        tm.year, tm.mon, tm.day, tm.hour, tm.min, tm.sec
    ).ok();
    crate::ffi_internal_printf!()(buffer.as_ptr() as _);
}

pub unsafe fn install_preloader_bldr_jump64_hook() {
    // 002229C0 LDR  R0, [SP,#0xD8+var_A8]
    // 002229C2 MOV  R1, R4
    // 002229C4 MOVW R2, #0x5920
    // 002229C8 BL   bldr_jump64 <-- this is the instruction we want to replace
    install_hook_at(ffi::BLDR_JMP_HOOK_ADDR, bldr_jump64_hook as _);
}

#[inline(never)]
unsafe extern "C" fn bldr_jump64_hook(addr: u32, arg1: u32, arg2: u32) {
    PASSED_PL_STAGE = true;
    eprintln!("Stage 2: Jumping from Preloader -> LK (0x{:08X})", addr);

    // Force the boot reason to BR_POWER_KEY as indicated by MTK:
    // if (mtk_detect_key(PL_PMIC_PWR_KEY) && hw_check_battery()) {
    //     print("%s Power key boot!\n", MOD);
    //     rtc_mark_bypass_pwrkey();
    //     return BR_POWER_KEY;
    // }
    eprintln!("Fixing boot reason to BR_POWER_KEY");
    core::ptr::write_volatile(ffi::BOOT_REASON, 0);
    ffi::rtc_mark_bypass_pwrkey();

    // Install the LK hooks before jumping to LK.
    eprintln!("Installing LK hooks");
    crate::lk::lk_install_hooks();

    // Continue the jump to Little Kernel (LK).
    eprintln!("Jumping to LK (0x{:08X}, 0x{:08X})", arg1, arg2);
    ffi::original_bldr_jump64(addr, arg1, arg2);
}
