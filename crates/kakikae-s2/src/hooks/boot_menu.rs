use super::{LK_BASE, LK_SIZE, utils::*};
use crate::*;
use core::ffi::*;
use crate::hooks::display::*;

athook::pattern_match!(pattern_match; (LK_BASE, LK_SIZE) {
    "? ? ? ? ? ? ? 02 13 68 ? ? ? 03 ? ? ? 53" @ 1,
    mtk_wdt_disable()
}, {
    "10 B5 ? ? ? FA 04 46" @ 1,
    mt65xx_backlight_on()
}, {
    "2D E9 F8 43 ? ? ? ? ? ? ? 09" @ 1,
    mtk_wdt_init()
}, {
    "38 B5 04 46 ? ? ? FF" @ 1,
    mtk_detect_key(key: c_ushort) -> c_int
});

athook::install_hooks!(install_hooks; (LK_BASE, LK_SIZE) {
    "70 B5 61 4C" @ 1,
    boot_mode_select_hook(orig: _),
});

athook::install_hooks!(install_logo_hooks; (LK_BASE, LK_SIZE) {
    "20 48 86 22" @ 1,
    lk_show_logo(orig: _) -> u32,
});

const MT65XX_BOOT_MENU_KEY: c_ushort = 0; // VOL_UP key
const MT65XX_MENU_OK_KEY: c_ushort = 1; // VOL_DOWN key

unsafe fn boot_mode_select_hook(orig: orig_boot_mode_select_hook) {

    let begin = get_timer(0);
    while get_timer(begin) < 50 {
        if mtk_detect_key(MT65XX_BOOT_MENU_KEY) == 1 {
            mtk_wdt_disable(); // disable watchdog to prevent reset
            mt65xx_backlight_on(); // turn the display backlight on
            mt65xx_leds_brightness_set(MTK_LEDS::MT65XX_LED_TYPE_LCD as c_int, 1024); // set brightness to half
            mdelay(1000); // allow message to be displayed
            mtk_wdt_init(); // reinitialize watchdog
            break; // continue to boot normally
        }
    }

    orig()
}


unsafe fn lk_show_logo(orig: orig_lk_show_logo) -> u32 {
    init_fb_screen();
    mt65xx_backlight_on();
    fill_rect_with_content_by_32bit_argb8888(
        get_fb_addr(),
        Rectangle { left :0, top: 0, right: 1080, bottom: 2160 },
        0x48380000 as *const _, // todo: custom logo
        *PHYSICAL_SCREEN_DATA,
        32);
    mt_disp_update(0, 0, 1080, 2160);
    mdelay(2000);
    return 0
}
pub enum MTK_LEDS {
    MT65XX_LED_TYPE_RED = 0,
    MT65XX_LED_TYPE_GREEN = 1,
    MT65XX_LED_TYPE_BLUE = 2,
    MT65XX_LED_TYPE_JOGBALL = 3,
    MT65XX_LED_TYPE_KEYBOARD = 4,
    MT65XX_LED_TYPE_BUTTON = 5,
    MT65XX_LED_TYPE_LCD = 6,
    MT65XX_LED_TYPE_TOTAL = 7,
}