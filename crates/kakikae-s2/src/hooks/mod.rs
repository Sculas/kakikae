mod boot_menu;
mod display;
mod utils;

pub const LK_BASE: usize = 0x4C400000;
pub const LK_SIZE: usize = 0x00100000;

pub unsafe fn install() {
    utils::pattern_match();
    display::pattern_match();
    boot_menu::pattern_match();
    boot_menu::install_hooks();
}

pub unsafe fn dprintf(s: &str) {
    utils::dprintf(s.as_ptr() as _);
}

pub fn follow_bl_insn(match_addr: u32) -> u32 {
    let instruction = unsafe { core::ptr::read_unaligned(match_addr as *const u32) };
    crate::eprintln!(
        "match_addr = {:#010X}, instruction = {:#010X}",
        match_addr,
        instruction
    );

    let first_halfword = (instruction >> 16) & 0xFFFF;
    let second_halfword = instruction & 0xFFFF;
    crate::eprintln!(
        "first_halfword = {:#010X}, second_halfword = {:#010X}",
        first_halfword,
        second_halfword
    );

    let s = (first_halfword >> 10) & 1;
    let imm10 = first_halfword & 0x3FF;
    let j1 = (second_halfword >> 13) & 1;
    let j2 = (second_halfword >> 11) & 1;
    let imm11 = second_halfword & 0x7FF;

    let i1 = if j1 ^ s != 0 { 0 } else { 1 };
    let i2 = if j2 ^ s != 0 { 0 } else { 1 };

    let mut imm32 = (s << 24) | (i1 << 23) | (i2 << 22) | (imm10 << 12) | (imm11 << 1);
    if s != 0 {
        imm32 |= 0xFF000000;
    }

    let result = (match_addr + 4).wrapping_add(imm32);
    crate::eprintln!("result = {:#010X}", result);
    result
}
