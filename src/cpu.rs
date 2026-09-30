use crate::instruction::Instruction;

const PROGRAM_START: u16 = 0x200;
const FONT_START: usize = 0x050;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
const RNG_SEED: u32 = 0x1234_5678;

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
    StackOverflow,
    StackUnderflow,
}

// chip 8 is a imaginary 1970s computer
pub struct Chip8 {
    pub memory: [u8; 4096], // ram (row of byte-sized boxes)
    pub v: [u8; 16],        // general-purpose register (V0 - VF) 1 byte each
    pub i: u16,             // index register (holds a memory address)
    pub pc: u16,            // program counter, holds the next instruction address
    pub sp: u8, // stack pointer, how many items are on the stack now (so, we have the next free slot)
    pub stack: [u16; 16], // return addresses saved when calling a subroutine
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub rng_state: u32,
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
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            rng_state: RNG_SEED,
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

    pub fn fetch(&mut self) -> u16 {
        let pc = self.pc as usize;
        let high = self.memory[pc] as u16;
        let low = self.memory[pc + 1] as u16;

        self.pc += 2;

        (high << 8) | low
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<(), Error> {
        match instruction {
            Instruction::ClearScreen => self.display.fill(false),
            Instruction::Jump(address) => self.pc = address,
            Instruction::LoadByte { x, nn } => self.v[x as usize] = nn,
            Instruction::AddByte { x, nn } => {
                let x = x as usize;
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            Instruction::LoadI(address) => self.i = address,
            Instruction::Draw { x, y, n } => self.draw(x, y, n),
            Instruction::Call(address) => self.call(address)?,
            Instruction::Return => self.return_from_subroutine()?,
            Instruction::SkipIfEqualByte { x, nn } => self.skip_if(self.v[x as usize] == nn),
            Instruction::SkipIfNotEqualByte { x, nn } => self.skip_if(self.v[x as usize] != nn),
            Instruction::SkipIfEqualReg { x, y } => {
                self.skip_if(self.v[x as usize] == self.v[y as usize])
            }
            Instruction::SkipIfNotEqualReg { x, y } => {
                self.skip_if(self.v[x as usize] != self.v[y as usize])
            }
            Instruction::LoadReg { x, y } => self.v[x as usize] = self.v[y as usize],
            Instruction::Or { x, y } => self.v[x as usize] |= self.v[y as usize],
            Instruction::And { x, y } => self.v[x as usize] &= self.v[y as usize],
            Instruction::Xor { x, y } => self.v[x as usize] ^= self.v[y as usize],
            Instruction::AddReg { x, y } => self.add_reg(x, y),
            Instruction::SubReg { x, y } => self.sub_reg(x, y),
            Instruction::SubReverse { x, y } => self.sub_reverse(x, y),
            Instruction::ShiftRight { x, .. } => self.shift_right(x),
            Instruction::ShiftLeft { x, .. } => self.shift_left(x),
            Instruction::JumpV0(address) => self.pc = address + self.v[0] as u16,
            Instruction::Random { x, nn } => self.v[x as usize] = self.next_random() & nn,
            Instruction::AddI(x) => self.i = self.i.wrapping_add(self.v[x as usize] as u16),
            Instruction::LoadFont(x) => self.load_font(x),
            Instruction::Bcd(x) => self.bcd(x),
            Instruction::StoreRegs(x) => self.store_regs(x),
            Instruction::LoadRegs(x) => self.load_regs(x),
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, x: u8, y: u8, n: u8) {
        let start_x = self.v[x as usize] as usize % DISPLAY_WIDTH;
        let start_y = self.v[y as usize] as usize % DISPLAY_HEIGHT;
        self.v[0xF] = 0;

        for row in 0..n as usize {
            let pixel_y = start_y + row;
            if pixel_y >= DISPLAY_HEIGHT {
                break;
            }
            let sprite_byte = self.memory[self.i as usize + row];

            for col in 0..8 {
                let pixel_x = start_x + col;
                if pixel_x >= DISPLAY_WIDTH {
                    break;
                }
                let sprite_pixel_on = ((sprite_byte >> (7 - col)) & 1) == 1;
                if sprite_pixel_on {
                    let index = pixel_y * DISPLAY_WIDTH + pixel_x;
                    if self.display[index] {
                        self.v[0xF] = 1;
                    }
                    self.display[index] ^= true;
                }
            }
        }
    }

    fn call(&mut self, address: u16) -> Result<(), Error> {
        if self.sp as usize >= self.stack.len() {
            return Err(Error::StackOverflow);
        }
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = address;
        Ok(())
    }

    fn return_from_subroutine(&mut self) -> Result<(), Error> {
        if self.sp == 0 {
            return Err(Error::StackUnderflow);
        }
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
        Ok(())
    }

    fn skip_if(&mut self, condition: bool) {
        if condition {
            self.pc += 2;
        }
    }

    fn add_reg(&mut self, x: u8, y: u8) {
        let (result, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = u8::from(carry);
    }

    fn sub_reg(&mut self, x: u8, y: u8) {
        let (result, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = u8::from(!borrow);
    }

    fn sub_reverse(&mut self, x: u8, y: u8) {
        let (result, borrow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = u8::from(!borrow);
    }

    fn shift_right(&mut self, x: u8) {
        let value = self.v[x as usize];
        self.v[x as usize] = value >> 1;
        self.v[0xF] = value & 1;
    }

    fn shift_left(&mut self, x: u8) {
        let value = self.v[x as usize];
        self.v[x as usize] = value << 1;
        self.v[0xF] = value >> 7;
    }

    fn next_random(&mut self) -> u8 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x;
        (x >> 24) as u8
    }

    fn load_font(&mut self, x: u8) {
        let digit = u16::from(self.v[x as usize] & 0xF);
        self.i = FONT_START as u16 + digit * 5;
    }

    fn bcd(&mut self, x: u8) {
        let value = self.v[x as usize];
        let start = self.i as usize;
        self.memory[start] = value / 100;
        self.memory[start + 1] = (value / 10) % 10;
        self.memory[start + 2] = value % 10;
    }

    fn store_regs(&mut self, x: u8) {
        let count = x as usize + 1;
        let start = self.i as usize;
        self.memory[start..start + count].copy_from_slice(&self.v[..count]);
    }

    fn load_regs(&mut self, x: u8) {
        let count = x as usize + 1;
        let start = self.i as usize;
        self.v[..count].copy_from_slice(&self.memory[start..start + count]);
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

    fn chip_with_sprite(sprite: &[u8]) -> Chip8 {
        let mut chip = Chip8::new();
        chip.memory[0x300..0x300 + sprite.len()].copy_from_slice(sprite);
        chip.i = 0x300;
        chip
    }

    fn pixel_at(chip: &Chip8, x: usize, y: usize) -> bool {
        chip.display[y * DISPLAY_WIDTH + x]
    }

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

    #[test]
    fn fetch_reads_big_endian_opcode_and_advances_pc() {
        let mut chip = Chip8::new();
        chip.load_rom(&[0xA2, 0xF0]).unwrap();

        assert_eq!(chip.fetch(), 0xA2F0);
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn fetch_reads_consecutive_opcodes() {
        let mut chip = Chip8::new();
        chip.load_rom(&[0xA2, 0xF0, 0x00, 0xE0]).unwrap();

        assert_eq!(chip.fetch(), 0xA2F0);
        assert_eq!(chip.fetch(), 0x00E0);
        assert_eq!(chip.pc, 0x204);
    }

    #[test]
    fn new_starts_with_blank_display() {
        let chip = Chip8::new();
        assert_eq!(chip.display, [false; DISPLAY_WIDTH * DISPLAY_HEIGHT]);
    }

    #[test]
    fn clear_screen_turns_all_pixels_off() {
        let mut chip = Chip8::new();
        chip.display[0] = true;
        chip.display[DISPLAY_WIDTH * DISPLAY_HEIGHT - 1] = true;

        chip.execute(Instruction::ClearScreen).unwrap();

        assert_eq!(chip.display, [false; DISPLAY_WIDTH * DISPLAY_HEIGHT]);
    }

    #[test]
    fn jump_sets_pc() {
        let mut chip = Chip8::new();
        chip.execute(Instruction::Jump(0x234)).unwrap();
        assert_eq!(chip.pc, 0x234);
    }

    #[test]
    fn load_byte_sets_register() {
        let mut chip = Chip8::new();
        chip.execute(Instruction::LoadByte { x: 0xA, nn: 0x2B })
            .unwrap();
        assert_eq!(chip.v[0xA], 0x2B);
    }

    #[test]
    fn add_byte_adds_to_register() {
        let mut chip = Chip8::new();
        chip.v[3] = 5;
        chip.execute(Instruction::AddByte { x: 3, nn: 7 }).unwrap();
        assert_eq!(chip.v[3], 12);
    }

    #[test]
    fn add_byte_wraps_and_leaves_vf_alone() {
        let mut chip = Chip8::new();
        chip.v[1] = 250;
        chip.execute(Instruction::AddByte { x: 1, nn: 10 }).unwrap();
        assert_eq!(chip.v[1], 4);
        assert_eq!(chip.v[0xF], 0);
    }

    #[test]
    fn load_i_sets_index_register() {
        let mut chip = Chip8::new();
        chip.execute(Instruction::LoadI(0x2F0)).unwrap();
        assert_eq!(chip.i, 0x2F0);
    }

    #[test]
    fn draw_lights_sprite_pixels() {
        let mut chip = chip_with_sprite(&[0b1010_0000]);
        chip.v[0] = 2;
        chip.v[1] = 3;

        chip.execute(Instruction::Draw { x: 0, y: 1, n: 1 })
            .unwrap();

        assert!(pixel_at(&chip, 2, 3));
        assert!(!pixel_at(&chip, 3, 3));
        assert!(pixel_at(&chip, 4, 3));
        assert_eq!(chip.v[0xF], 0);
    }

    #[test]
    fn drawing_twice_erases_and_sets_vf() {
        let mut chip = chip_with_sprite(&[0b1000_0000]);

        chip.execute(Instruction::Draw { x: 0, y: 1, n: 1 })
            .unwrap();
        chip.execute(Instruction::Draw { x: 0, y: 1, n: 1 })
            .unwrap();

        assert!(!pixel_at(&chip, 0, 0));
        assert_eq!(chip.v[0xF], 1);
    }

    #[test]
    fn draw_clips_at_screen_edges() {
        let mut chip = chip_with_sprite(&[0xFF, 0xFF]);
        chip.v[0] = 60;
        chip.v[1] = 31;

        chip.execute(Instruction::Draw { x: 0, y: 1, n: 2 })
            .unwrap();

        assert!(pixel_at(&chip, 63, 31));
        assert!(!pixel_at(&chip, 0, 31));
        assert!(!pixel_at(&chip, 60, 0));
    }

    #[test]
    fn draw_start_position_wraps() {
        let mut chip = chip_with_sprite(&[0b1000_0000]);
        chip.v[0] = 69;

        chip.execute(Instruction::Draw { x: 0, y: 1, n: 1 })
            .unwrap();

        assert!(pixel_at(&chip, 5, 0));
    }

    #[test]
    fn draw_renders_font_digit_zero() {
        let mut chip = Chip8::new();
        chip.i = 0x050;

        chip.execute(Instruction::Draw { x: 0, y: 1, n: 5 })
            .unwrap();

        assert!(pixel_at(&chip, 0, 0));
        assert!(!pixel_at(&chip, 4, 0));
        assert!(pixel_at(&chip, 3, 1));
        assert!(!pixel_at(&chip, 1, 1));
    }

    #[test]
    fn call_saves_return_address_and_jumps() {
        let mut chip = Chip8::new();
        chip.pc = 0x202;

        chip.execute(Instruction::Call(0x300)).unwrap();

        assert_eq!(chip.pc, 0x300);
        assert_eq!(chip.sp, 1);
        assert_eq!(chip.stack[0], 0x202);
    }

    #[test]
    fn return_goes_back_to_caller() {
        let mut chip = Chip8::new();
        chip.pc = 0x202;
        chip.execute(Instruction::Call(0x300)).unwrap();

        chip.execute(Instruction::Return).unwrap();

        assert_eq!(chip.pc, 0x202);
        assert_eq!(chip.sp, 0);
    }

    #[test]
    fn nested_calls_unwind_in_order() {
        let mut chip = Chip8::new();
        chip.pc = 0x202;
        chip.execute(Instruction::Call(0x300)).unwrap();
        chip.pc = 0x302;
        chip.execute(Instruction::Call(0x400)).unwrap();

        chip.execute(Instruction::Return).unwrap();
        assert_eq!(chip.pc, 0x302);
        chip.execute(Instruction::Return).unwrap();
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn call_fails_when_stack_is_full() {
        let mut chip = Chip8::new();
        for _ in 0..16 {
            chip.execute(Instruction::Call(0x300)).unwrap();
        }

        assert_eq!(
            chip.execute(Instruction::Call(0x300)),
            Err(Error::StackOverflow)
        );
    }

    #[test]
    fn return_fails_on_empty_stack() {
        let mut chip = Chip8::new();

        assert_eq!(
            chip.execute(Instruction::Return),
            Err(Error::StackUnderflow)
        );
    }

    #[test]
    fn skip_if_equal_byte() {
        let mut chip = Chip8::new();
        chip.v[2] = 0x42;

        chip.execute(Instruction::SkipIfEqualByte { x: 2, nn: 0x42 })
            .unwrap();
        assert_eq!(chip.pc, 0x202);

        chip.execute(Instruction::SkipIfEqualByte { x: 2, nn: 0x43 })
            .unwrap();
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn skip_if_not_equal_byte() {
        let mut chip = Chip8::new();
        chip.v[2] = 0x42;

        chip.execute(Instruction::SkipIfNotEqualByte { x: 2, nn: 0x42 })
            .unwrap();
        assert_eq!(chip.pc, 0x200);

        chip.execute(Instruction::SkipIfNotEqualByte { x: 2, nn: 0x43 })
            .unwrap();
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn skip_if_equal_reg() {
        let mut chip = Chip8::new();
        chip.v[1] = 7;
        chip.v[2] = 7;
        chip.v[3] = 8;

        chip.execute(Instruction::SkipIfEqualReg { x: 1, y: 3 })
            .unwrap();
        assert_eq!(chip.pc, 0x200);

        chip.execute(Instruction::SkipIfEqualReg { x: 1, y: 2 })
            .unwrap();
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn skip_if_not_equal_reg() {
        let mut chip = Chip8::new();
        chip.v[1] = 7;
        chip.v[2] = 7;
        chip.v[3] = 8;

        chip.execute(Instruction::SkipIfNotEqualReg { x: 1, y: 2 })
            .unwrap();
        assert_eq!(chip.pc, 0x200);

        chip.execute(Instruction::SkipIfNotEqualReg { x: 1, y: 3 })
            .unwrap();
        assert_eq!(chip.pc, 0x202);
    }

    #[test]
    fn load_reg_copies_value() {
        let mut chip = Chip8::new();
        chip.v[2] = 9;

        chip.execute(Instruction::LoadReg { x: 1, y: 2 }).unwrap();

        assert_eq!(chip.v[1], 9);
    }

    #[test]
    fn logic_operations() {
        let mut chip = Chip8::new();
        chip.v[2] = 0b1010;

        chip.v[1] = 0b1100;
        chip.execute(Instruction::Or { x: 1, y: 2 }).unwrap();
        assert_eq!(chip.v[1], 0b1110);

        chip.v[1] = 0b1100;
        chip.execute(Instruction::And { x: 1, y: 2 }).unwrap();
        assert_eq!(chip.v[1], 0b1000);

        chip.v[1] = 0b1100;
        chip.execute(Instruction::Xor { x: 1, y: 2 }).unwrap();
        assert_eq!(chip.v[1], 0b0110);
    }

    #[test]
    fn add_reg_without_carry() {
        let mut chip = Chip8::new();
        chip.v[1] = 10;
        chip.v[2] = 20;

        chip.execute(Instruction::AddReg { x: 1, y: 2 }).unwrap();

        assert_eq!(chip.v[1], 30);
        assert_eq!(chip.v[0xF], 0);
    }

    #[test]
    fn add_reg_sets_carry_and_wraps() {
        let mut chip = Chip8::new();
        chip.v[1] = 200;
        chip.v[2] = 100;

        chip.execute(Instruction::AddReg { x: 1, y: 2 }).unwrap();

        assert_eq!(chip.v[1], 44);
        assert_eq!(chip.v[0xF], 1);
    }

    #[test]
    fn sub_reg_without_borrow() {
        let mut chip = Chip8::new();
        chip.v[1] = 10;
        chip.v[2] = 3;

        chip.execute(Instruction::SubReg { x: 1, y: 2 }).unwrap();

        assert_eq!(chip.v[1], 7);
        assert_eq!(chip.v[0xF], 1);
    }

    #[test]
    fn sub_reg_with_borrow_wraps() {
        let mut chip = Chip8::new();
        chip.v[1] = 3;
        chip.v[2] = 10;

        chip.execute(Instruction::SubReg { x: 1, y: 2 }).unwrap();

        assert_eq!(chip.v[1], 249);
        assert_eq!(chip.v[0xF], 0);
    }

    #[test]
    fn sub_reverse_computes_vy_minus_vx() {
        let mut chip = Chip8::new();
        chip.v[1] = 3;
        chip.v[2] = 10;

        chip.execute(Instruction::SubReverse { x: 1, y: 2 })
            .unwrap();

        assert_eq!(chip.v[1], 7);
        assert_eq!(chip.v[0xF], 1);
    }

    #[test]
    fn shift_right_puts_dropped_bit_in_vf() {
        let mut chip = Chip8::new();

        chip.v[1] = 0b0000_0101;
        chip.execute(Instruction::ShiftRight { x: 1, y: 0 })
            .unwrap();
        assert_eq!(chip.v[1], 0b0000_0010);
        assert_eq!(chip.v[0xF], 1);

        chip.v[1] = 0b0000_0100;
        chip.execute(Instruction::ShiftRight { x: 1, y: 0 })
            .unwrap();
        assert_eq!(chip.v[1], 0b0000_0010);
        assert_eq!(chip.v[0xF], 0);
    }

    #[test]
    fn shift_left_puts_dropped_bit_in_vf() {
        let mut chip = Chip8::new();

        chip.v[1] = 0b1000_0001;
        chip.execute(Instruction::ShiftLeft { x: 1, y: 0 }).unwrap();
        assert_eq!(chip.v[1], 0b0000_0010);
        assert_eq!(chip.v[0xF], 1);

        chip.v[1] = 0b0100_0000;
        chip.execute(Instruction::ShiftLeft { x: 1, y: 0 }).unwrap();
        assert_eq!(chip.v[1], 0b1000_0000);
        assert_eq!(chip.v[0xF], 0);
    }

    #[test]
    fn flag_wins_when_x_is_vf() {
        let mut chip = Chip8::new();
        chip.v[0xF] = 200;
        chip.v[1] = 100;

        chip.execute(Instruction::AddReg { x: 0xF, y: 1 }).unwrap();

        assert_eq!(chip.v[0xF], 1);
    }

    #[test]
    fn jump_v0_adds_v0_to_address() {
        let mut chip = Chip8::new();
        chip.v[0] = 5;

        chip.execute(Instruction::JumpV0(0x300)).unwrap();

        assert_eq!(chip.pc, 0x305);
    }

    #[test]
    fn random_with_zero_mask_gives_zero() {
        let mut chip = Chip8::new();
        chip.v[1] = 77;

        chip.execute(Instruction::Random { x: 1, nn: 0x00 })
            .unwrap();

        assert_eq!(chip.v[1], 0);
    }

    #[test]
    fn random_respects_the_mask() {
        let mut chip = Chip8::new();

        for _ in 0..100 {
            chip.execute(Instruction::Random { x: 1, nn: 0x0F })
                .unwrap();
            assert_eq!(chip.v[1] & 0xF0, 0);
        }
    }

    #[test]
    fn random_is_not_constant() {
        let mut chip = Chip8::new();
        chip.execute(Instruction::Random { x: 1, nn: 0xFF })
            .unwrap();
        let first = chip.v[1];

        let mut changed = false;
        for _ in 0..100 {
            chip.execute(Instruction::Random { x: 1, nn: 0xFF })
                .unwrap();
            if chip.v[1] != first {
                changed = true;
            }
        }

        assert!(changed);
    }

    #[test]
    fn add_i_adds_vx_to_index() {
        let mut chip = Chip8::new();
        chip.i = 0x300;
        chip.v[1] = 5;

        chip.execute(Instruction::AddI(1)).unwrap();

        assert_eq!(chip.i, 0x305);
    }

    #[test]
    fn load_font_points_index_at_digit_sprite() {
        let mut chip = Chip8::new();
        chip.v[1] = 0xA;

        chip.execute(Instruction::LoadFont(1)).unwrap();

        assert_eq!(chip.i, 0x050 + 0xA * 5);
        assert_eq!(chip.memory[chip.i as usize], 0xF0);
        assert_eq!(chip.memory[chip.i as usize + 4], 0x90);
    }

    #[test]
    fn bcd_splits_decimal_digits() {
        let mut chip = Chip8::new();
        chip.i = 0x300;

        chip.v[2] = 156;
        chip.execute(Instruction::Bcd(2)).unwrap();
        assert_eq!(&chip.memory[0x300..0x303], &[1, 5, 6]);

        chip.v[2] = 7;
        chip.execute(Instruction::Bcd(2)).unwrap();
        assert_eq!(&chip.memory[0x300..0x303], &[0, 0, 7]);
    }

    #[test]
    fn store_regs_copies_v0_through_vx() {
        let mut chip = Chip8::new();
        chip.i = 0x300;
        chip.v[0] = 1;
        chip.v[1] = 2;
        chip.v[2] = 3;
        chip.v[3] = 4;

        chip.execute(Instruction::StoreRegs(2)).unwrap();

        assert_eq!(&chip.memory[0x300..0x303], &[1, 2, 3]);
        assert_eq!(chip.memory[0x303], 0);
        assert_eq!(chip.i, 0x300);
    }

    #[test]
    fn load_regs_copies_memory_into_v0_through_vx() {
        let mut chip = Chip8::new();
        chip.i = 0x300;
        chip.memory[0x300..0x303].copy_from_slice(&[9, 8, 7]);

        chip.execute(Instruction::LoadRegs(1)).unwrap();

        assert_eq!(chip.v[0], 9);
        assert_eq!(chip.v[1], 8);
        assert_eq!(chip.v[2], 0);
    }

    #[test]
    fn store_then_load_regs_round_trips() {
        let mut chip = Chip8::new();
        chip.i = 0x300;
        for n in 0..16 {
            chip.v[n] = n as u8 * 3;
        }
        let original = chip.v;

        chip.execute(Instruction::StoreRegs(0xF)).unwrap();
        chip.v = [0; 16];
        chip.execute(Instruction::LoadRegs(0xF)).unwrap();

        assert_eq!(chip.v, original);
    }
}
