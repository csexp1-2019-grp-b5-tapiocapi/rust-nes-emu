#![allow(dead_code)]

use crate::cpu_bus::NMI_INT;
use crate::nes;
use crate::rom::CharacterRom;
use bitflags::bitflags;
use enum_primitive::*;

#[rustfmt::skip]
const PALETTE: [u8; 64*3] =
/*    [124,124,124,
    0,0,252,
    0,0,188,
    68,40,188,*/
    [0,0,0,
    85,85,85,
    170,170,170,
    255,255,255,
    148,0,132,
    168,0,32,
    168,16,0,
    136,20,0,
    80,48,0,
    0,120,0,
    0,104,0,
    0,88,0,
    0,64,88,
    0,0,0,
    0,0,0,
    0,0,0,
    188,188,188,
    0,120,248,
    0,88,248,
    104,68,252,
    216,0,204,
    228,0,88,
    248,56,0,
    228,92,16,
    172,124,0,
    0,184,0,
    0,168,0,
    0,168,68,
    0,136,136,
    0,0,0,
    0,0,0,
    0,0,0,
    248,248,248,
    60,188,252,
    104,136,252,
    152,120,248,
    248,120,248,
    248,88,152,
    248,120,88,
    252,160,68,
    248,184,0,
    184,248,24,
    88,216,84,
    88,248,152,
    0,232,216,
    120,120,120,
    0,0,0,
    0,0,0,
    252,252,252,
    164,228,252,
    184,184,248,
    216,184,248,
    248,184,248,
    248,164,192,
    240,208,176,
    252,224,168,
    248,216,120,
    216,248,120,
    184,248,184,
    184,248,216,
    0,252,252,
    248,216,248,
    0,0,0,
    0,0,0];

pub const SPRITE_WIDTH: usize = 8;
pub const SPRITE_HEIGHT: usize = 8;

#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    data: [[u8; SPRITE_WIDTH]; SPRITE_HEIGHT],
}

impl Sprite {
    fn new(chr: &[u8]) -> Sprite {
        let mut data = [[0u8; SPRITE_WIDTH]; SPRITE_HEIGHT];

        for i in 0..8 {
            for j in 0..8 {
                data[i][j] = ((chr[i] & (0b1000_0000 >> j)) != 0) as u8
                    | ((((chr[i + 8] & (0b1000_0000 >> j)) != 0) as u8) << 1);
            }
        }

        Sprite { data }
    }
}

fn write_sprite(mat: &mut opencv::core::Mat, sprite: &Sprite) {
    for j in 0..8 {
        for k in 0..8 {
            let npalette = sprite.data[j as usize][k as usize] as usize * 3;
            *mat.at_2d_mut(j, k).unwrap() = opencv::core::Vec3::from([
                PALETTE[npalette],
                PALETTE[npalette + 1],
                PALETTE[npalette + 2],
            ]);
        }
    }
}

struct Vram {
    pub mem: Vec<u8>,
    pub chr_rom: CharacterRom,
    chr_ram: Vec<u8>,
}

impl Vram {
    #[allow(dead_code)]
    const ADDREE_SIZE: usize = 0x4000;
    #[allow(dead_code)]
    const VRAM_SIZE: usize = 0x2000;
    const VRAM_START: usize = 0x2000;

    fn new(chr_rom: CharacterRom) -> Self {
        let chr_len = chr_rom.data.len();
        let vram = Self {
            mem: vec![0; Self::VRAM_SIZE],
            chr_rom,
            chr_ram: if chr_len == 0 {
                vec![0; 0x2000]
            } else {
                Vec::new()
            },
        };

        vram
    }

    fn reset(&mut self) {
        self.mem = vec![0; Self::VRAM_SIZE];
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_rom.data.len() == 0 {
                    self.chr_ram[addr as usize]
                } else {
                    self.chr_rom.data[addr as usize]
                }
            }
            /*0x0000..=0x0FFF => {
                /* pattern table 0 */
            }
            0x1000..=0x1FFF => {
                /* pattern table 1 */
            }*/
            0x2000..=0x2FFF => self.mem[addr as usize - Vram::VRAM_START],
            /*0x2000..=0x23BF => {
                /* name table 0 */
            }
            0x23C0..=0x23FF => {
                /* attr table 0 */
            }
            0x2400..=0x27BF => {
                /* name table 1 */
            }
            0x27C0..=0x27FF => {
                /* attr table 1 */
            }
            0x2800..=0x2BBF => {
                /* name table 2 */
            }
            0x2BC0..=0x2BFF => {
                /* attr table 2 */
            }
            0x2C00..=0x2FBF => {
                /* name table 3 */
            }
            0x2FC0..=0x2FFF => {
                /* attr table 3 */
            }*/
            0x3000..=0x3EFF => {
                /* mirror of 0x2000 ..= 0x2EFF */
                self.read(addr - 0x1000)
            }
            0x3F00..=0x3F0F => {
                /* background palette table */
                unimplemented!();
            }
            0x3F10..=0x3F1F => {
                /* sprite palette table */
                unimplemented!();
            }
            0x3F20..=0x3FFF => {
                /* mirror of 0x3F00-0x3F1F */
                panic!("VRAM: read: accessing mirroring area of palette tables");
            }
            _ => panic!("VRAM: Invalid read at 0x{:X}", addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        println!("VRAM: write 0x{:x} at 0x{:x}", data, addr);
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_rom.data.len() == 0 {
                    self.chr_ram[addr as usize] = data;
                } else {
                    self.chr_rom.data[addr as usize] = data;
                }
            }
            /*0x0000..=0x0FFF => {
                /* pattern table 0 */
            }
            0x1000..=0x1FFF => {
                /* pattern table 1 */
            }*/
            0x2000..=0x2FFF => {
                self.mem[addr as usize - Vram::VRAM_START] = data;
            }
            /*0x2000..=0x23BF => {
                /* name table 0 */
            }
            0x23C0..=0x23FF => {
                /* attr table 0 */
            }
            0x2400..=0x27BF => {
                /* name table 1 */
            }
            0x27C0..=0x27FF => {
                /* attr table 1 */
            }
            0x2800..=0x2BBF => {
                /* name table 2 */
            }
            0x2BC0..=0x2BFF => {
                /* attr table 2 */
            }
            0x2C00..=0x2FBF => {
                /* name table 3 */
            }
            0x2FC0..=0x2FFF => {
                /* attr table 3 */
            }*/
            0x3000..=0x3EFF => {
                /* mirror of 0x2000 ..= 0x2EFF */
                self.write(addr - 0x1000, data);
            }
            0x3F00..=0x3F0F => {
                /* background palette table */
                println!("VRAM: write: bg palette table");
            }
            0x3F10..=0x3F1F => {
                /* sprite palette table */
                println!("VRAM: write: sprite palette table");
            }
            0x3F20..=0x3FFF => {
                /* mirror of 0x3F00-0x3F1F */
                panic!("VRAM: write: accessing mirroring area of palette tables");
            }
            _ => panic!("VRAM: Invalid write at 0x{:X}", addr),
        }
    }
}

enum_from_primitive! {
    #[doc = "PPU memory mapped register type"]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum RegType {
        PPUCTRL     = 0,
        PPUMASK     = 1,
        PPUSTATUS   = 2,
        OAMADDR     = 3,
        OAMDATA     = 4,
        PPUSCROLL   = 5,
        PPUADDR     = 6,
        PPUDATA     = 7,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PpuPtrState {
    High,
    Low,
}

impl PpuPtrState {
    fn toggle(&mut self) {
        match *self {
            PpuPtrState::High => *self = PpuPtrState::Low,
            PpuPtrState::Low => *self = PpuPtrState::High,
        }
    }
}

struct PpuPtr {
    addr: u16,
    state: PpuPtrState,
}

impl Default for PpuPtr {
    fn default() -> PpuPtr {
        PpuPtr {
            addr: 0,
            state: PpuPtrState::High,
        }
    }
}

impl PpuPtr {
    pub fn new() -> PpuPtr {
        Default::default()
    }

    pub fn get(&self) -> u16 {
        self.addr
    }

    pub fn get_and_inc(&mut self) -> u16 {
        let addr = self.addr;
        self.addr += 1;
        addr
    }

    pub fn write(&mut self, addr: u8) {
        match self.state {
            PpuPtrState::High => self.addr = (addr as u16) << 8 | (self.addr << 8) >> 8,
            PpuPtrState::Low => self.addr = (addr as u16) | (self.addr >> 8) << 8,
        }
        self.state.toggle();
    }

    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

struct PpuCtrlReg {
    flags: u8,
}

impl PpuCtrlReg {
    fn new() -> PpuCtrlReg {
        Self::from_u8(0)
    }

    fn from_u8(flags: u8) -> PpuCtrlReg {
        PpuCtrlReg { flags }
    }

    fn set(&mut self, flags: u8) {
        self.flags = flags;
    }

    /*
     * Base nametable address
     * (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
     */
    fn base_nametable_addr(&self) -> u16 {
        match self.flags & 0b0000_0011 {
            0b0000_0000 => 0x2000,
            0b0000_0001 => 0x2400,
            0b0000_0010 => 0x2800,
            0b0000_0011 => 0x2C00,
            _ => unreachable!(),
        }
    }

    /*
     * VRAM address increment per CPU read/write of PPUDATA
     * (0: add 1, going across; 1: add 32, going down)
     *
     * Return: true if incrementation required
     */
    fn vram_addr_increment(&self) -> bool {
        (self.flags & 0b0000_0100) == 0
    }

    /*
     * Sprite pattern table address for 8x8 sprites
     * (0: $0000; 1: $1000; ignored in 8x16 mode)
     */
    fn sprite_pattern_table_addr(&self) -> u16 {
        if (self.flags & 0b0000_1000) == 0 {
            0x0000
        } else {
            0x1000
        }
    }

    /*
     * Background pattern table address (0: $0000; 1: $1000)
     */
    fn bg_pattern_table_addr(&self) -> u16 {
        if (self.flags & 0b0001_0000) == 0 {
            0x0000
        } else {
            0x1000
        }
    }

    /*
     * Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
     */
    fn sprite_size(&self) -> bool {
        (self.flags & 0b0010_0000) != 0
    }

    /*
     * PPU master/slave select
     * (0: read backdrop from EXT pins; 1: output color on EXT pins)
     */
    fn ppu_master_slave(&self) -> bool {
        (self.flags & 0b0100_0000) != 0
    }

    /*
     * Generate an NMI at the start of the
     * vertical blanking interval (0: off; 1: on)
     */
    fn generate_nmi(&self) -> bool {
        (self.flags & 0b1000_0000) != 0
    }
}

bitflags! {
    struct PpuMask: u8 {
        const GRAYSCALE             = 0b0000_0001;
        const SHOW_BG_LEFTMOST      = 0b0000_0010;
        const SHOW_SPRITES_LEFTMOST = 0b0000_0100;
        const SHOW_BG               = 0b0000_1000;
        const SHOW_SPRITES          = 0b0001_0000;
        const SHOW_ALL              = 0b0001_1110;
        const EMPHASIZE_RED         = 0b0010_0000;
        const EMPHASIZE_GREEN       = 0b0100_0000;
        const EMPHASIZE_BLUE        = 0b1000_0000;
    }
}

#[derive(Debug)]
struct OamEntry {
    y: u8,
    tile: u8,
    attr: u8,
    x: u8,
}

impl OamEntry {
    fn new(entry: &[u8]) -> OamEntry {
        assert_eq!(entry.len(), 4);
        OamEntry {
            y: entry[0],
            tile: entry[1],
            attr: entry[2],
            x: entry[3],
        }
    }
}

pub struct Ppu {
    ctrlreg: PpuCtrlReg,
    mask: PpuMask,
    ppuptr: PpuPtr,
    oamptr: u8,
    sprite_ram: Vec<u8>,
    vbuf: opencv::core::Mat,
    vram: Vram,
    last_written: u8,
}

impl Ppu {
    pub fn new(chr_rom: CharacterRom) -> Ppu {
        Ppu {
            ctrlreg: PpuCtrlReg::new(),
            /* for the lazy ROMs not initializing PPUMASK */
            mask: PpuMask::SHOW_ALL,
            ppuptr: PpuPtr::new(),
            oamptr: 0,
            sprite_ram: vec![0; 256],
            vbuf: unsafe {
                opencv::core::Mat::new_rows_cols(240, 256, opencv::core::CV_8UC3).unwrap()
            },
            vram: Vram::new(chr_rom),
            last_written: 0,
        }
    }

    fn show(&mut self) {
        if !self.mask.intersects(PpuMask::SHOW_ALL) {
            return;
        }

        self.update_whole_vbuf();

        let mut screen = opencv::core::Mat::new().unwrap();

        opencv::imgproc::resize(
            &self.vbuf,
            &mut screen,
            opencv::core::Size::new(1024, 1024),
            0.0,
            0.0,
            0,
        )
        .unwrap();

        opencv::highgui::imshow(nes::CV_WINDOW_TITLE, &screen).unwrap();
    }

    fn update_whole_vbuf(&mut self) {
        let ntbase = self.ctrlreg.base_nametable_addr() as usize - Vram::VRAM_START;
        let bg_ptrn_tab_addr = self.ctrlreg.bg_pattern_table_addr();
        let sprite_ptrn_tab_addr = self.ctrlreg.sprite_pattern_table_addr();

        for (i, sprite_index) in (&self.vram.mem[ntbase..(ntbase + 960)]).iter().enumerate() {
            //println!(
            //    "get_mat: i={} sprite={}: ({}, {})",
            //    i,
            //    sprite,
            //    ((i % 32) * SPRITE_WIDTH) as i32,
            //    ((i / 32) * SPRITE_HEIGHT) as i32
            //);

            let mut roi = opencv::core::Mat::roi(
                &self.vbuf,
                opencv::core::Rect::new(
                    ((i % 32) * SPRITE_WIDTH) as i32,
                    ((i / 32) * SPRITE_HEIGHT) as i32,
                    SPRITE_WIDTH as i32,
                    SPRITE_HEIGHT as i32,
                ),
            )
            .unwrap();

            let bg_addr = bg_ptrn_tab_addr + (*sprite_index as u16) * 16;
            let sprite: Vec<u8> = (bg_addr..(bg_addr + 16))
                .map(|addr| self.vram.read(addr))
                .collect();
            assert_eq!(sprite.len(), 16);

            write_sprite(&mut roi, &Sprite::new(&sprite));
        }

        for i in (0..255).step_by(4) {
            let entry = OamEntry::new(&self.sprite_ram[i..(i + 4)]);
            if entry.y == 0 {
                continue;
            }

            let mut roi = opencv::core::Mat::roi(
                &self.vbuf,
                opencv::core::Rect::new(
                    entry.x as i32 * SPRITE_WIDTH as i32,
                    entry.y as i32 * SPRITE_HEIGHT as i32,
                    SPRITE_WIDTH as i32,
                    SPRITE_HEIGHT as i32,
                ),
            )
            .unwrap();

            let sprite_addr = sprite_ptrn_tab_addr + (entry.tile as u16) * 16;
            let sprite: Vec<u8> = (sprite_addr..(sprite_addr + 16))
                .map(|addr| self.vram.read(addr))
                .collect();
            assert_eq!(sprite.len(), 16);

            write_sprite(&mut roi, &Sprite::new(&sprite));
        }
    }

    pub fn read(&mut self, regtype: RegType) -> u8 {
        println!("PPU: read: {:?}", regtype);
        match regtype {
            RegType::PPUSTATUS => {
                /* our vblank always ready for now :) */
                0b1000_0000 | (self.last_written & 0b0001_1111)
            }
            RegType::OAMDATA => {
                let addr = self.oamptr;
                self.oamptr = (self.oamptr as u16 + 1) as u8;

                self.sprite_ram[addr as usize]
            }
            RegType::PPUDATA => {
                let addr = if self.ctrlreg.vram_addr_increment() {
                    self.ppuptr.get_and_inc()
                } else {
                    self.ppuptr.get()
                };

                self.vram.read(addr)
            }
            _ => panic!("PPU: Trying to read write-only register: {:?}", regtype),
        }
    }

    pub fn write(&mut self, regtype: RegType, data: u8) {
        println!("PPU: write: {:?}: {:x}", regtype, data);
        self.last_written = data;

        if self.ctrlreg.generate_nmi() {
            *NMI_INT.borrow_mut() = true;
        }

        match regtype {
            RegType::PPUCTRL => {
                self.ctrlreg.set(data);
                println!(
                    "PPUCTRL: write: sprite pattern table addr: 0x{:x}",
                    self.ctrlreg.sprite_pattern_table_addr()
                );
                println!(
                    "PPUCTRL: write: bg pattern table addr: 0x{:x}",
                    self.ctrlreg.bg_pattern_table_addr()
                );
            }
            RegType::PPUMASK => {
                /* use unwrap() cuz all bits correspond to flags */
                self.mask = PpuMask::from_bits(data).unwrap();
                self.show();
            }
            RegType::OAMADDR => {
                self.oamptr = data;
            }
            RegType::OAMDATA => {
                let addr = self.oamptr;
                self.oamptr = (self.oamptr as u16 + 1) as u8;

                let addr = addr as usize;
                self.sprite_ram[addr] = data;

                let nsprite = addr / 4;
                let sprite_begin = nsprite * 4;

                println!(
                    "OAMDATA: write: {:?}",
                    OamEntry::new(&self.sprite_ram[sprite_begin..(sprite_begin + 4)])
                );

                self.show();
            }
            RegType::PPUSCROLL => {
                //unimplemented!();
            }
            RegType::PPUADDR => {
                self.ppuptr.write(data);
            }
            RegType::PPUDATA => {
                let addr = if self.ctrlreg.vram_addr_increment() {
                    self.ppuptr.get_and_inc()
                } else {
                    self.ppuptr.get()
                };

                self.vram.write(addr, data);
                self.show();
            }
            _ => panic!("PPU: Trying to write read-only register: {:?}", regtype),
        }
    }
}

#[test]
fn sprite_test() {
    use crate::rom;
    use opencv::prelude::*;

    let buffer = std::fs::read("sample1/sample1.nes").unwrap();
    //let buffer =
    //    std::fs::read("~/Documents/fc3_full_win32_20190611/fc3_full_win32_20190611/marioBros3.nes")
    //        .unwrap();

    let (_, chr_rom) = rom::load(buffer).unwrap();
    println!("chr rom size: {}", chr_rom.data.len());

    let ppu = Ppu::new(chr_rom);

    let title = "Sprite";

    let count = 100i32;
    let length = 24;

    let sprites_img = unsafe {
        opencv::core::Mat::new_rows_cols(length * count, length * count, opencv::core::CV_8UC3)
            .unwrap()
    };

    'outer: for i in 0..count {
        for j in 0..count {
            let index = (j + i * count) as usize;
            if index >= ppu.vram.sprites.len() {
                break 'outer;
            }

            let sprite = ppu.vram.sprites[index];

            let mut img =
                unsafe { opencv::core::Mat::new_rows_cols(8, 8, opencv::core::CV_8UC3).unwrap() };
            for l in 0..8 {
                for m in 0..8 {
                    *img.at_2d_mut(l, m).unwrap() = sprite.data[l as usize][m as usize] * 63;
                }
            }

            let mut scaled = opencv::core::Mat::new().unwrap();
            opencv::imgproc::resize(
                &img,
                &mut scaled,
                opencv::core::Size::new(length, length),
                0.0,
                0.0,
                0,
            )
            .unwrap();

            //println!("({}, {}) {}x{}", j * length, i * length, length, length);
            let mut roi = opencv::core::Mat::roi(
                &sprites_img,
                opencv::core::Rect::new(j * length, i * length, length, length),
            )
            .unwrap();
            scaled.copy_to(&mut roi).unwrap();
        }
    }

    opencv::highgui::imshow(title, &sprites_img).unwrap();
    opencv::highgui::wait_key(0).unwrap();

    opencv::imgcodecs::imwrite("./sprites.png", &sprites_img, &Vector::new()).unwrap();
    /*
    for sprite in ppu.sprites {
        println!("Showing");
        let mut img =
            unsafe { opencv::core::Mat::new_rows_cols(8, 8, opencv::core::CV_8UC1).unwrap() };
        for i in 0..8 {
            for j in 0..8 {
                *img.at_2d_mut(i, j).unwrap() = sprite.data[i as usize][j as usize] * 63;
            }
        }

        let mut scaled = opencv::core::Mat::new().unwrap();
        opencv::imgproc::resize(
            &img,
            &mut scaled,
            opencv::core::Size::new(32, 32),
            0.0,
            0.0,
            0,
        ).unwrap();

        opencv::highgui::imshow(title, &scaled);
        opencv::highgui::wait_key(0).unwrap();
    }*/
}

#[test]
fn ppu_ctrl_reg_test() {
    let ctrlreg1 = PpuCtrlReg::new();
    assert_eq!(ctrlreg1.base_nametable_addr(), 0x2000);
    assert_eq!(ctrlreg1.vram_addr_increment(), true);
    assert_eq!(ctrlreg1.sprite_pattern_table_addr(), 0x0000);
    assert_eq!(ctrlreg1.bg_pattern_table_addr(), 0x0000);
    assert_eq!(ctrlreg1.sprite_size(), false);
    assert_eq!(ctrlreg1.ppu_master_slave(), false);
    assert_eq!(ctrlreg1.generate_nmi(), false);

    let ctrlreg2 = PpuCtrlReg::from_u8(0x8);
    assert_eq!(ctrlreg2.base_nametable_addr(), 0x2000);
    assert_eq!(ctrlreg2.vram_addr_increment(), true);
    assert_eq!(ctrlreg2.sprite_pattern_table_addr(), 0x1000);
    assert_eq!(ctrlreg2.bg_pattern_table_addr(), 0x0000);
    assert_eq!(ctrlreg2.sprite_size(), false);
    assert_eq!(ctrlreg2.ppu_master_slave(), false);
    assert_eq!(ctrlreg2.generate_nmi(), false);
}

#[test]
fn ppu_mask_test() {
    let mask1 = PpuMask::from_bits(0x1e).unwrap();
    assert!(!mask1.contains(PpuMask::GRAYSCALE));
    assert!(mask1.contains(PpuMask::SHOW_BG_LEFTMOST));
    assert!(mask1.contains(PpuMask::SHOW_SPRITES_LEFTMOST));
    assert!(mask1.contains(PpuMask::SHOW_BG));
    assert!(mask1.contains(PpuMask::SHOW_SPRITES));
    assert!(mask1.contains(PpuMask::SHOW_ALL));
    assert!(!mask1.contains(PpuMask::EMPHASIZE_RED));
    assert!(!mask1.contains(PpuMask::EMPHASIZE_GREEN));
    assert!(!mask1.contains(PpuMask::EMPHASIZE_BLUE));
}
