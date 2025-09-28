#![allow(dead_code)]

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// to provide global writer which can be used as interface from other modules
#[cfg(not(test))]
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

#[cfg(test)]
struct SyncUnsafeCell<T>(core::cell::UnsafeCell<T>);

#[cfg(test)]
unsafe impl<T> Sync for SyncUnsafeCell<T> {}

#[cfg(test)]
impl<T> SyncUnsafeCell<T> {
    const fn new(value: T) -> Self {
        SyncUnsafeCell(core::cell::UnsafeCell::new(value))
    }
    
    fn get(&self) -> *mut T {
        self.0.get()
    }
}

// Allocate mem to fake vga buf
#[cfg(test)]
lazy_static! {
    static ref FAKE_VGA: SyncUnsafeCell<[u8; 4000]> = 
        SyncUnsafeCell::new([0; 4000]); // 25*80*2 = 4000 bytes
    
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(FAKE_VGA.get() as *mut Buffer) },
    });
}

// #[cfg(test)]
// pub struct TestWriter {
//     column_position: usize,
//     color_code: ColorCode,
// }

// #[cfg(test)]
// lazy_static! {
//     pub static ref TEST_BUF: Mutex<[[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]> = {
//         Mutex::new([[ScreenChar {
//             ascii_character: b' ',
//             color_code: ColorCode::new(Color::LightGreen, Color::Black),
//         }; BUFFER_WIDTH]; BUFFER_HEIGHT])
//     };
//     pub static ref WRITER: Mutex<TestWriter> = Mutex::new(TestWriter {
//         column_position: 0,
//         color_code: ColorCode::new(Color::LightGreen, Color::Black),
//     });
// }

// #[cfg(test)]
// impl TestWriter {
//     pub fn write_byte(&mut self, byte: u8) {
//         match byte {
//             b'\n' => self.new_line(),
//             byte => {
//                 if self.column_position >= BUFFER_WIDTH {
//                     self.new_line();
//                 }
                
//                 let row = BUFFER_HEIGHT - 1;
//                 let col = self.column_position;
                
//                 // Write to test buffer
//                 let mut buffer = TEST_BUF.lock();
//                 buffer[row][col] = ScreenChar {
//                     ascii_character: byte,
//                     color_code: self.color_code,
//                 };
                
//                 self.column_position += 1;
//             }
//         }
//     }

//     pub fn write_string(&mut self, s: &str) {
//         for byte in s.bytes() {
//             match byte {
//                 0x20..=0x7e | b'\n' => self.write_byte(byte),
//                 _ => self.write_byte(0xfe),
//             }
//         }
//     }

//     fn new_line(&mut self) {
//         let mut buffer = TEST_BUF.lock();
        
//         // Scroll up
//         for row in 1..BUFFER_HEIGHT {
//             for col in 0..BUFFER_WIDTH {
//                 buffer[row - 1][col] = buffer[row][col];
//             }
//         }
        
//         // Clear bottom row
//         for col in 0..BUFFER_WIDTH {
//             buffer[BUFFER_HEIGHT - 1][col] = ScreenChar {
//                 ascii_character: b' ',
//                 color_code: self.color_code,
//             };
//         }
        
//         self.column_position = 0;
//     }
// }

// #[cfg(test)]
// impl fmt::Write for TestWriter {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         self.write_string(s);
//         Ok(())
//     }
// }


// #[cfg(test)]
// lazy_static! {
//     static ref MOCK_BUFFER: UnsafeCell<Buffer> = {
//         use core::mem::MaybeUninit;
//         let mut buf: MaybeUninit<Buffer> = MaybeUninit::uninit();

//         unsafe {
//             let buf_ptr = buf.as_mut_ptr();
//             for row in 0..BUFFER_HEIGHT {
//                 for col in 0..BUFFER_WIDTH {
//                     let ch_ptr = &mut (*buf_ptr).chars[row][col] as *mut Volatile<ScreenChar>;
//                     ch_ptr.write(Volatile::new(ScreenChar {
//                         ascii_character: b' ',
//                         color_code: ColorCode(0),
//                     }));
//                 }
//             }
//             UnsafeCell::new(buf.assume_init())
//         }
//     };

//     pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
//         column_position: 0,
//         color_code: ColorCode::new(Color::LightGreen, Color::Black),
//         buffer: unsafe { &mut *MOCK_BUFFER.get() }
//         // buffer: unsafe { &mut *(&MOCK_BUFFER as *const _ as *mut Buffer) }, // cast to mutable
//     });
// }


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, // lifetime valid for whole prog run time -> VGA Buff
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    // Since rust strings are utf-8
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII || newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not printable ASCII
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
