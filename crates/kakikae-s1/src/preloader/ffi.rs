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

pub const PRINTF_PTR: usize = 0x22D944 + 1;

def_extern! { vars;
    BOOT_REASON        @ 0x26A4F4 -> *mut u32,
}
def_extern! { fns;
    rtc_mark_bypass_pwrkey @ 0x22EA5C+1 -> fn(),
    rtc_get_time           @ 0x22F504+1 -> fn(tm: *mut RtcTime),
}
