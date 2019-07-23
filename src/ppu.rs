use crate::rom::CharacterRom;
use crate::wram::Wram;

#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    data: [[u8; 8]; 8],
}

impl Sprite {
    fn new(chr: &[u8]) -> Sprite {
        let mut data = [[0u8; 8]; 8];

        for i in 0..16 {
            for j in 0..8 {
                let k = 7 - j;
                data[i % 8][j] += (chr[i] & (1 << k)) >> k;
            }
        }

        Sprite { data }
    }
}

pub struct Ppu {
    sprites: Vec<Sprite>,
    vram: Wram,
}

impl Ppu {
    pub fn new(chr_rom: &CharacterRom) -> Ppu {
        Ppu {
            sprites: chr_rom.data.chunks(16).map(Sprite::new).collect(),
            vram: Wram::new(2048),
        }
    }

    pub fn show(sprite: &Sprite) {
        for row in &sprite.data {
            for pixel in row {
                print!("{}", if *pixel > 0 { "." } else { " " });
            }
            println!();
        }
    }

    pub fn show_all(&self) {
        for sprite in &self.sprites {
            Ppu::show(sprite);
            println!();
            println!();
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            2 => {
                /* PPU state */
                unimplemented!();
            }
            4 => {
                /* OAM data */
                unimplemented!();
            }
            7 => {
                /* PPU data */
                unimplemented!();
            }
            _ => {
                unimplemented!();
            }
        }
    }

    pub fn write(&self, addr: u16, data: u8) {
        match addr {
            0 => {
                /* PPU CTL */
                unimplemented!();
            }
            1 => {
                /* PPU MASK */
                unimplemented!();
            }
            3 => {
                /* OAM ADDR */
                unimplemented!();
            }
            4 => {
                /* OAM DATA */
                unimplemented!();
            }
            5 => {
                /* PPU SCROLL */
                unimplemented!();
            }
            6 => {
                /* PPU ADDR */
                unimplemented!();
            }
            7 => {
                /* PPU DATA */
                unimplemented!();
            }
            _ => {
                unimplemented!();
            }
        }
    }
}

#[test]
fn sprite_test() {
    use crate::rom;

    let buffer = std::fs::read("sample1/sample1.nes").unwrap();
    //let buffer = std::fs::read("/home/devm33/Documents/fc3_full_win32_20190611/fc3_full_win32_20190611/marioBros3.nes").unwrap();

    let (_, chr_rom) = rom::load(buffer);
    let ppu = Ppu::new(&chr_rom);

    println!("chr rom size: {}", chr_rom.data.len());
    //ppu.show_all();

    let title = "Sprite";
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
            opencv::core::Size::new(512, 512),
            0.0,
            0.0,
            0,
        ).unwrap();

        opencv::highgui::imshow(title, &scaled);
        opencv::highgui::wait_key(0).unwrap();
    }
}
