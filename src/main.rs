#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;
mod vga_buffer;

#[panic_handler]

fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    vga_buffer::WRITER.lock().write_str("CrabOS Supremacy").unwrap();
    write!(vga_buffer::WRITER.lock(), "\n some numbers: {} {}", 42, 1.33).unwrap();

    loop {}
}
