pub fn follow_bl_insn(match_addr: u32) -> u32 {
    let instruction = unsafe { core::ptr::read_unaligned(match_addr as *const u32) };
    let first_halfword = (instruction >> 16) & 0xFFFF;
    let second_halfword = instruction & 0xFFFF;

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

    (match_addr + 4).wrapping_add(imm32)
}
