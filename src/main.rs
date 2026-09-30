use chip8::cpu::Chip8;
use std::{env, fs};

const STEPS: usize = 1000;

fn main() {
    let path = env::args().nth(1).expect("usage: chip8 <rom-file>");
    let rom = fs::read(&path).expect("could not read the ROM file");

    let mut chip = Chip8::new();
    chip.load_rom(&rom).expect("ROM is too large");

    for _ in 0..STEPS {
        chip.step().expect("instruction failed");
    }

    print!("{}", chip.display_as_text());
}
