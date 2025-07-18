pub struct UART {
    base: usize,
}
impl UART {
    pub fn new(base: usize) -> Self {
        UART { base }
    }
    fn write_to_register(&self, offset: usize, value: u32) {
        unsafe {
            ((self.base + offset) as *mut u32).write_volatile(value);
        }
    }
    // Initialize UART for basic operation
    unsafe fn unsafe_init(&self) {
        #[cfg(target_arch = "x86_64")]
        {
            todo!()
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        {
            let irbd_offset: usize = 0x24; // Interrupt Register Offset
            let fbrd_offset: usize = 0x28; // FIFO Register Offset
            let lcr_offset: usize = 0x2C; // Line Control Register Offset
            let cr_offset: usize = 0x30; // Control Register Offset
            let imsc_offset: usize = 0x34; // Interrupt Mask Set/Clear Register Offset
            let dmacr_offset: usize = 0x38; // DMA Control Register Offset
            // let cr_offset: usize = 0x40; // Control Register Offset
            self.write_to_register(cr_offset, 0x0);
            self.write_to_register(irbd_offset, 1);
            self.write_to_register(fbrd_offset, 40);
            self.write_to_register(lcr_offset, (1 << 4) | (1 << 5) | (1 << 6)); // 8 bits, no parity, one stop bit
            self.write_to_register(cr_offset, (1 << 0) | (1 << 8) | (1 << 9)); // Enable UART, TX, RX
        }
    }
    pub fn init(&self) {
        unsafe {
            self.unsafe_init();
        }
    }
    #[cfg(target_arch = "aarch64")]
    unsafe fn write_byte_aarch64(&self, byte: u8) {
        const TXFF: u32 = 1 << 5; // Transmit FIFO full flag
        let uart_fr: u32 = self.base as u32 + 0x18; // UART Flag Register offset
        unsafe {
            while ((uart_fr as *const u32).read_volatile() & TXFF) != 0 {}
            (self.base as *mut u8).write_volatile(byte as u8); // Write byte to data register
        }
    }
    unsafe fn write_byte_x86_64(&self, byte: u8) {
        // Wait for the UART to be ready to transmit
        while unsafe { ((self.base + 5) as *const u8).read_volatile() } & 0x20 == 0 {}
        // Write the byte to the UART data register
        unsafe { (self.base as *mut u8).write_volatile(byte) };
    }

    unsafe fn read_byte(&self) -> u8 {
        // Read a byte from the UART data register
        unsafe { (self.base as *const u8).read_volatile() }
    }
    fn write_byte(&self, byte: u8) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            self.write_byte_x86_64(byte)
        };
        #[cfg(target_arch = "aarch64")]
        unsafe {
            self.write_byte_aarch64(byte)
        };
        // #[cfg(target_arch = "riscv64")]
        // unsafe {
        //     self.write_byte_aarch64(byte)
        // }; // Assuming similar behavior for RISC-V
    }
    pub fn write_string(&self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
}

impl Default for UART {
    fn default() -> Self {
        //default bases for different architectures
        #[cfg(target_arch = "x86_64")]
        return Self::new(0x3F8);

        #[cfg(target_arch = "aarch64")]
        return Self::new(0x900_0000); // PL011 UART base address (QEMU virt machine)}
        // Alternative addresses to try if the above doesn't work:
        // 0x1c090000 - Some ARM development boards
        // 0x10009000 - Some other QEMU configurations
        #[cfg(target_arch = "riscv64")]
        return Self::new(0x10000000); // NS16550A UART base address (QEMU virt machine)
    }
}
