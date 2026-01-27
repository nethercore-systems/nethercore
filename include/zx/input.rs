//! Input Functions — Buttons, Analog Sticks, and Triggers

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Check if a button is currently held.
    ///
    /// # Button indices
    /// 0=UP, 1=DOWN, 2=LEFT, 3=RIGHT, 4=A, 5=B, 6=X, 7=Y,
    /// 8=L1, 9=R1, 10=L3, 11=R3, 12=START, 13=SELECT
    ///
    /// # Returns
    /// 1 if held, 0 otherwise.
    pub fn button_held(player: u32, button: u32) -> u32;

    /// Check if a button was just pressed this tick.
    ///
    /// # Returns
    /// 1 if just pressed, 0 otherwise.
    pub fn button_pressed(player: u32, button: u32) -> u32;

    /// Check if a button was just released this tick.
    ///
    /// # Returns
    /// 1 if just released, 0 otherwise.
    pub fn button_released(player: u32, button: u32) -> u32;

    /// Get bitmask of all held buttons.
    pub fn buttons_held(player: u32) -> u32;

    /// Get bitmask of all buttons just pressed this tick.
    pub fn buttons_pressed(player: u32) -> u32;

    /// Get bitmask of all buttons just released this tick.
    pub fn buttons_released(player: u32) -> u32;

    /// Get left stick X axis value (-1.0 to 1.0).
    pub fn left_stick_x(player: u32) -> f32;

    /// Get left stick Y axis value (-1.0 to 1.0).
    pub fn left_stick_y(player: u32) -> f32;

    /// Get right stick X axis value (-1.0 to 1.0).
    pub fn right_stick_x(player: u32) -> f32;

    /// Get right stick Y axis value (-1.0 to 1.0).
    pub fn right_stick_y(player: u32) -> f32;

    /// Get both left stick axes at once (more efficient).
    ///
    /// Writes X and Y values to the provided pointers.
    pub fn left_stick(player: u32, out_x: *mut f32, out_y: *mut f32);

    /// Get both right stick axes at once (more efficient).
    ///
    /// Writes X and Y values to the provided pointers.
    pub fn right_stick(player: u32, out_x: *mut f32, out_y: *mut f32);

    /// Get left trigger value (0.0 to 1.0).
    pub fn trigger_left(player: u32) -> f32;

    /// Get right trigger value (0.0 to 1.0).
    pub fn trigger_right(player: u32) -> f32;
}
