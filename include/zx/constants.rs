//! Constants

/// Button indices for input functions
pub mod button {
    pub const UP: u32 = 0;
    pub const DOWN: u32 = 1;
    pub const LEFT: u32 = 2;
    pub const RIGHT: u32 = 3;
    pub const A: u32 = 4;
    pub const B: u32 = 5;
    pub const X: u32 = 6;
    pub const Y: u32 = 7;
    pub const L1: u32 = 8;
    pub const R1: u32 = 9;
    pub const L3: u32 = 10;
    pub const R3: u32 = 11;
    pub const START: u32 = 12;
    pub const SELECT: u32 = 13;
}

/// Cull modes for `cull_mode()`
pub mod cull {
    pub const NONE: u32 = 0;
    pub const BACK: u32 = 1;
    pub const FRONT: u32 = 2;
}

/// Vertex format flags for mesh loading
pub mod format {
    pub const POS: u8 = 0;
    pub const UV: u8 = 1;
    pub const COLOR: u8 = 2;
    pub const NORMAL: u8 = 4;
    pub const SKINNED: u8 = 8;
    pub const TANGENT: u8 = 16;

    // Common combinations
    pub const POS_UV: u8 = UV;
    pub const POS_COLOR: u8 = COLOR;
    pub const POS_NORMAL: u8 = NORMAL;
    pub const POS_UV_NORMAL: u8 = UV | NORMAL;
    pub const POS_UV_COLOR: u8 = UV | COLOR;
    pub const POS_UV_COLOR_NORMAL: u8 = UV | COLOR | NORMAL;
    pub const POS_SKINNED: u8 = SKINNED;
    pub const POS_NORMAL_SKINNED: u8 = NORMAL | SKINNED;
    pub const POS_UV_NORMAL_SKINNED: u8 = UV | NORMAL | SKINNED;

    // Tangent combinations (requires NORMAL)
    pub const POS_UV_NORMAL_TANGENT: u8 = UV | NORMAL | TANGENT;
    pub const POS_UV_COLOR_NORMAL_TANGENT: u8 = UV | COLOR | NORMAL | TANGENT;
}

/// Billboard modes for `draw_billboard()`
pub mod billboard {
    pub const SPHERICAL: u32 = 1;
    pub const CYLINDRICAL_Y: u32 = 2;
    pub const CYLINDRICAL_X: u32 = 3;
    pub const CYLINDRICAL_Z: u32 = 4;
}

/// Screen dimensions (fixed 540p resolution)
pub mod screen {
    /// Screen width in pixels
    pub const WIDTH: u32 = 960;
    /// Screen height in pixels
    pub const HEIGHT: u32 = 540;
}

/// Comparison functions for `begin_pass_full()` depth and stencil parameters
pub mod compare {
    pub const NEVER: u32 = 1;
    pub const LESS: u32 = 2;
    pub const EQUAL: u32 = 3;
    pub const LESS_EQUAL: u32 = 4;
    pub const GREATER: u32 = 5;
    pub const NOT_EQUAL: u32 = 6;
    pub const GREATER_EQUAL: u32 = 7;
    pub const ALWAYS: u32 = 8;
}

/// Stencil operations for `begin_pass_full()` stencil parameters
pub mod stencil_op {
    pub const KEEP: u32 = 0;
    pub const ZERO: u32 = 1;
    pub const REPLACE: u32 = 2;
    pub const INCREMENT_CLAMP: u32 = 3;
    pub const DECREMENT_CLAMP: u32 = 4;
    pub const INVERT: u32 = 5;
    pub const INCREMENT_WRAP: u32 = 6;
    pub const DECREMENT_WRAP: u32 = 7;
}
