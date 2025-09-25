pub(crate) mod boot_menu;
pub(crate) mod display;
pub(crate) mod utils;

pub const LK_BASE: usize = 0x4C400000;
pub const LK_SIZE: usize = 0x00100000;

pub unsafe fn install() {
    utils::pattern_match();
    display::pattern_match();
    boot_menu::pattern_match();
    boot_menu::install_hooks();
    boot_menu::install_logo_hooks();
}

