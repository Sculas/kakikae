pub const BACKUP_LEN: usize = 9; // code len (4) + pointer length (4) + align(1)

pub unsafe fn write_hook(target: *mut u8, hook_fn: usize) {
    const THUMB_NOP: u16 = 0xbf00; // asm: nop
    const LDR_PC_PC: [u16; 2] = [0xf8df, 0xf000]; // asm: ldr.w pc, [pc]

    let mut target = target as *mut u16;
    if target as u32 % 4 != 0 {
        unsafe {
            target.write_unaligned(THUMB_NOP);
            target = target.offset(1);
        }
    }

    unsafe {
        (target as *mut [u16; 2]).write_unaligned(LDR_PC_PC);
        (target.offset(2) as *mut usize).write_unaligned(hook_fn);
    }
}
