mod boot_menu;
mod display;
mod utils;

pub const LK_BASE: usize = 0x4C400000;
pub const LK_SIZE: usize = 0x00100000;

pub unsafe fn install() {
    utils::pattern_match();
    utils::pattern_patch();
    display::pattern_match();
    boot_menu::pattern_match();
    boot_menu::install_hooks();
}

pub unsafe fn dprintf(s: &str) {
    utils::dprintf(s.as_ptr() as _);
}
