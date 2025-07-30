mod boot_menu;
mod display;
mod utils;

pub const LK_BASE: usize = 0x4C400000;
pub const LK_SIZE: usize = 0x00100000;

pub unsafe fn lk_install_hooks() {
    utils::pattern_match();

    boot_menu::pattern_match();
    boot_menu::install_hooks();
}
