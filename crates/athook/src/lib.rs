#![no_std]
#![no_main]
#![feature(portable_simd)]
#![allow(
    unsafe_op_in_unsafe_fn,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types,
    clippy::macro_metavars_in_unsafe,
    clippy::missing_safety_doc
)]

mod macros;
mod raw;
mod utils;

pub use raw::BACKUP_LEN;
pub use utils::*;

pub unsafe fn install_hook_at(target: *mut u32, hook_fn_addr: usize) {
    target.write_unaligned(0x47304e00); // ldr r6, [pc, #0]; bx r6
    target.offset(1).write_unaligned(hook_fn_addr as _);
}

#[doc(hidden)]
pub mod __private {
    use super::*;
    pub use athook_macros::*;
    pub use pastey::*;
    pub use patterns::*;

    pub unsafe fn enable_hook(orig_fn: *mut u8, hook_fn: *const u8) -> [u8; BACKUP_LEN] {
        let orig_fn = (orig_fn as usize & !1) as *mut u8;
        let backup = orig_fn.cast::<[u8; BACKUP_LEN]>().read_unaligned();
        raw::write_hook(orig_fn, hook_fn as usize);
        raw::clear_cache_and_flush(orig_fn, orig_fn.add(BACKUP_LEN));
        backup
    }

    #[rustfmt::skip]
    pub unsafe fn disable_hook(orig_fn: *mut u8, orig_code: [u8; BACKUP_LEN]) {
        let orig_fn = (orig_fn as usize & !1) as *mut u8;
        orig_fn.cast::<[u8; BACKUP_LEN]>().write_unaligned(orig_code);
        raw::clear_cache_and_flush(orig_fn, orig_fn.add(BACKUP_LEN));
    }
}
