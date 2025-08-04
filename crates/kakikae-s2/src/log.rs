use core::{fmt::Write, mem::transmute};

#[doc(hidden)]
pub static mut PL_PRINT: Option<kakikae_shared::PL_PRINT> = None;

pub unsafe fn init(pl_print_ptr: usize) {
    PL_PRINT = Some(transmute(pl_print_ptr));
}

pub unsafe fn switch_to_lk() {
    PL_PRINT = None;
}

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {unsafe {
        if let Some(__pl_print) = $crate::log::PL_PRINT {
            __pl_print(format_args!($($arg)*), module_path!(), line!());
        } else {
            $crate::log::lk_println(format_args!($($arg)*), module_path!(), line!());
        }
    }};
}

#[doc(hidden)]
pub fn lk_println(args: core::fmt::Arguments, module: &str, line: u32) {
    let mut buffer = heapless::String::<256>::new();
    write!(&mut buffer, "[LK][{module}:{line}] {args}\n\0",).ok();
    unsafe { crate::hooks::dprintf(&buffer) };
}
