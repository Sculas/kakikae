use super::{LK_BASE, LK_SIZE};
use crate::*;
use athook::follow_bl_insn;
use core::{ffi::*, fmt::Write};

pub const PHYSICAL_SCREEN_DATA: *mut ScreenData =  0x4c68e7d0 as *mut _;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ScreenData {
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u32,
    pub rotation: u32,
    pub needAlign: u32,
    pub alignWidth: u32,
    pub need180Adjust: u32,
    pub fb_size: u32,
    pub fill_dst_bits: u32,
    pub red_offset: u32,
    pub blue_offset: u32,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Rectangle {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}
athook::pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "70 B5 05 46 ? ? ? FC" @ 1,
    video_puts(s: *const c_char)
}, {
    "30 B5 05 1E" @ 1,
    fbcon_displ_string(scale: c_int, string: *const c_char)
}, {
    "? ? ? FA 80 46 16 4F" @ 1 = follow_bl_insn,
    pub get_fb_addr() -> *mut u32
}, {
    "? ? ? FA 7C 44 24 68" @ 1 = follow_bl_insn,
    pub get_temp_fb_addr() -> *mut u32
}, {
    "2D E9 F0 41 3E 22" @ 1,
    pub init_fb_screen() -> u32
}, {
    "07 4B 10 B5 7B 44" @ 1,
    pub mt_disp_update(arg1: u32, arg2: u32, arg3: u32, arg4: u32) -> u32
}, {
    "84 B0 2D E9 F0 4F 9F B0" @ 1,
    pub fill_rect_with_content_by_32bit_argb8888(fill_addr: *mut u32, rect: Rectangle, src: *const u32, screen: ScreenData, bits: u32) -> *const u32
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
}, {
    "2D E9 F0 43 91 B0 BF 4F" @ 1,
    pub mt65xx_leds_brightness_set(led_type: c_int, level: c_int)
}, {
    "ED E9 F0 41 1A 4D" @ 1,
    pub video_draw_image(addr: *const c_int, x: c_int, y: c_int, w: c_int, h: c_int, color: *const c_int)
});

#[macro_export]
macro_rules! video_scaled_println {
    ($scale:expr, $($arg:tt)*) => {
        $crate::hooks::display::video_scaled_println(format_args!($($arg)*), $scale)
    };
}
#[macro_export]
macro_rules! video_println {
    ($($arg:tt)*) => {
        $crate::hooks::display::video_println(format_args!($($arg)*), module_path!(), line!())
    };
}

#[doc(hidden)]
pub fn video_scaled_println(args: core::fmt::Arguments, scale: c_int) {
    let mut buffer = heapless::String::<256>::new();
    write!(&mut buffer, "{args}\n\0").ok();
    unsafe { fbcon_displ_string(scale, buffer.as_ptr() as _) };
}
#[doc(hidden)]
pub fn video_println(args: core::fmt::Arguments, module: &str, line: u32) {
    let mut buffer = heapless::String::<256>::new();
    write!(&mut buffer, "[{module}:{line}] {args}\n\0").ok();
    unsafe { video_puts(buffer.as_ptr() as _) };
}
