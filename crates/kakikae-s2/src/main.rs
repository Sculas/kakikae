#![no_std]
#![no_main]
#![feature(c_variadic)]
#![allow(
    unsafe_op_in_unsafe_fn,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types,
    clippy::macro_metavars_in_unsafe,
    clippy::missing_safety_doc
)]

mod hooks;
mod log;

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    eprintln!("FATAL: an unrecoverable error has occurred: {}", info);
    loop {}
}

const _: kakikae_shared::S2_ENTRY_POINT = main;

// global_asm! doesn't respect the RUSTFLAGS, so it compiles in ARM mode anyway, and we MUST
// have Thumb2 mode here. So we use a #[naked] function here, that does respect the RUSTFLAGS.
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn start() {
    core::arch::naked_asm!("b main")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(pl_print_ptr: usize, in_pl_phase: *const bool) {
    log::init(pl_print_ptr, in_pl_phase);
    eprintln!("kakikae / stage 2 (LK, using 0x{:08X})", pl_print_ptr);

    eprintln!("Installing LK hooks");
    hooks::install();
}
