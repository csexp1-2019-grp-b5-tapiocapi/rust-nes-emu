mod nes;
mod cpu;
mod cpu_bus;
mod ram;
mod rom;

fn main() {
    let nes = nes::load("../sample1/sample1.nes");
    nes.start();
}
