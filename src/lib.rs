//! A CHIP-8 emulator core.
//!
//! The crate has three parts:
//!
//! - [`cpu`] holds the machine itself ([`cpu::Chip8`]): memory, registers and the CPU cycle.
//! - [`instruction`] turns a 16-bit opcode into an [`instruction::Instruction`].
//! - [`quirks`] lists the behaviours that differ between CHIP-8 versions.
//!
//! The core knows nothing about windows, keyboards or sound, so it can be tested on its own.
//!
//! ```
//! use chip8::cpu::Chip8;
//!
//! let mut chip = Chip8::new();
//! chip.load_rom(&[0x6A, 0x2B]).unwrap(); // VA = 0x2B
//! chip.step().unwrap();
//! assert_eq!(chip.v[0xA], 0x2B);
//! ```

pub mod cpu;
pub mod instruction;
pub mod quirks;
