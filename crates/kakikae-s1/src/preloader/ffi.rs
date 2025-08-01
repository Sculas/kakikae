use kakikae_shared::def_extern;

pub type PrintfFn = extern "C" fn(*const core::ffi::c_char, ...) -> i32;

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
    PRINTF_PTR         @ 0x22D944+1 -> *mut u8,
    BOOT_REASON        @ 0x26A4F4   -> *mut u32,
    BLDR_JMP_HOOK_ADDR @ 0x2229c8   -> *mut u32,
}
def_extern! { fns;
    original_bldr_jump64   @ 0x2220B8+1 -> fn(addr: u32, arg1: u32, arg2: u32),
    rtc_mark_bypass_pwrkey @ 0x22EA5C+1 -> fn(),
    rtc_get_time           @ 0x22F504+1 -> fn(tm: *mut RtcTime),
}

