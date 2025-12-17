//! FFI guard functions for enforcing lifecycle constraints

use anyhow::{Result, bail};
use wasmtime::Caller;

use super::ZGameContext;

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
/// // For functions returning u32:
/// fn load_texture(...) -> u32 {
///     if let Err(e) = check_init_only(&caller, "load_texture") {
///         tracing::warn!("{}", e);
///         return 0;
///     }
///     // ... rest of function
/// }
/// ```
pub(crate) fn check_init_only(caller: &Caller<'_, ZGameContext>, fn_name: &str) -> Result<()> {
    if !caller.data().game.in_init {
        bail!("{}: can only be called during init()", fn_name);
    }
    Ok(())
}
