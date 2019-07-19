mod cpu;
mod cpu_bus;
mod nes;
mod ram;
mod rom;

use nes::Nes;

fn main() {
    let nes = Nes::load("../sample1/sample1.nes");
    nes.start();
}
