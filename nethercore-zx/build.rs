//! Build script for nethercore-zx.
//!
//! The heavy lifting lives in `build_support/` to keep this entrypoint small.

mod build_support;

fn main() {
    build_support::run();
}
