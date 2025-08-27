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
use core::ptr::write_volatile;
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
            0x424f4f54 => { // BOOT
                put_dword(echo);
                brom::uart_println("Hooking PL -> LK jump...");
                preloader::install_hooks();
                brom::uart_println("Jumping to PL, byebye!");
                core::arch::asm!("mov r0, 0x0","mov r1, 0x0","mov r2, 0x0","mov r3, 0x0","mov r4, 0x0","mov r5, 0x0","mov r6, 0x0");
                core::arch::asm!("ldr pc, =(0x201000)");
                core::hint::unreachable_unchecked();
            }
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
                brom::ffi::cmd_handler()
            }
            _ => {
                brom::uart_println("unknown command");
            }
        }
        echo = 0;
    }

}

fn put_dword(dword: u32) {
    usbdl_put_data(&dword.swap_bytes(), 4)
}
fn get_dword(data: &mut u32) {
    let mut recv = 0;
    usbdl_get_data(&mut recv, 4);
    *data = recv.swap_bytes()
}