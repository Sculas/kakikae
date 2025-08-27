use kakikae_shared::def_extern;

def_extern! { fns;
    usbdl_get_data     @ 0xE1F9 -> fn(n: *mut u32, sz: u32),
    usbdl_put_data     @ 0xE287 -> fn(n: *const u32, sz: u32),
    cmd_handler        @ 0xF029 -> fn(),
}