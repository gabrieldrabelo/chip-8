use chip8::cpu::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use minifb::{Key, Scale, Window, WindowOptions};
use std::{env, fs};

const STEPS_PER_FRAME: usize = 12;
const PIXEL_ON: u32 = 0x00FF_FFFF;
const PIXEL_OFF: u32 = 0x0000_0000;

fn main() {
    let path = env::args().nth(1).expect("usage: chip8 <rom-file>");
    let rom = fs::read(&path).expect("could not read the ROM file");

    let mut chip = Chip8::new();
    chip.load_rom(&rom).expect("ROM is too large");

    let options = WindowOptions {
        scale: Scale::X16,
        ..WindowOptions::default()
    };
    let mut window = Window::new("CHIP-8", DISPLAY_WIDTH, DISPLAY_HEIGHT, options)
        .expect("could not create the window");
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for _ in 0..STEPS_PER_FRAME {
            chip.step().expect("instruction failed");
        }

        let pixels: Vec<u32> = chip
            .display
            .iter()
            .map(|&on| if on { PIXEL_ON } else { PIXEL_OFF })
            .collect();

        window
            .update_with_buffer(&pixels, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .expect("could not update the window");
    }
}
