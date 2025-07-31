use crate::eprintln;

mod boot_menu;
mod display;
mod utils;

pub const LK_BASE: usize = 0x4C400000;
pub const LK_SIZE: usize = 0x00100000;

pub unsafe fn lk_install_hooks() {
    eprintln!("utils::pattern_match");
    utils::pattern_match();

    eprintln!("display::pattern_match");
    display::pattern_match();

    eprintln!("boot_menu::pattern_match");
    boot_menu::pattern_match();

    eprintln!("boot_menu::install_hooks");
    boot_menu::install_hooks();
}
