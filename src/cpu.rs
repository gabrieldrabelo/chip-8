const PROGRAM_START: u16 = 0x200;
const FONT_START: usize = 0x050;

#[rustfmt::skip]
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug, PartialEq)]
pub enum Error {
    RomTooLarge { size: usize, max: usize },
}

pub struct Chip8 {
    pub memory: [u8; 4096],
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        memory[FONT_START..FONT_START + FONTSET.len()].copy_from_slice(&FONTSET);

        Self {
            memory,
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START,
            sp: 0,
            stack: [0; 16],
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), Error> {
        let start = PROGRAM_START as usize;
        let max = self.memory.len() - start;

        if rom.len() > max {
            return Err(Error::RomTooLarge {
                size: rom.len(),
                max,
            });
        }

        self.memory[start..start + rom.len()].copy_from_slice(rom);
        Ok(())
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_program_start() {
        let chip = Chip8::new();
        assert_eq!(chip.pc, 0x200);
    }

    #[test]
    fn new_leaves_program_area_zeroed() {
        let chip = Chip8::new();
        assert_eq!(&chip.memory[0x200..], &[0u8; 4096 - 0x200]);
    }

    #[test]
    fn new_loads_fontset() {
        let chip = Chip8::new();
        assert_eq!(chip.memory[0x050], 0xF0);
        assert_eq!(chip.memory[0x050 + 79], 0x80);
    }

    #[test]
    fn load_rom_copies_at_program_start() {
        let mut chip = Chip8::new();
        chip.load_rom(&[0xA2, 0xF0]).unwrap();
        assert_eq!(chip.memory[0x200], 0xA2);
        assert_eq!(chip.memory[0x201], 0xF0);
    }

    #[test]
    fn load_rom_accepts_max_size() {
        let mut chip = Chip8::new();
        assert!(chip.load_rom(&[1u8; 3584]).is_ok());
    }

    #[test]
    fn load_rom_rejects_oversized_rom() {
        let mut chip = Chip8::new();
        let rom = [0u8; 3585];
        assert_eq!(
            chip.load_rom(&rom),
            Err(Error::RomTooLarge {
                size: 3585,
                max: 3584
            })
        );
    }
}
