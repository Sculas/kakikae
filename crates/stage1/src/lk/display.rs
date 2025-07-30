use crate::lk::{LK_BASE, LK_SIZE};
use crate::{hooks::follow_bl_insn, pattern_match};
use core::{ffi::*, fmt::Write};

pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "70 B5 05 46 ? ? ? FC" @ 1,
    video_puts(s: *const c_char)
}, {
    "0A 4B 00 21" @ 1,
    pub video_clean_screen()
}, {
    "00 28 14 DB 0A 4B" @ 1,
    pub video_set_cursor(row: c_int, col: c_int)
}, {
    "? ? ? FE 00 21 01 38" @ 1 = follow_bl_insn,
    pub video_get_rows()
}, {
    "? ? ? FE E3 1C" @ 1 = follow_bl_insn,
    pub video_get_colums()
});

#[macro_export]
macro_rules! video_println {
    ($($arg:tt)*) => {
        $crate::lk::display::println(format_args!($($arg)*))
    };
}

#[doc(hidden)]
pub fn println(args: core::fmt::Arguments) {
    let mut buffer = heapless::String::<256>::new();
    write!(&mut buffer, "[kakikae:LK] {args}\n\0").ok();
    unsafe { video_puts(buffer.as_ptr() as _) };
}
