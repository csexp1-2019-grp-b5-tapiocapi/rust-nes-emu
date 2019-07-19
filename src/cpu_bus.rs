use crate::ram;
use crate::rom;

pub struct CpuBus {
    ram: ram::Ram,
    rom: rom::Rom
}

pub fn init(nes_ram: ram::Ram, nes_rom: rom::Rom) -> CpuBus {
    let bus = CpuBus {
        ram: nes_ram,
        rom: nes_rom,
    };
    return bus;
}

impl CpuBus {
    pub fn read_by_cpu(self, addr: u16) -> i8 {
        if addr < 0x0800 {          // WRAM
            &self.ram.read(addr);
        } else if addr < 0x2000 {   // WRAM Mirror
            &self.ram.read(addr - 0x800);
        } else if addr < 0x2008 {   // PPU Register
        } else if addr < 0x4000 {   // PPU Mirror
        } else if addr == 0x4016 {  // Joypad P1
        } else if addr == 0x4017 {  // Joypad P2
        } else if addr < 0x6000 {   // Extended ROM
        } else if addr < 0x8000 {   // Extended RAM
        } else if addr < 0xC000 {   // PRG-ROM
            return self.read_by_cpu(addr - 0x8000);
        } else {//0xC000 ~ 0xFFFF   // PRG-ROM
            return self.read_by_cpu(addr - 0x8000);
        }
        return 0;
    }

    fn write_by_cpu(&self, addr: u16, data: u8) {
    }
}
