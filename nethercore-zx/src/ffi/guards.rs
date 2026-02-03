//! FFI guard functions for enforcing lifecycle constraints

use anyhow::{Result, bail};
use wasmtime::Caller;

use super::ZXGameContext;

/// Guard macro for init-only functions returning u32.
/// Returns 0 and logs a warning if called outside init().
///
/// # Usage
/// ```rust,ignore
/// fn load_texture(...) -> u32 {
///     guard_init_only!(caller, "load_texture");
///     // ... rest of function
/// }
/// ```
macro_rules! guard_init_only {
    ($caller:expr, $fn_name:expr) => {
        if let Err(e) = $crate::ffi::guards::check_init_only(&$caller, $fn_name) {
            tracing::warn!("{}", e);
            return 0;
        }
    };
}
pub(crate) use guard_init_only;

/// Check if we're in init phase (init-only function guard)
///
/// Returns an error if called after init() has completed.
///
/// # Usage
/// ```rust,ignore
/// // For functions returning Result<u32>:
/// fn load_mesh(...) -> Result<u32> {
///     check_init_only(&caller, "load_mesh")?;
///     // ... rest of function
/// }
///
/// // For functions returning u32, use the guard_init_only! macro instead.
/// ```
pub(crate) fn check_init_only(caller: &Caller<'_, ZXGameContext>, fn_name: &str) -> Result<()> {
    if !caller.data().game.in_init {
        bail!("{}: can only be called during init()", fn_name);
    }
    Ok(())
}
