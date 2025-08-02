use crate::{EXECUTABLE_MEM_REQUEST, HHDM_REQUEST};

pub struct Pointer {
    pub addr: usize,
}

impl Pointer {
    pub fn new(addr: usize) -> Self {
        Pointer { addr }
    }

    pub fn as_mut<T>(&mut self) -> *mut T {
        self.addr as *mut T
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.addr as *const T
    }
    pub fn write(&self, value: u8) {
        //let off = HHDM_REQUEST.get_response().unwrap().offset();
        let off = 0;
        unsafe {
            ((self.addr + off as usize) as *mut u8).write(value);
        }
    }
    pub fn read(&self) -> u8 {
        let off = HHDM_REQUEST.get_response().unwrap().offset();
        unsafe { ((self.addr + off as usize) as *const u8).read() }
    }
}
