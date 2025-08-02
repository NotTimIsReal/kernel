pub struct MairRegister {
    mt_normal: u8,
    mt_normal_no_caching: u8,
    mt_device_ngnrne: u8,
    mt_device_ngnre: u8,
}

impl MairRegister {
    //Indexes for memory types
    pub const MT_NORMAL: usize = 0;
    pub const MT_NORMAL_NO_CACHING: usize = 1;
    pub const MT_DEVICE_NGNRNE: usize = 3;
    pub const MT_DEVICE_NGNRE: usize = 4;

    pub fn new() -> Self {
        Self {
            mt_normal: 0xff,
            mt_normal_no_caching: 0x44,
            mt_device_ngnrne: 0x00,
            mt_device_ngnre: 0x04,
        }
    }

    fn mair_attr(attr: u64, index: usize) -> u64 {
        attr << (8 * index)
    }

    pub fn setup(&self) {
        let mair = Self::mair_attr(self.mt_normal as u64, Self::MT_NORMAL)
            | Self::mair_attr(self.mt_normal_no_caching as u64, Self::MT_NORMAL_NO_CACHING)
            | Self::mair_attr(self.mt_device_ngnrne as u64, Self::MT_DEVICE_NGNRNE)
            | Self::mair_attr(self.mt_device_ngnre as u64, Self::MT_DEVICE_NGNRE);

        // Store `mair` into MAIR_EL1 register
        unsafe {
            core::arch::asm!("msr mair_el1, {}", in(reg) mair);
        }
    }
}
