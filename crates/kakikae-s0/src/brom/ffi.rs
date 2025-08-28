use kakikae_shared::def_extern;

def_extern! { vars;
    SLA_PASSED_1 @ 0x10286C -> *mut u8,
    DAA_PASSED_1 @ 0x102ACC -> *mut u32,
    DAA_PASSED_2 @ 0x102AD4 -> *mut u32,
    UART_REG0    @ 0x11002014 -> *mut u32,
    WATCHDOG     @ 0x10007000 -> *mut u32,
    USBDL_PUT_WORD_FN  @ 0xE13F -> *mut u32,
}
def_extern! { fns;
    usbdl_get_dword    @ 0xE183 -> fn(n: *mut u32, sz: u32) -> u32,
    usbdl_put_dword    @ 0xE1B7 -> fn(n: *const u32, sz: u32),
    usbdl_get_data     @ 0xE1F9 -> fn(n: *mut u32, sz: u32),
    usbdl_put_data     @ 0xE287 -> fn(n: *const u32, sz: u32),
    send_usb_response  @ 0x4C8F -> fn(arg1: u32, arg2: u32, arg3: u32),
    cmd_handler        @ 0xF029 -> fn(),
}