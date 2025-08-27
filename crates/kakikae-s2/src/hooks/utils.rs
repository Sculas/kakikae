use super::{LK_BASE, LK_SIZE};
use crate::*;
use athook::follow_bl_insn;
use core::ffi::*;

athook::pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "? ? ? FE 38 46 21 46" @ 1 = follow_bl_insn,
    pub mdelay(n: c_ulong)
}, {
    "10 B5 04 46 ? ? ? FF A0 42" @ 1,
    pub get_timer(base: c_ulong) -> c_ulong
}, {
    "0F B4 3A 4B" @ 1,
    pub dprintf(fmt: *const c_char) -> c_int = ...
}, {
    "? ? ? FD 04 46 B8 B1" @ 1 = follow_bl_insn,
    pub malloc(size: usize) -> *mut u8
}, {
    "? ? ? FD 28 46 38 BD" @ 1 = follow_bl_insn,
    pub free(data: *mut u8)
});