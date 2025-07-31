use crate::def_extern;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct RtcTime {
    pub sec: u16,
    pub min: u16,
    pub hour: u16,
    pub day: u16,
    pub mon: u16,
    pub year: u16,
}

def_extern! { vars;
    BOOT_REASON        @ 0x26A4F4   -> *mut u32,
    BLDR_JMP_HOOK_ADDR @ 0x2229c8   -> *mut u32,
}
def_extern! { fns;
    original_bldr_jump64   @ 0x2220B8+1 -> fn(addr: u32, arg1: u32, arg2: u32),
    rtc_mark_bypass_pwrkey @ 0x22EA5C+1 -> fn(),
    rtc_get_time           @ 0x22F504+1 -> fn(tm: *mut RtcTime),
}

// Cannot define this using dex_extern! due to the variadic nature of printf.
#[macro_export]
macro_rules! ffi_internal_printf {
    () => {
        unsafe {
            core::mem::transmute::<usize, extern "C" fn(*const core::ffi::c_char, ...) -> i32>(
                0x22D944usize + 1,
            )
        }
    };
}
