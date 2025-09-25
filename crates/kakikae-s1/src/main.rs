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

use core::ops::Div;
use core::ptr::{null_mut, write_volatile};
use crate::brom::ffi::{usbdl_get_data, usbdl_put_data};

mod brom;
mod preloader;

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    pl_println!("FATAL: an unrecoverable error has occurred: {}", info);
    loop {}
}

core::arch::global_asm!(include_str!("start.S"));

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main() -> ! {
    const HELLO_MAGIC: u32 = 0x48454C4F; // "HELO"

    brom::uart_println("acknowledging our existence...");

    let mut echo = 0;
    get_dword(&mut echo);
    if echo != HELLO_MAGIC {
        brom::uart_println("echo mismatch, halting!");
        panic!("echo mismatch, can't continue");
    }
    put_dword(HELLO_MAGIC);

    loop {
        get_dword(&mut echo);
        match echo {
            0x44415441 => { // DATA
                put_dword(echo);
                let mut location = 0;
                get_dword(&mut location);
                put_dword(location);
                let mut size = 0;
                get_dword(&mut size);
                put_dword(size);
                let mut pos = 0;
                while pos <= size - 64 {
                    usbdl_get_data((location + pos) as *mut u32, 64);
                    pos += 64;
                }
                continue
            }
            0x434f4d44 => { // COMD
                put_dword(echo);
                brom::uart_println("Hooking PL -> LK jump...");
                preloader::install_bldr_jump_hook();
                preloader::log::install_handshake_patch();
                brom::ffi::cmd_handler()
            }
            _ => {
                brom::uart_println("unknown command");
            }
        }
        echo = 0;
    }

}

#[inline(always)]
unsafe fn put_dword(dword: u32) {
    brom::ffi::usbdl_put_dword(dword as *mut u32, 1)
}
#[inline(always)]
unsafe fn get_dword(data: &mut u32) {
    *data = brom::ffi::usbdl_get_dword(null_mut(), 1);
}
unsafe fn wdt_reboot() -> ! {
    brom::ffi::WATCHDOG.offset(8/4).write_volatile(0x1971);
    brom::ffi::WATCHDOG.offset(0/4).write_volatile(0x22000014);
    brom::ffi::WATCHDOG.offset(0x14/4).write_volatile(0x1209);
    core::hint::unreachable_unchecked()
}