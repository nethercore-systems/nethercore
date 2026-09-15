//! Build-time checks related to FFI consistency.

pub(crate) fn check_ffi_freshness() {
    for input in [
        "../include/zx",
        "../include/zx.rs",
        "../include/zx.h",
        "../include/zx.zig",
        "../tools/ffi-gen/templates",
    ] {
        println!("cargo:rerun-if-changed={input}");
    }

    // Checkout timestamps do not establish whether generated content is current.
    let in_sync = ffi_gen::check_for_console("zx").expect("Failed to verify ZX FFI bindings");
    assert!(
        in_sync,
        "FFI bindings are stale! Run: cargo xtask ffi generate"
    );
}
