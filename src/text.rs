use core::fmt::Write;

use flanterm::sys as ft;

pub struct TermEmulator {
    ctx: *mut ft::flanterm_context,
}
impl TermEmulator {
    pub fn new(
        fb_ptr: *mut u32,
        width: usize,
        height: usize,
        pitch: usize,
        red_mask_size_shift: (u8, u8),
        green_mask_size_shift: (u8, u8),
        blue_mask_size_shift: (u8, u8),
    ) -> Self {
        let ctx = unsafe {
            ft::flanterm_fb_init(
                None,
                None,
                fb_ptr,
                width,
                height,
                pitch,
                red_mask_size_shift.0,
                red_mask_size_shift.1,
                green_mask_size_shift.0,
                green_mask_size_shift.1,
                blue_mask_size_shift.0,
                blue_mask_size_shift.1,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                0,
                1,
                0,
                0,
                0,
            )
        };
        TermEmulator { ctx }
    }
    pub fn print(&self, msg: &str) {
        let count = msg.len();
        if count == 0 {
            return;
        }
        let cmsg = msg.as_ptr() as *const core::ffi::c_char;
        unsafe {
            ft::flanterm_write(self.ctx, cmsg, count);
        }
    }
    pub fn println(&self, msg: &str) {
        self.print(msg);
        self.print("\n");
    }
}
unsafe impl Sync for TermEmulator {}
impl Write for TermEmulator {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}
//define println! macro for TermEmulator
#[macro_export]
macro_rules! print{
    ()=>{
        if let Some(term) = unsafe { $crate::TERM_EMULATOR.as_mut() } {
            term.print("");
        }
    };
    ($($arg:tt)*) => {
        #[allow(static_mut_refs)]
        if let Some(term) = unsafe { $crate::TERM_EMULATOR.as_mut() } {
            use core::fmt::Write;
            term.write_fmt(format_args!($($arg)*)).unwrap();
        }
    };
}

#[macro_export]
macro_rules! println {
    () => {
        if let Some(term) = unsafe { $crate::TERM_EMULATOR.as_mut() } {
            term.println("");
        }
    };
    ($($arg:tt)*) => {
       crate::print!("{}\n", format_args!($($arg)*))
    };
}
