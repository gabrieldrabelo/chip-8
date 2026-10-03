//! The CHIP-8 core, wrapped so a web page can use it.

use chip8::cpu::Chip8;
use wasm_bindgen::prelude::*;

/// A CHIP-8 machine that JavaScript can drive.
#[wasm_bindgen]
pub struct Emulator {
    chip: Chip8,
}

#[wasm_bindgen]
impl Emulator {
    /// Creates a machine with nothing loaded.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        Emulator { chip: Chip8::new() }
    }

    /// Loads a ROM into memory.
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), JsError> {
        self.chip.load_rom(rom)?;
        Ok(())
    }

    /// Runs one instruction.
    pub fn step(&mut self) -> Result<(), JsError> {
        self.chip.step()?;
        Ok(())
    }

    /// Counts the timers down. Call it 60 times per second.
    pub fn tick_timers(&mut self) {
        self.chip.tick_timers();
    }

    /// Tells the machine that a hex key (0 to 15) is pressed or released.
    pub fn set_key(&mut self, key: usize, pressed: bool) {
        if let Some(slot) = self.chip.keys.get_mut(key) {
            *slot = pressed;
        }
    }

    /// The screen, one byte per pixel: 1 is lit, 0 is off.
    pub fn pixels(&self) -> Vec<u8> {
        self.chip.display.iter().map(|&on| u8::from(on)).collect()
    }

    /// Whether the beep should be playing right now.
    pub fn is_beeping(&self) -> bool {
        self.chip.sound_timer > 0
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
