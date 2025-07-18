use dtb;

use crate::{HHDM_REQUEST, println};

pub struct DTBReader<'a> {
    dtb_addr: usize,
    reader: dtb::Reader<'a>,
}
impl<'a> DTBReader<'a> {
    pub fn new(dtb_addr: usize) -> Self {
        let reader = unsafe { dtb::Reader::read_from_address(dtb_addr).unwrap() };
        DTBReader { dtb_addr, reader }
    }
    pub fn list_reserved_mem(&self) {
        for region in self.reader.reserved_mem_entries() {
            println!(
                "Reserved Memory: Start: {:#x}, Size: {:#x}",
                region.address, region.size
            );
        }
    }
}
