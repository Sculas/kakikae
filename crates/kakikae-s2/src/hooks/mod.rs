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

pub unsafe fn follow_bl_insn(match_addr: athook::MatchAddress) -> athook::MatchAddress {
    crate::eprintln!("match_addr = {:#010X}", match_addr.0);
    let insn_addr = match_addr.0 as *const u16;
    let tmp_addr_log = insn_addr as u32;
    crate::eprintln!("insn_addr = {:#010X}", tmp_addr_log);

    let first_halfword = core::ptr::read_unaligned(insn_addr) as u32;
    crate::eprintln!("first_halfword = {:#010X}", first_halfword);
    let second_halfword = core::ptr::read_unaligned(insn_addr.offset(1)) as u32;
    crate::eprintln!("second_halfword = {:#010X}", second_halfword);

    let s = (first_halfword >> 10) & 1;
    let imm10 = first_halfword & 0x3FF;
    let j1 = (second_halfword >> 13) & 1;
    let j2 = (second_halfword >> 11) & 1;
    let imm11 = second_halfword & 0x7FF;
    crate::eprintln!("a = {} {} {} {} {}", s, imm10, j1, j2, imm11);

    let i1 = if j1 ^ s != 0 { 0 } else { 1 };
    let i2 = if j2 ^ s != 0 { 0 } else { 1 };
    crate::eprintln!("b = {} {}", i1, i2);

    let mut imm32 = (s << 24) | (i1 << 23) | (i2 << 22) | (imm10 << 12) | (imm11 << 1);
    crate::eprintln!("imm32 a = {}", imm32);
    if s != 0 {
        imm32 |= 0xFF000000;
    }
    crate::eprintln!("imm32 b = {}", imm32);

    let result = (match_addr.0 + 4).wrapping_add(imm32);
    crate::eprintln!("result = {}", result);
    athook::MatchAddress(result)
}