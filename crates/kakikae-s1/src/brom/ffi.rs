use kakikae_shared::def_extern;

def_extern! { vars;
    SLA_PASSED_1 @ 0x102860 -> *mut u8,
    DAA_PASSED_1 @ 0x102A8C -> *mut u32,
    DAA_PASSED_2 @ 0x102A94 -> *mut u32,
}
def_extern! { fns;
    usbdl_get_dword    @ 0xE183 -> fn(n: *mut u32),
    usbdl_put_dword    @ 0xE1B7 -> fn(n: u32),
    send_usb_response  @ 0x4C8F -> fn(arg1: u32, arg2: u32, arg3: u32),
}
