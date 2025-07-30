use crate::eprintln;

mod boot_menu;
mod display;
mod utils;

pub const LK_BASE: usize = 0x4C400000;
pub const LK_SIZE: usize = 0x00100000;

pub unsafe fn lk_install_hooks() {
    eprintln!("(init) utils::pattern_match");
    utils::pattern_match();

    let start_ms = utils::get_timer(0);

    eprintln!(
        "({}ms) boot_menu::pattern_match",
        utils::get_timer(start_ms)
    );
    boot_menu::pattern_match();

    eprintln!(
        "({}ms) boot_menu::install_hooks",
        utils::get_timer(start_ms)
    );
    boot_menu::install_hooks();
}
