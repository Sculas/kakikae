use super::{LK_BASE, LK_SIZE};
use crate::*;
use athook::follow_bl_insn;
use core::ffi::*;

athook::pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "? ? ? FF 22 68" @ 1 = follow_bl_insn,
    pub !thumb fn mdelay(n: c_ulong)
}, {
    "10 B5 04 46 ? ? ? FF A0 42" @ 1,
    pub !thumb fn get_timer(base: c_ulong) -> c_ulong
}, {
    "0F B4 ? ? F0 B5 9B B0" @ 1,
    pub !thumb fn dprintf(fmt: *const c_char) -> c_int = ...
});

athook::pattern_patch!(pattern_patch; (LK_BASE, LK_SIZE) {
    "BA F1 01 0F 07 D1 DF F8 38 05" @ 1,
    "?? ?? 00 ?? ?? ?? ?? ?? ?? ??" = enable_logging,
});
