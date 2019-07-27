use num_traits::FromPrimitive;

use crate::ppu;
use crate::ram;
use crate::rom;

pub struct CpuBus {
    wram: ram::Ram,
    prog_rom: rom::ProgramRom,
    ppu: ppu::Ppu,
}

impl CpuBus {
    pub fn new(wram: ram::Ram, prog_rom: rom::ProgramRom, ppu: ppu::Ppu) -> CpuBus {
        CpuBus {
            wram,
            prog_rom,
            ppu,
        }
    }

    pub fn read_by_cpu(&self, addr: u16) -> u8 {
        //println!("read_by_cpu {:x}", addr);
        if addr < 0x0800 {
            // WRAM
            self.wram.read(addr)
        } else if addr < 0x2000 {
            // WRAM Mirror
            self.wram.read(addr - 0x800)
        } else if addr < 0x2008 {
            // PPU Register
            self.ppu
                .read(ppu::RegType::from_u16(addr - 0x2000).unwrap())
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
            self.prog_rom.read(addr - 0x8000)
        } else {
            //0xC000 ~ 0xFFFF   // PRG-ROM
            let base_addr = if self.prog_rom.data.len() == 0x4000 {
                0xC000
            } else {
                0x8000
            };

            self.prog_rom.read(addr - base_addr)
        }
    }

    pub fn write_by_cpu(&mut self, addr: u16, data: u8) {
        if addr < 0x800 {
            self.wram.write(addr, data);
        } else if addr < 0x2000 {
            self.wram.write(addr - 0x800, data);
        } else if addr < 0x2008 {
            self.ppu
                .write(ppu::RegType::from_u16(addr - 0x2000).unwrap(), data)
        } else if addr < 0x4020 && addr >= 0x4000 {
            //0x4014 -> dma
            //0x4016 -> joypad1
            //0x4017 -> joypad2
            //others -> apu
        }
    }
}
