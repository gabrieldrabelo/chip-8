#[derive(Debug, PartialEq)]
pub struct Quirks {
    pub vf_reset: bool,
    pub memory_increments_i: bool,
    pub display_wait: bool,
    pub clipping: bool,
    pub shift_uses_vy: bool,
    pub jump_uses_vx: bool,
}

impl Quirks {
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
