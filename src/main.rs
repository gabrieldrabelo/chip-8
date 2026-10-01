use chip8::cpu::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use chip8::quirks::Quirks;
use clap::{Parser, ValueEnum};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use rodio::source::{SineWave, Source};
use rodio::{DeviceSinkBuilder, Player};
use std::path::PathBuf;
use std::{fs, process};

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

/// A CHIP-8 emulator.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Path to the ROM file to run
    rom: PathBuf,

    /// CPU speed in instructions per second (Up/Down arrows change it while playing)
    #[arg(
        long,
        default_value_t = DEFAULT_SPEED,
        value_parser = clap::value_parser!(u32).range(MIN_SPEED as i64..=MAX_SPEED as i64)
    )]
    speed: u32,

    /// Size of each CHIP-8 pixel on the screen
    #[arg(long, value_enum, default_value_t = PixelScale::X16)]
    scale: PixelScale,

    /// Which set of CHIP-8 quirks to use
    #[arg(long, value_enum, default_value_t = QuirksPreset::Modern)]
    quirks: QuirksPreset,

    /// Pause when the program counter reaches this address, in hex (repeatable)
    #[arg(long = "break", value_name = "ADDRESS", value_parser = parse_address)]
    breakpoints: Vec<u16>,
}

#[derive(Clone, Copy, ValueEnum)]
enum QuirksPreset {
    Modern,
    Original,
}

impl QuirksPreset {
    fn quirks(self) -> Quirks {
        match self {
            QuirksPreset::Modern => Quirks::modern(),
            QuirksPreset::Original => Quirks::original(),
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum PixelScale {
    #[value(name = "1")]
    X1,
    #[value(name = "2")]
    X2,
    #[value(name = "4")]
    X4,
    #[value(name = "8")]
    X8,
    #[value(name = "16")]
    X16,
    #[value(name = "32")]
    X32,
}

impl PixelScale {
    fn to_minifb(self) -> Scale {
        match self {
            PixelScale::X1 => Scale::X1,
            PixelScale::X2 => Scale::X2,
            PixelScale::X4 => Scale::X4,
            PixelScale::X8 => Scale::X8,
            PixelScale::X16 => Scale::X16,
            PixelScale::X32 => Scale::X32,
        }
    }
}

fn parse_address(text: &str) -> Result<u16, String> {
    let digits = text.trim_start_matches("0x");
    let address = u16::from_str_radix(digits, 16).map_err(|error| error.to_string())?;
    if address > 0xFFF {
        return Err(String::from("the address must be at most 0xFFF"));
    }
    Ok(address)
}

fn window_title(speed: u32, paused: bool) -> String {
    let suffix = if paused { " [paused]" } else { "" };
    format!("CHIP-8 - {speed} instructions/s{suffix}")
}

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
    if let Err(message) = run(Cli::parse()) {
        eprintln!("error: {message}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let rom = fs::read(&cli.rom)
        .map_err(|error| format!("could not read {}: {error}", cli.rom.display()))?;

    let mut chip = Chip8::new();
    chip.quirks = cli.quirks.quirks();
    chip.load_rom(&rom)
        .map_err(|error| format!("could not load the ROM: {error:?}"))?;

    let options = WindowOptions {
        scale: cli.scale.to_minifb(),
        ..WindowOptions::default()
    };
    let mut window = Window::new(
        &window_title(cli.speed, false),
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        options,
    )
    .map_err(|error| format!("could not create the window: {error}"))?;
    window.set_target_fps(FPS as usize);

    let audio = DeviceSinkBuilder::open_default_sink()
        .map_err(|error| format!("could not open the audio output: {error}"))?;
    let beep = Player::connect_new(audio.mixer());
    beep.append(SineWave::new(BEEP_FREQUENCY_HZ).amplify(BEEP_VOLUME));
    beep.pause();

    let mut speed = cli.speed;
    let mut paused = false;
    let mut carry = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let new_speed = adjusted_speed(&window, speed);

        let pause_toggled = window.is_key_pressed(Key::Space, KeyRepeat::No);

        if new_speed != speed || pause_toggled {
            speed = new_speed;
            paused ^= pause_toggled;
            window.set_title(&window_title(speed, paused));
        }

        if pause_toggled && paused {
            println!("{}\n", chip.debug_state());
        }

        for (key, index) in KEYMAP {
            chip.keys[index] = window.is_key_down(key);
        }

        let steps = if !paused {
            steps_for_frame(speed, &mut carry)
        } else if window.is_key_pressed(Key::N, KeyRepeat::No) {
            1
        } else {
            0
        };

        for _ in 0..steps {
            chip.step()
                .map_err(|error| format!("instruction failed: {error:?}"))?;

            if paused {
                println!("{}\n", chip.debug_state());
            } else if cli.breakpoints.contains(&chip.pc) {
                paused = true;
                window.set_title(&window_title(speed, paused));
                println!("breakpoint at {:#05X}\n{}\n", chip.pc, chip.debug_state());
                break;
            }
        }

        if chip.sound_timer > 0 && !paused {
            beep.play();
        } else {
            beep.pause();
        }

        if !paused {
            chip.tick_timers();
        }

        let pixels: Vec<u32> = chip
            .display
            .iter()
            .map(|&on| if on { PIXEL_ON } else { PIXEL_OFF })
            .collect();

        window
            .update_with_buffer(&pixels, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .map_err(|error| format!("could not update the window: {error}"))?;
    }

    Ok(())
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

    #[test]
    fn cli_definition_is_valid() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_address_accepts_hex_with_or_without_prefix() {
        assert_eq!(parse_address("0x2A4"), Ok(0x2A4));
        assert_eq!(parse_address("2A4"), Ok(0x2A4));
    }

    #[test]
    fn parse_address_rejects_invalid_input() {
        assert!(parse_address("zzz").is_err());
        assert!(parse_address("0x1000").is_err());
    }
}
