#![no_std]
#![no_main]
#![allow(
    unsafe_op_in_unsafe_fn,
    non_snake_case,
    non_camel_case_types,
    clippy::macro_metavars_in_unsafe,
    clippy::missing_safety_doc
)]

use core::ptr::null_mut;
use crate::brom::ffi::WATCHDOG;

pub mod brom;


core::arch::global_asm!(include_str!("start.S"));

#[inline(never)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe {
        wdt_reboot()
    }
}



#[unsafe(no_mangle)]
pub unsafe extern "C" fn main() -> ! {
    WATCHDOG.write_volatile(0x22000064);
    // prevent timeout
    brom::ffi::send_usb_response(1, 0, 1);


    // Fix usbdl send ptrs
    let mpos = brom::ffi::USBDL_PUT_WORD_FN as usize + 7;
    let offset = ldr_lit(mpos, *(mpos as *const u16));
    let usbdl_ptr = offset as *mut u32;
    (*usbdl_ptr as *mut u32).offset(2).write_volatile(usbdl_ptr.offset(2).read_volatile());


    // Disable security
    core::ptr::write_volatile(brom::ffi::SLA_PASSED_1, 1);
    core::ptr::write_volatile(brom::ffi::DAA_PASSED_1, 1);
    core::ptr::write_volatile(brom::ffi::DAA_PASSED_2, u32::MAX);

    const HELLO_MAGIC: u32 = 0x48454C4F; // "HELO"

    let mut echo = 0;

    get_dword(&mut echo);
    // reboot if magic mismatches
    if echo != HELLO_MAGIC {
        wdt_reboot()
    }
    put_dword(echo);

    let mut location = 0;
    get_dword(&mut location);
    put_dword(location);
    let mut size = 0;
    get_dword(&mut size);
    put_dword(size);

    let mut pos = 0;
    while pos <= size {
        brom::ffi::usbdl_get_data((location + pos) as *mut u32, 64);
        pos += 64;
    }
    // Switch to stage 1
    // Should be consistent with link.x
    core::arch::asm!(
            "ldr pc, ={stage1}",
            stage1 = const kakikae_shared::S1_BASE_ADDR,
    );
    core::hint::unreachable_unchecked();
}
#[inline(always)]
unsafe fn put_dword(dword: u32) {
    brom::ffi::usbdl_put_dword(dword as *mut u32, 1)
}
#[inline(always)]
unsafe fn get_dword(data: &mut u32) {
    *data = brom::ffi::usbdl_get_dword(null_mut(), 1);
}
fn ldr_lit(curpc: usize, ins: u16) -> usize {
    let imm8 = (ins & 0xFF) as usize;
    let pc = curpc / 4 * 4;
    pc + (imm8 * 4) + 4
}
unsafe fn wdt_reboot() -> ! {
    brom::ffi::WATCHDOG.offset(8/4).write_volatile(0x1971);
    brom::ffi::WATCHDOG.offset(0/4).write_volatile(0x22000014);
    brom::ffi::WATCHDOG.offset(0x14/4).write_volatile(0x1209);
    core::hint::unreachable_unchecked()
}