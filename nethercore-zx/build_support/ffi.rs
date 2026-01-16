//! Build-time checks related to FFI consistency.

use std::fs;
use std::path::Path;

pub(crate) fn check_ffi_freshness() {
    let ffi_source = Path::new("../include/zx.rs");
    let c_header = Path::new("../include/zx.h");
    let zig_bindings = Path::new("../include/zx.zig");

    println!("cargo:rerun-if-changed=../include/zx.rs");

    let source_time = match fs::metadata(ffi_source).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return,
    };

    let c_time = fs::metadata(c_header).and_then(|m| m.modified()).ok();
    let zig_time = fs::metadata(zig_bindings).and_then(|m| m.modified()).ok();

    let c_stale = c_time.is_none_or(|t| source_time > t);
    let zig_stale = zig_time.is_none_or(|t| source_time > t);

    if c_stale || zig_stale {
        panic!("FFI bindings are stale! Run: cargo xtask ffi generate");
    }
}
