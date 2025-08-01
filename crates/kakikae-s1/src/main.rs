#![no_std]
#![no_main]
#![allow(
    unsafe_op_in_unsafe_fn,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types,
    clippy::macro_metavars_in_unsafe,
    clippy::missing_safety_doc
)]

use core::ptr::write_volatile;

mod brom;
mod preloader;

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    eprintln!("FATAL: an unrecoverable error has occurred: {}", info);
    loop {}
}

core::arch::global_asm!(include_str!("start.S"));

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main() -> ! {
    // const HELLO_MAGIC: u32 = 0x48454C4F; // "HELO"

    brom::uart_println("kakikae / stage 1 (mt6785)");
    brom::ffi::send_usb_response(1, 0, 1); // prevent timeout

    // TODO: add this back in before release...
    // brom::uart_println("acknowledging our existence...");
    // brom::ffi::usbdl_put_dword(HELLO_MAGIC);
    // let mut echo = 0;
    // brom::ffi::usbdl_get_dword(&mut echo);
    // if echo != HELLO_MAGIC {
    //     brom::uart_println("echo mismatch, halting!");
    //     panic!("echo mismatch, can't continue");
    // }

    brom::uart_println("Disabling SLA/DAA checks...");
    write_volatile(brom::ffi::SLA_PASSED_1, 1);
    write_volatile(brom::ffi::DAA_PASSED_1, 1);
    write_volatile(brom::ffi::DAA_PASSED_2, u32::MAX);

    brom::uart_println("Hooking PL -> LK jump...");
    preloader::install_preloader_bldr_jump64_hook();

    brom::uart_println("Jumping to PL, byebye!");
    core::arch::asm!("ldr pc, =(0x201000)");
    core::hint::unreachable_unchecked();
}
