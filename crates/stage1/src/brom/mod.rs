pub mod ffi;

pub fn uart_println(s: &str) {
    s.chars().for_each(low_uart_putc);
    low_uart_putc('\r');
}

fn low_uart_putc(c: char) {
    unsafe {
        const UART_REG0: *mut u32 = 0x11002014 as *mut _;
        const UART_REG1: *mut u32 = 0x11002000 as *mut _;
        while ((core::ptr::read_volatile(UART_REG0)) & 0x20) == 0 {}
        core::ptr::write_volatile(UART_REG1, c as u32);
    }
}
