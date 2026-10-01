//! The CHIP-8 instruction set and the decoder that recognises it.

/// One CHIP-8 instruction, already split into its arguments.
///
/// In the arguments, `x` and `y` are register numbers, `n` is a 4-bit number,
/// `nn` is a byte and `nnn` (the `u16` ones) is a 12-bit address.
/// The comment next to each variant shows the opcode it comes from.
#[derive(Debug, PartialEq)]
pub enum Instruction {
    ClearScreen,                          // 00E0
    Return,                               // 00EE
    Jump(u16),                            // 1NNN
    Call(u16),                            // 2NNN
    SkipIfEqualByte { x: u8, nn: u8 },    // 3XNN
    SkipIfNotEqualByte { x: u8, nn: u8 }, // 4XNN
    SkipIfEqualReg { x: u8, y: u8 },      // 5XY0
    LoadByte { x: u8, nn: u8 },           // 6XNN
    AddByte { x: u8, nn: u8 },            // 7XNN
    LoadReg { x: u8, y: u8 },             // 8XY0
    Or { x: u8, y: u8 },                  // 8XY1
    And { x: u8, y: u8 },                 // 8XY2
    Xor { x: u8, y: u8 },                 // 8XY3
    AddReg { x: u8, y: u8 },              // 8XY4
    SubReg { x: u8, y: u8 },              // 8XY5
    ShiftRight { x: u8, y: u8 },          // 8XY6
    SubReverse { x: u8, y: u8 },          // 8XY7
    ShiftLeft { x: u8, y: u8 },           // 8XYE
    SkipIfNotEqualReg { x: u8, y: u8 },   // 9XY0
    LoadI(u16),                           // ANNN
    JumpV0(u16),                          // BNNN
    Random { x: u8, nn: u8 },             // CXNN
    Draw { x: u8, y: u8, n: u8 },         // DXYN
    SkipIfKeyPressed(u8),                 // EX9E
    SkipIfKeyNotPressed(u8),              // EXA1
    LoadDelay(u8),                        // FX07
    WaitKey(u8),                          // FX0A
    SetDelay(u8),                         // FX15
    SetSound(u8),                         // FX18
    AddI(u8),                             // FX1E
    LoadFont(u8),                         // FX29
    Bcd(u8),                              // FX33
    StoreRegs(u8),                        // FX55
    LoadRegs(u8),                         // FX65
    Unknown(u16),                         // qualquer outro opcode
}

/// Recognises an opcode and splits it into an [`Instruction`].
///
/// An opcode that matches nothing becomes [`Instruction::Unknown`].
///
/// ```
/// use chip8::instruction::{decode, Instruction};
///
/// assert_eq!(decode(0x1234), Instruction::Jump(0x234));
/// ```
pub fn decode(opcode: u16) -> Instruction {
    let nibbles = (
        ((opcode >> 12) & 0xF) as u8,
        ((opcode >> 8) & 0xF) as u8,
        ((opcode >> 4) & 0xF) as u8,
        (opcode & 0xF) as u8,
    );

    let x = nibbles.1;
    let y = nibbles.2;
    let n = nibbles.3;
    let nn = (opcode & 0xFF) as u8;
    let nnn = opcode & 0xFFF;

    match nibbles {
        (0x0, 0x0, 0xE, 0x0) => Instruction::ClearScreen,
        (0x0, 0x0, 0xE, 0xE) => Instruction::Return,
        (0x1, _, _, _) => Instruction::Jump(nnn),
        (0x2, _, _, _) => Instruction::Call(nnn),
        (0x3, _, _, _) => Instruction::SkipIfEqualByte { x, nn },
        (0x4, _, _, _) => Instruction::SkipIfNotEqualByte { x, nn },
        (0x5, _, _, 0x0) => Instruction::SkipIfEqualReg { x, y },
        (0x6, _, _, _) => Instruction::LoadByte { x, nn },
        (0x7, _, _, _) => Instruction::AddByte { x, nn },
        (0x8, _, _, 0x0) => Instruction::LoadReg { x, y },
        (0x8, _, _, 0x1) => Instruction::Or { x, y },
        (0x8, _, _, 0x2) => Instruction::And { x, y },
        (0x8, _, _, 0x3) => Instruction::Xor { x, y },
        (0x8, _, _, 0x4) => Instruction::AddReg { x, y },
        (0x8, _, _, 0x5) => Instruction::SubReg { x, y },
        (0x8, _, _, 0x6) => Instruction::ShiftRight { x, y },
        (0x8, _, _, 0x7) => Instruction::SubReverse { x, y },
        (0x8, _, _, 0xE) => Instruction::ShiftLeft { x, y },
        (0x9, _, _, 0x0) => Instruction::SkipIfNotEqualReg { x, y },
        (0xA, _, _, _) => Instruction::LoadI(nnn),
        (0xB, _, _, _) => Instruction::JumpV0(nnn),
        (0xC, _, _, _) => Instruction::Random { x, nn },
        (0xD, _, _, _) => Instruction::Draw { x, y, n },
        (0xE, _, 0x9, 0xE) => Instruction::SkipIfKeyPressed(x),
        (0xE, _, 0xA, 0x1) => Instruction::SkipIfKeyNotPressed(x),
        (0xF, _, 0x0, 0x7) => Instruction::LoadDelay(x),
        (0xF, _, 0x0, 0xA) => Instruction::WaitKey(x),
        (0xF, _, 0x1, 0x5) => Instruction::SetDelay(x),
        (0xF, _, 0x1, 0x8) => Instruction::SetSound(x),
        (0xF, _, 0x1, 0xE) => Instruction::AddI(x),
        (0xF, _, 0x2, 0x9) => Instruction::LoadFont(x),
        (0xF, _, 0x3, 0x3) => Instruction::Bcd(x),
        (0xF, _, 0x5, 0x5) => Instruction::StoreRegs(x),
        (0xF, _, 0x6, 0x5) => Instruction::LoadRegs(x),
        _ => Instruction::Unknown(opcode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_clear_screen() {
        assert_eq!(decode(0x00E0), Instruction::ClearScreen);
    }

    #[test]
    fn decodes_jump() {
        assert_eq!(decode(0x1234), Instruction::Jump(0x234));
    }

    #[test]
    fn decodes_register_and_byte() {
        assert_eq!(decode(0x6A2B), Instruction::LoadByte { x: 0xA, nn: 0x2B });
    }

    #[test]
    fn decodes_draw() {
        assert_eq!(decode(0xD125), Instruction::Draw { x: 1, y: 2, n: 5 });
    }

    #[test]
    fn decodes_alu_by_last_nibble() {
        assert_eq!(decode(0x8124), Instruction::AddReg { x: 1, y: 2 });
        assert_eq!(decode(0x812E), Instruction::ShiftLeft { x: 1, y: 2 });
    }

    #[test]
    fn decodes_f_group_by_last_two_nibbles() {
        assert_eq!(decode(0xF333), Instruction::Bcd(3));
    }

    #[test]
    fn unknown_opcode_is_kept() {
        assert_eq!(decode(0x0123), Instruction::Unknown(0x0123));
    }
}
