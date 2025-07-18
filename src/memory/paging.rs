use aarch64_paging::{
    idmap::{self, IdMap},
    paging::{Attributes, MemoryRegion},
};
use buddy_system_allocator::{Heap, LockedHeap};
use core::{arch::asm, ptr::NonNull, slice::SplitInclusiveMut};
use limine::memory_map::EntryType;

use crate::{HHDM_REQUEST, MEMORY_MAP_REQUEST, println};
#[global_allocator]
static ALLOCATOR: LockedHeap<32> = LockedHeap::new();
const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
#[cfg(target_arch = "aarch64")]
static mut ID_MAPS: Vec<IdMap> = Vec::new();
pub fn init_heap() {
    unsafe {
        ALLOCATOR.lock().init(&raw mut HEAP as usize, HEAP_SIZE);
    }
}
extern crate alloc;
use alloc::vec::Vec;
trait MemorySetupT {
    fn new() -> Self;
    fn map(&self);
}
#[cfg(target_arch = "aarch64")]
struct ARM64MemorySetup {}

#[cfg(target_arch = "aarch64")]
impl MemorySetupT for ARM64MemorySetup {
    fn new() -> Self {
        ARM64MemorySetup {}
    }

    fn map(&self) {
        const NORMAL_CACHEABLE: Attributes = Attributes::VALID
            .union(Attributes::ATTRIBUTE_INDEX_1)
            .union(Attributes::INNER_SHAREABLE)
            .union(Attributes::ACCESSED)
            .union(Attributes::NON_GLOBAL);
        const DEVICE_ATTR: Attributes = Attributes::VALID
            .union(Attributes::ATTRIBUTE_INDEX_0)
            .union(Attributes::ACCESSED)
            .union(Attributes::UXN);
        let mem_map = MEMORY_MAP_REQUEST.get_response().unwrap().entries();

        for entry in mem_map {
            let mut idmap =
                idmap::IdMap::new(1, 1, aarch64_paging::paging::TranslationRegime::El1And0);
            //check if 0x900_0000 is in the reserved region
            match entry.entry_type {
                EntryType::USABLE => {
                    println!("{:#X} - {:#X}", entry.base, entry.base + entry.length,);
                    idmap
                        .map_range(
                            &MemoryRegion::new(
                                entry.base as usize,
                                (entry.base + entry.length) as usize,
                            ),
                            NORMAL_CACHEABLE,
                        )
                        .unwrap();
                }
                EntryType::RESERVED => {
                    // println!(
                    //     "Reserved memory region: {:#X} - {:#X}",
                    //     entry.base,
                    //     entry.base + entry.length
                    // );

                    idmap
                        .map_range(
                            &MemoryRegion::new(
                                entry.base as usize,
                                (entry.base + entry.length) as usize,
                            ),
                            DEVICE_ATTR,
                        )
                        .unwrap();
                }
                // EntryType::BOOTLOADER_RECLAIMABLE | EntryType::ACPI_RECLAIMABLE => {
                //     idmap
                //         .map_range(
                //             &MemoryRegion::new(
                //                 entry.base as usize,
                //                 (entry.base + entry.length) as usize,
                //             ),
                //             Attributes::READ_ONLY | Attributes::VALID | Attributes::ACCESSED,
                //         )
                //         .unwrap();
                // }
                _ => {
                    idmap
                        .map_range(
                            &MemoryRegion::new(
                                entry.base as usize,
                                (entry.base + entry.length) as usize,
                            ),
                            DEVICE_ATTR,
                        )
                        .unwrap();
                }
            }
            unsafe {
                idmap.activate();
            }
            #[allow(static_mut_refs)]
            unsafe {
                ID_MAPS.push(idmap)
            };
        }
    }
}
struct X86MemorySetup;
impl MemorySetupT for X86MemorySetup {
    fn new() -> Self {
        todo!();
    }

    fn map(&self) {
        todo!();
    }
}

pub struct MemorySetup {
    #[cfg(target_arch = "aarch64")]
    setup_api: ARM64MemorySetup,
    #[cfg(target_arch = "x86_64")]
    setup_api: X86MemorySetup,
}

impl MemorySetup {
    pub fn new() -> Self {
        #[cfg(target_arch = "aarch64")]
        let api = ARM64MemorySetup::new();
        #[cfg(target_arch = "x86_64")]
        let api = X86MemorySetup::new();

        MemorySetup { setup_api: api }
    }

    pub fn map(&self) {
        self.setup_api.map();
    }
}
