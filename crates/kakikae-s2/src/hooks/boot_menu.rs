use super::{LK_BASE, LK_SIZE, utils::*};
use crate::*;
use core::ffi::*;

athook::pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "? ? ? ? ? ? ? 02 13 68 ? ? ? 03 ? ? ? 53" @ 1,
    !thumb fn mtk_wdt_disable()
}, {
    "10 B5 ? ? ? FA 04 46" @ 1,
    !thumb fn mt65xx_backlight_on()
}, {
    "2D E9 F8 43 ? ? ? ? ? ? ? 09" @ 1,
    !thumb fn mtk_wdt_init()
}, {
    "38 B5 04 46 ? ? ? FF" @ 1,
    !thumb fn mtk_detect_key(key: c_ushort) -> c_int
});

athook::install_hooks!(install_hooks; (LK_BASE, LK_SIZE) {
    "70 B5 61 4C" @ 1,
    !thumb fn boot_mode_select_hook(orig: _),
});

const MT65XX_BOOT_MENU_KEY: c_ushort = 0; // VOL_UP key
const MT65XX_MENU_OK_KEY: c_ushort = 1; // VOL_DOWN key

unsafe fn boot_mode_select_hook(orig: orig_boot_mode_select_hook) {
    lk_println!("LK: boot_mode_select_hook");
    video_println!("hello from LK boot menu hook!");

    let begin = get_timer(0);
    while get_timer(begin) < 50 {
        if mtk_detect_key(MT65XX_BOOT_MENU_KEY) == 1 {
            mtk_wdt_disable(); // disable watchdog to prevent reset
            mt65xx_backlight_on(); // turn the display backlight on
            video_println!("entering boot menu...");
            mdelay(300); // allow message to be displayed
            lk_show_custom_boot_menu(); // show custom boot menu
            mtk_wdt_init(); // reinitialize watchdog
            break; // continue to boot normally
        }
    }

    orig()
}

unsafe fn lk_show_custom_boot_menu() {
    video_println!("this will be a custom boot menu very soon :)");
}