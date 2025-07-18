#![no_std]
#![no_main]
#![feature(stmt_expr_attributes)]
extern crate core;

mod pci;
mod text;
mod uart;
use core::arch::asm;
mod dtb;
mod hw;
mod memory;
use limine::BaseRevision;
use limine::memory_map::EntryType;
use limine::request::{
    FramebufferRequest, MemoryMapRequest, PagingModeRequest, RequestsEndMarker,
    RequestsStartMarker, StackSizeRequest,
};

use crate::dtb::DTBReader;
use crate::memory::paging::{self, init_heap};
use crate::pci::PCIInterface;
use crate::text::TermEmulator;

/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
/// Be sure to mark all limine requests with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
#[used]
#[unsafe(link_section = ".requests")]
static PAGING_REQUEST: PagingModeRequest =
    PagingModeRequest::new().with_mode(limine::paging::Mode::FOUR_LEVEL);
// Some reasonable size
#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: limine::request::HhdmRequest = limine::request::HhdmRequest::new();
#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();
const STACK_SIZE: u64 = 0x100000;
#[used]
#[unsafe(link_section = ".requests")]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(STACK_SIZE);
/// Define the stand and end markers for Limine requests.
#[used]
#[unsafe(link_section = ".requests")]
static DTB_REQUEST: limine::request::DeviceTreeBlobRequest =
    limine::request::DeviceTreeBlobRequest::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
static mut TERM_EMULATOR: Option<TermEmulator> = None;
#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());
    let framebufferresp = FRAMEBUFFER_REQUEST.get_response().unwrap();
    let fb = framebufferresp.framebuffers().next().unwrap();
    let term = text::TermEmulator::new(
        fb.addr().cast::<u32>(),
        fb.width() as usize,
        fb.height() as usize,
        fb.pitch() as usize,
        (fb.red_mask_size(), fb.red_mask_shift()),
        (fb.green_mask_size(), fb.green_mask_shift()),
        (fb.blue_mask_size(), fb.blue_mask_shift()),
    );
    unsafe {
        TERM_EMULATOR = Some(term);
    }
    init_heap();
    let pagingresp = PAGING_REQUEST.get_response().unwrap().mode();
    println!("Booting into Kernel");
    match pagingresp {
        limine::paging::Mode::FOUR_LEVEL => println!("Using FOUR_LEVEL paging mode"),
        limine::paging::Mode::FIVE_LEVEL => println!("Using FIVE_LEVEL paging mode"),
        _ => println!("Using unknown paging mode"),
    }
    let paging_offset = HHDM_REQUEST.get_response().unwrap().offset();
    println!("HHDM Offset: {:#X}", paging_offset);
    paging::MemorySetup::new().map();
    crate::dtb::DTBReader::new(0x4000_0000).list_reserved_mem();

    for c in b"Hello, world!\r" {
        println!("{}", c);
        hw::Pointer::new(0x900_0000).write(*c);
    }
    //PCIInterface::new().test();
    // let uart = uart::UART::default();
    // //uart.init();
    // uart.write_string("Hello, world!\n");
    //draw a frame buffer if available
    // Initialize PCI subsystem
    hcf();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    // Print the panic message to the terminal emulator if available
    println!("Panic occurred: {}", _info);

    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}
