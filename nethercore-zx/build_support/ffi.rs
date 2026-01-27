//! Build-time checks related to FFI consistency.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

pub(crate) fn check_ffi_freshness() {
    let ffi_source_dir = Path::new("../include/zx");
    let c_header = Path::new("../include/zx.h");
    let zig_bindings = Path::new("../include/zx.zig");

    println!("cargo:rerun-if-changed=../include/zx");

    let source_time = match newest_mtime_in_dir(ffi_source_dir) {
        Some(t) => t,
        None => return,
    };

    let c_time = fs::metadata(c_header).and_then(|m| m.modified()).ok();
    let zig_time = fs::metadata(zig_bindings).and_then(|m| m.modified()).ok();

    let c_stale = c_time.is_none_or(|t| source_time > t);
    let zig_stale = zig_time.is_none_or(|t| source_time > t);

    if c_stale || zig_stale {
        panic!("FFI bindings are stale! Run: cargo xtask ffi generate");
    }
}

fn newest_mtime_in_dir(dir: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if newest.is_none_or(|n| mtime > n) {
                        newest = Some(mtime);
                    }
                }
            }
        }
    }
    newest
}
