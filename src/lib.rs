mod cpu;
mod cpu_bus;
mod nes;
//mod ppu;
mod ram;
mod rom;

use wasm_bindgen::prelude::*;

use nes::Nes;
use std::io;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

fn nes_main(file_name: &str/*args: Vec<String>*/) -> io::Result<i32>{
    //if args.len() < 2 {
    //    println!("Usage: {} NES", args[0]);
    //    return Ok(-1);
    //}
    let mut nes = Nes::load(&file_name)?;
    nes.start();

    Ok(0)
}

#[wasm_bindgen]
pub fn start(file_name: &str) {
    //let args = std::env::args().collect();
    let exit_code = nes_main(file_name).unwrap_or_else(|e| {
        eprintln!("FATAL: {}", e);
        -1
    });
}
