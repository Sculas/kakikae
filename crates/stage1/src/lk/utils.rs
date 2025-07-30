use crate::lk::{LK_BASE, LK_SIZE};
use crate::{hooks::follow_bl_insn, pattern_match};
use core::ffi::*;

pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "? ? ? FF 22 68" @ 2 = follow_bl_insn,
    pub mdelay(n: c_ulong)
});
