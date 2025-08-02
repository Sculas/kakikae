use super::{LK_BASE, LK_SIZE};
use crate::*;
use super::follow_bl_insn;
use core::{ffi::*, fmt::Write};

athook::pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "70 B5 05 46 ? ? ? FC" @ 1,
    !thumb fn video_puts(s: *const c_char)
}, {
    "0A 4B 00 21" @ 1,
    pub !thumb fn video_clean_screen()
}, {
    "00 28 14 DB 0A 4B" @ 1,
    pub !thumb fn video_set_cursor(row: c_int, col: c_int)
}, {
    "? ? ? FE 00 21 01 38" @ 1 = follow_bl_insn,
    pub !thumb fn video_get_rows()
}, {
    "? ? ? FE E3 1C" @ 1 = follow_bl_insn,
    pub !thumb fn video_get_colums()
});

#[macro_export]
macro_rules! video_println {
    ($($arg:tt)*) => {
        $crate::hooks::display::video_println(format_args!($($arg)*), module_path!(), line!())
    };
}

#[doc(hidden)]
pub fn video_println(args: core::fmt::Arguments, module: &str, line: u32) {
    let mut buffer = heapless::String::<256>::new();
    write!(&mut buffer, "[{module}:{line}] {args}\n\0",).ok();
    unsafe { video_puts(buffer.as_ptr() as _) };
}
