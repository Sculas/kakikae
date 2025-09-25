use super::ffi;
use core::{fmt::Write, mem::transmute};
use kakikae_shared::{PL_BASE, PL_SIZE};


#[macro_export]
macro_rules! lk_println {
    ($($arg:tt)*) => {
        pl_println!($($arg)*)
    };
}

#[macro_export]
macro_rules! pl_println {
    ($($arg:tt)*) => {
        $crate::preloader::log::pl_println(format_args!($($arg)*), module_path!(), line!())
    };
}


athook::pattern_patch!(install_handshake_patch; (PL_BASE, PL_SIZE) {
    "BA F1 01 0F 07 D1 DF F8 38 05" @ 1,
    "?? ?? 00 ?? ?? ?? ?? ?? ?? ??" = enable_logging,
});

const _: kakikae_shared::PL_PRINT = pl_println;

#[doc(hidden)]
pub fn pl_println(args: core::fmt::Arguments, module: &str, line: u32) {
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
