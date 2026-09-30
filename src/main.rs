use chip8::cpu::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use chip8::quirks::Quirks;
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use rodio::source::{SineWave, Source};
use rodio::{DeviceSinkBuilder, Player};
use std::{env, fs};

const FPS: u32 = 60;
const DEFAULT_SPEED: u32 = 700;
const SPEED_STEP: u32 = 100;
const MIN_SPEED: u32 = 100;
const MAX_SPEED: u32 = 3000;
const PIXEL_ON: u32 = 0x00FF_FFFF;
const PIXEL_OFF: u32 = 0x0000_0000;
const KEYMAP: [(Key, usize); 16] = [
    (Key::Key1, 0x1),
    (Key::Key2, 0x2),
    (Key::Key3, 0x3),
    (Key::Key4, 0xC),
    (Key::Q, 0x4),
    (Key::W, 0x5),
    (Key::E, 0x6),
    (Key::R, 0xD),
    (Key::A, 0x7),
    (Key::S, 0x8),
    (Key::D, 0x9),
    (Key::F, 0xE),
    (Key::Z, 0xA),
    (Key::X, 0x0),
    (Key::C, 0xB),
    (Key::V, 0xF),
];
const BEEP_FREQUENCY_HZ: f32 = 440.0;
const BEEP_VOLUME: f32 = 0.2;

fn steps_for_frame(speed: u32, carry: &mut u32) -> u32 {
    *carry += speed;
    let steps = *carry / FPS;
    *carry %= FPS;
    steps
}

fn adjusted_speed(window: &Window, speed: u32) -> u32 {
    if window.is_key_pressed(Key::Up, KeyRepeat::No) {
        (speed + SPEED_STEP).min(MAX_SPEED)
    } else if window.is_key_pressed(Key::Down, KeyRepeat::No) {
        speed.saturating_sub(SPEED_STEP).max(MIN_SPEED)
    } else {
        speed
    }
}

fn main() {
    let path = env::args().nth(1).expect("usage: chip8 <rom-file>");
    let rom = fs::read(&path).expect("could not read the ROM file");

    let mut chip = Chip8::new();
    chip.quirks = match env::args().nth(2).as_deref() {
        Some("original") => Quirks::original(),
        _ => Quirks::modern(),
    };
    chip.load_rom(&rom).expect("ROM is too large");

    let options = WindowOptions {
        scale: Scale::X16,
        ..WindowOptions::default()
    };
    let mut window = Window::new("CHIP-8", DISPLAY_WIDTH, DISPLAY_HEIGHT, options)
        .expect("could not create the window");
    window.set_target_fps(FPS as usize);
    let audio = DeviceSinkBuilder::open_default_sink().expect("could not open the audio output");
    let beep = Player::connect_new(audio.mixer());
    beep.append(SineWave::new(BEEP_FREQUENCY_HZ).amplify(BEEP_VOLUME));
    beep.pause();

    let mut speed = DEFAULT_SPEED;
    let mut carry = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let new_speed = adjusted_speed(&window, speed);

        if new_speed != speed {
            speed = new_speed;
            window.set_title(&format!("CHIP-8 - {speed} instructions/s"));
        }

        for (key, index) in KEYMAP {
            chip.keys[index] = window.is_key_down(key);
        }

        let steps = steps_for_frame(speed, &mut carry);
        for _ in 0..steps {
            chip.step().expect("instruction failed");
        }

        if chip.sound_timer > 0 {
            beep.play();
        } else {
            beep.pause();
        }

        chip.tick_timers();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixty_frames_run_exactly_one_second_of_instructions() {
        let mut carry = 0;
        let total: u32 = (0..60).map(|_| steps_for_frame(700, &mut carry)).sum();
        assert_eq!(total, 700);
    }

    #[test]
    fn each_frame_gets_a_whole_number_close_to_the_average() {
        let mut carry = 0;
        for _ in 0..60 {
            let steps = steps_for_frame(700, &mut carry);
            assert!(steps == 11 || steps == 12);
        }
    }
}
