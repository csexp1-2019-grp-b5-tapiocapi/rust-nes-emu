mod cpu;
mod cpu_bus;
mod nes;
mod wram;
mod rom;

use nes::Nes;
use std::io;

fn nes_main() -> io::Result<i32> {
    let mut nes = Nes::load("../sample1/sample1.nes")?;
    nes.start();

    Ok(0)
}

fn main() {
    let exit_code = nes_main().unwrap_or_else(|e| {
        eprintln!("FATAL: {}", e);
        -1
    });

    std::process::exit(exit_code);
}
