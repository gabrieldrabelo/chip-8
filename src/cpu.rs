const PROGRAM_START: u16 = 0x200;

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
        Self {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START,
            sp: 0,
            stack: [0; 16],
        }
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
    fn new_has_zeroed_memory() {
        let chip = Chip8::new();
        assert_eq!(chip.memory, [0u8; 4096]);
    }
}
