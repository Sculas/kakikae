use crate::lk::{LK_BASE, LK_SIZE};
use crate::{hooks::follow_bl_insn, pattern_match};
use core::ffi::*;

pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "? ? ? FF 22 68" @ 1 = follow_bl_insn,
    pub mdelay(n: c_ulong)
}, {
    "10 B5 04 46 ? ? ? FF A0 42" @ 1,
    pub get_timer(base: c_ulong) -> c_ulong
});
