mod cpu;
mod cpu_bus;
mod nes;
mod ppu;
mod ram;
mod rom;

use nes::Nes;
use std::io;

fn nes_main(args: Vec<String>) -> io::Result<i32> {
    if args.len() < 2 {
        println!("Usage: {} NES", args[0]);
        return Ok(-1);
    }

    let mut nes = Nes::load(&args[1])?;
    nes.start();

    Ok(0)
}

fn main() {
    let args = std::env::args().collect();
    let exit_code = nes_main(args).unwrap_or_else(|e| {
        eprintln!("FATAL: {}", e);
        -1
    });

    std::process::exit(exit_code);
}
