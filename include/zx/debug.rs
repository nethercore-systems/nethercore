//! Debug Inspection System
//!
//! Runtime value inspection and editing for development.
//! Press F3 to open panel. Zero overhead in release builds (compiles out).
//! All debug functions use length-prefixed strings: (name_ptr, name_len).

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    // --- Primitive Type Registration (Editable) ---

    /// Register an i8 value for debug inspection.
    pub fn debug_register_i8(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register an i16 value for debug inspection.
    pub fn debug_register_i16(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register an i32 value for debug inspection.
    pub fn debug_register_i32(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register a u8 value for debug inspection.
    pub fn debug_register_u8(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register a u16 value for debug inspection.
    pub fn debug_register_u16(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register a u32 value for debug inspection.
    pub fn debug_register_u32(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register an f32 value for debug inspection.
    pub fn debug_register_f32(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register a bool value for debug inspection.
    pub fn debug_register_bool(name_ptr: *const u8, name_len: u32, ptr: *const u8);

    // --- Range-Constrained Registration (Slider UI) ---

    /// Register an i32 with min/max range constraints.
    pub fn debug_register_i32_range(
        name_ptr: *const u8,
        name_len: u32,
        ptr: *const u8,
        min: i32,
        max: i32,
    );
    /// Register an f32 with min/max range constraints.
    pub fn debug_register_f32_range(
        name_ptr: *const u8,
        name_len: u32,
        ptr: *const u8,
        min: f32,
        max: f32,
    );
    /// Register a u8 with min/max range constraints.
    pub fn debug_register_u8_range(
        name_ptr: *const u8,
        name_len: u32,
        ptr: *const u8,
        min: u32,
        max: u32,
    );
    /// Register a u16 with min/max range constraints.
    pub fn debug_register_u16_range(
        name_ptr: *const u8,
        name_len: u32,
        ptr: *const u8,
        min: u32,
        max: u32,
    );
    /// Register an i16 with min/max range constraints.
    pub fn debug_register_i16_range(
        name_ptr: *const u8,
        name_len: u32,
        ptr: *const u8,
        min: i32,
        max: i32,
    );

    // --- Compound Type Registration (Editable) ---

    /// Register a Vec2 (2 floats: x, y) for debug inspection.
    pub fn debug_register_vec2(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register a Vec3 (3 floats: x, y, z) for debug inspection.
    pub fn debug_register_vec3(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register a Rect (4 i16: x, y, w, h) for debug inspection.
    pub fn debug_register_rect(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register a Color (4 u8: RGBA) for debug inspection with color picker.
    pub fn debug_register_color(name_ptr: *const u8, name_len: u32, ptr: *const u8);

    // --- Fixed-Point Type Registration (Editable) ---

    /// Register Q8.8 fixed-point (i16) for debug inspection.
    pub fn debug_register_fixed_i16_q8(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register Q16.16 fixed-point (i32) for debug inspection.
    pub fn debug_register_fixed_i32_q16(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register Q24.8 fixed-point (i32) for debug inspection.
    pub fn debug_register_fixed_i32_q8(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Register Q8.24 fixed-point (i32) for debug inspection.
    pub fn debug_register_fixed_i32_q24(name_ptr: *const u8, name_len: u32, ptr: *const u8);

    // --- Watch Functions (Read-Only Display) ---

    /// Watch an i8 value (read-only).
    pub fn debug_watch_i8(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch an i16 value (read-only).
    pub fn debug_watch_i16(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch an i32 value (read-only).
    pub fn debug_watch_i32(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch a u8 value (read-only).
    pub fn debug_watch_u8(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch a u16 value (read-only).
    pub fn debug_watch_u16(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch a u32 value (read-only).
    pub fn debug_watch_u32(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch an f32 value (read-only).
    pub fn debug_watch_f32(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch a bool value (read-only).
    pub fn debug_watch_bool(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch a Vec2 value (read-only).
    pub fn debug_watch_vec2(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch a Vec3 value (read-only).
    pub fn debug_watch_vec3(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch a Rect value (read-only).
    pub fn debug_watch_rect(name_ptr: *const u8, name_len: u32, ptr: *const u8);
    /// Watch a Color value (read-only).
    pub fn debug_watch_color(name_ptr: *const u8, name_len: u32, ptr: *const u8);

    // --- Grouping Functions ---

    /// Begin a collapsible group in the debug UI.
    pub fn debug_group_begin(name_ptr: *const u8, name_len: u32);
    /// End the current debug group.
    pub fn debug_group_end();

    // --- Action Registration (Buttons) ---

    /// Register a simple action with no parameters.
    ///
    /// Creates a button in the debug UI that calls the specified WASM function when clicked.
    ///
    /// # Parameters
    /// - `name_ptr`: Pointer to button label string
    /// - `name_len`: Length of button label
    /// - `func_name_ptr`: Pointer to WASM function name string
    /// - `func_name_len`: Length of function name
    pub fn debug_register_action(
        name_ptr: *const u8,
        name_len: u32,
        func_name_ptr: *const u8,
        func_name_len: u32,
    );

    /// Begin building an action with parameters.
    ///
    /// Use with debug_action_param_* and debug_action_end() to create an action with input fields.
    ///
    /// # Parameters
    /// - `name_ptr`: Pointer to button label string
    /// - `name_len`: Length of button label
    /// - `func_name_ptr`: Pointer to WASM function name string
    /// - `func_name_len`: Length of function name
    pub fn debug_action_begin(
        name_ptr: *const u8,
        name_len: u32,
        func_name_ptr: *const u8,
        func_name_len: u32,
    );

    /// Add an i32 parameter to the pending action.
    ///
    /// # Parameters
    /// - `name_ptr`: Pointer to parameter label string
    /// - `name_len`: Length of parameter label
    /// - `default_value`: Default value for the parameter
    pub fn debug_action_param_i32(name_ptr: *const u8, name_len: u32, default_value: i32);

    /// Add an f32 parameter to the pending action.
    ///
    /// # Parameters
    /// - `name_ptr`: Pointer to parameter label string
    /// - `name_len`: Length of parameter label
    /// - `default_value`: Default value for the parameter
    pub fn debug_action_param_f32(name_ptr: *const u8, name_len: u32, default_value: f32);

    /// Finish building the pending action.
    ///
    /// Completes the action registration started with debug_action_begin().
    pub fn debug_action_end();

    // --- State Query Functions ---

    /// Query if the game is currently paused (debug mode).
    ///
    /// # Returns
    /// 1 if paused, 0 if running normally.
    pub fn debug_is_paused() -> i32;

    /// Get the current time scale multiplier.
    ///
    /// # Returns
    /// 1.0 = normal, 0.5 = half-speed, 2.0 = double-speed, etc.
    pub fn debug_get_time_scale() -> f32;
}
