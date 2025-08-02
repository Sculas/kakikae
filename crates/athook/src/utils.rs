// use crate::MatchAddress;
// use core::ptr::read_unaligned;
// 
// pub unsafe fn follow_bl_insn(match_addr: MatchAddress) -> MatchAddress {
//     let insn_addr = match_addr.0 as *const u16;
// 
//     let first_halfword = read_unaligned(insn_addr) as u32;
//     let second_halfword = read_unaligned(insn_addr.offset(1)) as u32;
// 
//     let s = (first_halfword >> 10) & 1;
//     let imm10 = first_halfword & 0x3FF;
//     let j1 = (second_halfword >> 13) & 1;
//     let j2 = (second_halfword >> 11) & 1;
//     let imm11 = second_halfword & 0x7FF;
// 
//     let i1 = if j1 ^ s != 0 { 0 } else { 1 };
//     let i2 = if j2 ^ s != 0 { 0 } else { 1 };
// 
//     let mut imm32 = (s << 24) | (i1 << 23) | (i2 << 22) | (imm10 << 12) | (imm11 << 1);
//     if s != 0 {
//         imm32 |= 0xFF000000;
//     }
// 
//     MatchAddress((match_addr.0 + 4).wrapping_add(imm32))
// }
