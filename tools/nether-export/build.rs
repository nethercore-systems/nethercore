// Build script for nether-export
//
// This links the C++ standard library required by intel_tex_2's ISPC kernels.

fn main() {
    // Link C++ standard library for intel_tex_2's ISPC kernels
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=stdc++");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");
}
