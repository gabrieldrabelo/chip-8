//! The behaviours that differ between versions of the CHIP-8.

/// A switch for each behaviour that differs between CHIP-8 versions.
///
/// Each game expects one set of behaviours. Use [`Quirks::original`] or
/// [`Quirks::modern`], or change the fields one by one.
#[derive(Debug, PartialEq)]
pub struct Quirks {
    /// `8XY1`, `8XY2` and `8XY3` reset `VF` to zero.
    pub vf_reset: bool,
    /// `FX55` and `FX65` move the index register `I`.
    pub memory_increments_i: bool,
    /// `DXYN` waits for the next frame, so at most 60 sprites are drawn per second.
    pub display_wait: bool,
    /// Sprites are cut at the screen edge instead of wrapping around.
    pub clipping: bool,
    /// `8XY6` and `8XYE` shift `Vy` instead of `Vx`.
    pub shift_uses_vy: bool,
    /// `BNNN` adds `Vx` (the first digit of `nnn`) instead of `V0`.
    pub jump_uses_vx: bool,
}

impl Quirks {
    /// The behaviour of most modern emulators, and the default of [`Chip8::new`](crate::cpu::Chip8::new).
    pub fn modern() -> Self {
        Self {
            vf_reset: false,
            memory_increments_i: false,
            display_wait: false,
            clipping: true,
            shift_uses_vy: false,
            jump_uses_vx: false,
        }
    }

    /// The behaviour of the original CHIP-8 on the COSMAC VIP (1977).
    pub fn original() -> Self {
        Self {
            vf_reset: true,
            memory_increments_i: true,
            display_wait: true,
            clipping: true,
            shift_uses_vy: true,
            jump_uses_vx: false,
        }
    }
}
