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

pub type PL_PRINT = fn(args: core::fmt::Arguments, module: &str, line: u32);

pub const PL_BASE: usize = 0x201000;
pub const PL_SIZE: usize = 0x50000;

pub const S1_BASE_ADDR: usize = 0x250000; // keep in sync with kakikae-s1/link.x
pub const S2_BASE_ADDR: usize = 0x48380000; // keep in sync with kakikae-s2/link.x
pub type S2_ENTRY_POINT = unsafe extern "C" fn(pl_print_ptr: usize);

#[macro_export]
macro_rules! def_extern {
    (vars; $($name:ident @ $addr:literal $(+$thumb:literal)? -> $($type:ty)*),*$(,)?) => {
        $(pub const $name: $($type)* = ($addr $(+$thumb)?) as *mut _;)*
    };
    (fns; $($name:ident @ $addr:literal $(+$thumb:literal)? -> fn($($arg:ident : $argtype:ty),*) $(-> $rtype:ty)?),*$(,)?) => {
        $(#[inline(always)] pub fn $name($($arg: $argtype),*) $(-> $rtype)? {
            let func: extern "C" fn($($arg: $argtype),*) $(-> $rtype)? = unsafe { core::mem::transmute($addr $(+$thumb)?) };
            func($($arg),*)
        })*
    };
}
