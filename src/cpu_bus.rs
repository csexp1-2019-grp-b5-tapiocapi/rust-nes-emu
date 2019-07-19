use crate::ram;
use crate::rom;

pub struct CpuBus {
    ram: ram::Ram,
    rom: rom::Rom,
}
impl CpuBus {
    pub fn init(nes_ram: ram::Ram, nes_rom: rom::Rom) -> CpuBus {
        CpuBus {
            ram: nes_ram,
            rom: nes_rom,
        }
    }

    pub fn read_by_cpu(&self, addr: u16) -> u8 {
        println!("read_by_cpu {:x}", addr);
        if addr < 0x0800 {
            // WRAM
            self.ram.read(addr)
        } else if addr < 0x2000 {
            // WRAM Mirror
            self.ram.read(addr - 0x800)
        } else if addr < 0x2008 {
            // PPU Register
            0
        } else if addr < 0x4000 {
            // PPU Mirror
            0
        } else if addr == 0x4016 {
            // Joypad P1
            0
        } else if addr == 0x4017 {
            // Joypad P2
            0
        } else if addr < 0x6000 {
            // Extended ROM
            0
        } else if addr < 0x8000 {
            // Extended RAM
            0
        } else if addr < 0xC000 {
            // PRG-ROM
            self.rom.read(addr - 0x8000)
        } else {
            //0xC000 ~ 0xFFFF   // PRG-ROM
            self.rom.read(addr - 0x8000)
        }
    }

    fn write_by_cpu(&self, addr: u16, data: u8) {}
}
