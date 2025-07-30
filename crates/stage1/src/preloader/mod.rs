mod ffi;

use crate::hooks::install_hook_at;
use core::fmt::{self, Write};

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        $crate::preloader::println(format_args!($($arg)*))
    };
}

#[doc(hidden)]
pub fn println(args: fmt::Arguments) {
    let mut buffer = heapless::String::<256>::new();
    write!(&mut buffer, "[kakikae] {args}\n\0").ok();
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
    eprintln!("Stage 2: Jumping from Preloader -> LK (0x{:08X})", addr);

    // Force the boot reason to BR_POWER_KEY as indicated by MTK:
    // if (mtk_detect_key(PL_PMIC_PWR_KEY) && hw_check_battery()) {
    //     print("%s Power key boot!\n", MOD);
    //     rtc_mark_bypass_pwrkey();
    //     return BR_POWER_KEY;
    // }
    core::ptr::write_volatile(ffi::BOOT_REASON, 0);
    ffi::rtc_mark_bypass_pwrkey();

    // Install the LK hooks before jumping to LK.
    crate::lk::lk_install_hooks();

    // Continue the jump to Little Kernel (LK).
    ffi::original_bldr_jump64(addr, arg1, arg2);
}
