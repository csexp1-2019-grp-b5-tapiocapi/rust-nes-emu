#![allow(dead_code)]

use crate::nes;
use crate::rom::CharacterRom;
use enum_primitive::*;
use opencv::prelude::*;

pub const SPRITE_WIDTH: usize = 8;
pub const SPRITE_HEIGHT: usize = 8;

#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    data: [[u8; SPRITE_WIDTH]; SPRITE_HEIGHT],
}

impl Sprite {
    fn new(chr: &[u8]) -> Sprite {
        let mut data = [[0u8; SPRITE_WIDTH]; SPRITE_HEIGHT];

        for i in 0..16 {
            for j in 0..8 {
                let k = 7 - j;
                data[i % 8][j] += (chr[i] & (1 << k)) >> k;
            }
        }

        Sprite { data }
    }
}

struct Vram {
    mem: Vec<u8>,
    vbuf: opencv::core::Mat,
    sprites: Vec<Sprite>,
}

impl Vram {
    #[allow(dead_code)]
    const ADDREE_SIZE: usize = 0x4000;
    #[allow(dead_code)]
    const VRAM_SIZE: usize = 0x2000;
    const VRAM_START: usize = 0x2000;

    fn new(chr_rom: &CharacterRom) -> Self {
        let vram = Self {
            mem: vec![0; Self::VRAM_SIZE],
            vbuf: unsafe {
                opencv::core::Mat::new_rows_cols(240, 256, opencv::core::CV_8UC1).unwrap()
            },
            sprites: chr_rom.data.chunks(16).map(Sprite::new).collect(),
        };

        vram.show();
        vram
    }

    fn reset(&mut self) {
        self.mem = vec![0; Self::VRAM_SIZE];
    }

    fn show(&self) {
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

        /*
         * waiting for 1 millisec sometimes causes incomplete drawing
         */
        opencv::highgui::wait_key(10).unwrap();
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x0FFF => {
                /* pattern table 0 */
                unimplemented!();
            }
            0x1000..=0x1FFF => {
                /* pattern table 1 */
                unimplemented!();
            }
            0x2000..=0x23BF => {
                /* name table 0 */
                self.mem[addr as usize - Vram::VRAM_START]
            }
            0x23C0..=0x23FF => {
                /* attr table 0 */
                unimplemented!();
            }
            0x2400..=0x27BF => {
                /* name table 1 */
                unimplemented!();
            }
            0x27C0..=0x27FF => {
                /* attr table 1 */
                unimplemented!();
            }
            0x2800..=0x2BBF => {
                /* name table 2 */
                unimplemented!();
            }
            0x2BC0..=0x2BFF => {
                /* attr table 2 */
                unimplemented!();
            }
            0x2C00..=0x2FBF => {
                /* name table 3 */
                unimplemented!();
            }
            0x2FC0..=0x2FFF => {
                /* attr table 3 */
                unimplemented!();
            }
            0x3000..=0x3EFF => {
                /* mirror of 0x2000 ..= 0x2EFF */
                self.read(addr - 0x1000)
            }
            _ => panic!("Invalid read at 0x{:X}", addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        println!("VRAM: write 0x{:x} at 0x{:x}", data, addr);
        match addr {
            0x0000..=0x0FFF => {
                /* pattern table 0 */
                //unimplemented!();
            }
            0x1000..=0x1FFF => {
                /* pattern table 1 */
                unimplemented!();
            }
            0x2000..=0x23BF => {
                /* name table 0 */
                let pos = addr as usize - Vram::VRAM_START;
                println!("VRAM: SPRITES: drawing: {}", data as char);

                self.mem[pos] = data;
                self.update_vbuf(pos as u16);
                self.show();
            }
            0x23C0..=0x23FF => {
                /* attr table 0 */
                unimplemented!();
            }
            0x2400..=0x27BF => {
                /* name table 1 */
                unimplemented!();
            }
            0x27C0..=0x27FF => {
                /* attr table 1 */
                unimplemented!();
            }
            0x2800..=0x2BBF => {
                /* name table 2 */
                unimplemented!();
            }
            0x2BC0..=0x2BFF => {
                /* attr table 2 */
                unimplemented!();
            }
            0x2C00..=0x2FBF => {
                /* name table 3 */
                unimplemented!();
            }
            0x2FC0..=0x2FFF => {
                /* attr table 3 */
                //unimplemented!();
            }
            0x3000..=0x3EFF => {
                /* mirror of 0x2000 ..= 0x2EFF */
                self.write(addr - 0x1000, data);
            }
            0x3F00..=0x3F0F => {
                /* background palette */
                //unimplemented!();
            }
            0x3F10..=0x3F1F => {
                /* sprite palette */
                unimplemented!();
            }
            0x3F20..=0x3FFF => {
                /* mirror of 0x3F00-0x3F1F */
                unimplemented!();
            }
            _ => panic!("Invalid read at 0x{:X}", addr),
        }
    }

    fn write_sprite(mat: &mut opencv::core::Mat, sprites: &[Sprite], sprite: usize) {
        for j in 0..8 {
            for k in 0..8 {
                *mat.at_2d_mut(j, k).unwrap() = sprites[sprite].data[j as usize][k as usize] * 63;
            }
        }
    }

    fn update_vbuf(&mut self, addr: u16) {
        let addr = addr as usize;

        let mut sprite_roi = opencv::core::Mat::roi(
            &self.vbuf,
            opencv::core::Rect::new(
                ((addr % 32) * SPRITE_WIDTH) as i32,
                ((addr / 32) * SPRITE_HEIGHT) as i32,
                SPRITE_WIDTH as i32,
                SPRITE_HEIGHT as i32,
            ),
        )
        .unwrap();

        let sprite = self.mem[addr] as usize;
        Self::write_sprite(&mut sprite_roi, &self.sprites, sprite);
    }

    fn update_whole_vbuf(&mut self) {
        for (i, sprite) in (&self.mem[0..960]).iter().enumerate() {
            println!(
                "get_mat: i={} sprite={}: ({}, {})",
                i,
                sprite,
                ((i % 32) * SPRITE_WIDTH) as i32,
                ((i / 32) * SPRITE_HEIGHT) as i32
            );

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

            Self::write_sprite(&mut roi, &self.sprites, *sprite as usize);
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

pub struct Ppu {
    ppuptr: PpuPtr,
    vram: Vram,
}

impl Ppu {
    pub fn new(chr_rom: &CharacterRom) -> Ppu {
        Ppu {
            ppuptr: PpuPtr::new(),
            vram: Vram::new(chr_rom),
        }
    }

    pub fn read(&self, regtype: RegType) -> u8 {
        println!("PPU: read: {:?}", regtype);
        match regtype {
            RegType::PPUSTATUS => {
                unimplemented!();
            }
            RegType::OAMDATA => {
                unimplemented!();
            }
            RegType::PPUDATA => {
                unimplemented!();
            }
            _ => panic!("Trying to read write-only address: {:?}", regtype),
        }
    }

    pub fn write(&mut self, regtype: RegType, data: u8) {
        println!("PPU: write: {:?}: {:x}", regtype, data);
        match regtype {
            RegType::PPUCTRL => {
                //unimplemented!();
            }
            RegType::PPUMASK => {
                //unimplemented!();
            }
            RegType::OAMADDR => {
                unimplemented!();
            }
            RegType::OAMDATA => {
                unimplemented!();
            }
            RegType::PPUSCROLL => {
                //unimplemented!();
            }
            RegType::PPUADDR => {
                self.ppuptr.write(data);
            }
            RegType::PPUDATA => {
                self.vram.write(self.ppuptr.get_and_inc(), data);
            }
            _ => panic!("Trying to write read-only address: {:?}", regtype),
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

    let (_, chr_rom) = rom::load(buffer);
    let ppu = Ppu::new(&chr_rom);

    println!("chr rom size: {}", chr_rom.data.len());

    let title = "Sprite";

    let count = 100i32;
    let length = 24;

    let sprites_img = unsafe {
        opencv::core::Mat::new_rows_cols(length * count, length * count, opencv::core::CV_8UC1)
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
                unsafe { opencv::core::Mat::new_rows_cols(8, 8, opencv::core::CV_8UC1).unwrap() };
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
            scaled.copy_to(&mut roi);
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
