use kakikae_shared::def_extern;

def_extern! { vars;
    WATCHDOG     @ 0x10007000 -> *mut u32,
}

def_extern! { fns;
    usbdl_get_dword    @ 0xE183 -> fn(n: *mut u32, sz: u32) -> u32,
    usbdl_put_dword    @ 0xE1B7 -> fn(n: *const u32, sz: u32),
    usbdl_get_data     @ 0xE1F9 -> fn(n: *mut u32, sz: u32),
    usbdl_put_data     @ 0xE287 -> fn(n: *const u32, sz: u32),
    cmd_handler        @ 0xF029 -> fn(),
}