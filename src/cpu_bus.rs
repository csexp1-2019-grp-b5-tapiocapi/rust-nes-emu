fn read_by_cpu(addr: u16) {
    if addr < 0x0800 {          // WRAM
    } else if addr < 0x2000 {   // WRAM Mirror
    } else if addr < 0x2008 {   // PPU Register
    } else if addr < 0x4000 {   // PPU Mirror
    } else if addr == 0x4016 {  // Joypad P1
    } else if addr == 0x4017 {  // Joypad P2
    } else if addr < 0x6000 {   // Extended ROM
    } else if addr < 0x8000 {   // Extended RAM
    } else if addr < 0xC000 {   // PRG-ROM
    } else {//0xC000 ~ 0xFFFF   // PRG-ROM
    }
}

fn write_by_cpu() {
}
