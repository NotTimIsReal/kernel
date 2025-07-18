use crate::{HHDM_REQUEST, hw::Pointer, println};

#[derive(Debug, Clone, Copy)]
struct Device {
    pub bus: u8,
    pub device: u8,
    pub functions: [u8; 8],
    pub vendor_id: u16,
    pub device_id: u16,
}
impl Default for Device {
    fn default() -> Self {
        Device {
            bus: 0,
            device: 0,
            functions: [0; 8],
            vendor_id: 0xFFFF, // 0xFFFF indicates no device present
            device_id: 0xFFFF,
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct Bus {
    pub bus_number: u8,
    pub devices: [Device; 32], // 32 devices per bus
}
impl Default for Bus {
    fn default() -> Self {
        Bus {
            bus_number: 0,
            devices: [Device::default(); 32],
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct HeaderType {
    pub header_type: u8,
    pub multi_function: bool,
}

pub struct PCIInterface {
    buses: [Bus; 256],
}

impl PCIInterface {
    pub fn new() -> Self {
        PCIInterface {
            buses: [Bus::default(); 256],
        }
    }
    fn read_config_word(&self, bus: u8, device: u8, function: u8, offset: u8) -> u16 {
        let address = 0x8000_0000
            | ((bus as u32) << 16)
            | ((device as u32) << 11)
            | ((function as u32) << 8)
            | (offset as u32);
        //println!("Reading PCI config word at address: {:#X}", address);
        Pointer::new(address as usize).read() as u16
    }
    fn check_vendor(&self, bus: u8, device: u8) -> bool {
        let vendor_id = self.read_config_word(bus, device, 0, 0);
        vendor_id != 0xFFFF // 0xFFFF indicates no device present
    }
    fn get_header_type(&self, bus: u8, device: u8) -> HeaderType {
        let header_type = self.read_config_word(bus, device, 0, (0x0E) as u8) as u8;
        HeaderType {
            header_type,
            multi_function: (header_type & 0x80) != 0,
        }
    }
    fn get_vendor_id(&self, bus: u8, device: u8) -> u16 {
        self.read_config_word(bus, device, 0, 0)
    }
    //returns (device_present, multi_function)
    fn check_device(&self, bus: u8, device: u8) -> (bool, bool) {
        let vendor_id = self.get_vendor_id(bus, device);
        if vendor_id == 0xFFFF {
            return (false, false); // No device present
        }
        let header_type = self.get_header_type(bus, device);
        (true, header_type.multi_function)
    }
    pub fn test(&self) {
        for i in 0..256 {
            let bus = i as u8;
            for j in 0..32 {
                let device = j as u8;
                if self.check_vendor(bus, device) {
                    let header_type = self.get_header_type(bus, device);

                    // println!(
                    //     "Bus: {}, Device: {}, Header Type: {}, Multi-function: {}",
                    //     bus, device, header_type.header_type, header_type.multi_function
                    // );
                    // Read vendor and device ID
                    let vendor_id = self.read_config_word(bus, device, 0, 0);
                    if vendor_id == 0xFFFF {
                        continue; // No device present
                    }
                    let device_id = self.read_config_word(bus, device, 0, 2);
                    println!("Vendor ID: {:#X}, Device ID: {:#X}", vendor_id, device_id);
                }
            }
        }
        // println!("{:?}", self.get_header_type(0, 0));
    }
}
