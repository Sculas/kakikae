use core::{fmt::Write, mem::transmute};

#[doc(hidden)]
pub static mut PL_PRINT: Option<kakikae_shared::PL_PRINT> = None;
#[doc(hidden)]
pub static mut IN_PL_PHASE: Option<*const bool> = None;

pub unsafe fn init(pl_print_ptr: usize, in_pl_phase: *const bool) {
    PL_PRINT = Some(transmute(pl_print_ptr));
    IN_PL_PHASE = Some(in_pl_phase);
}

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {unsafe {
        match ($crate::log::PL_PRINT, $crate::log::IN_PL_PHASE.map(|v| *v)) {
            (Some(__pl_print), Some(true)) => {
                __pl_print(format_args!($($arg)*), module_path!(), line!());
            }
            (Some(_), Some(false)) => {
                $crate::log::lk_println(format_args!($($arg)*), module_path!(), line!());
            }
            _ => {} // nothing we can do...
        }
    }};
}

#[doc(hidden)]
pub fn lk_println(args: core::fmt::Arguments, module: &str, line: u32) {
    let mut buffer = heapless::String::<256>::new();
    write!(&mut buffer, "[{module}:{line}] {args}\n\0",).ok();
    unsafe { crate::hooks::dprintf(&buffer) };
}
