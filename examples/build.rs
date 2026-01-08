use std::env;
use std::path::PathBuf;

fn main() {
    // Get the workspace root (two levels up from examples/)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(&manifest_dir);
    let workspace_root = manifest_path.parent().unwrap();
    
    // Decide which CMake preset/config to use.
    // By default, match Cargo profile:
    // - debug   -> msvc-debug / Debug
    // - release -> msvc-release / Release
    // You can override the preset via env var SPOUTDX_FFI_CMAKE_PRESET.
    let cargo_profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let default_preset = if cargo_profile.eq_ignore_ascii_case("release") {
        "msvc-release"
    } else {
        "msvc-debug"
    };
    let preset = env::var("SPOUTDX_FFI_CMAKE_PRESET").unwrap_or_else(|_| default_preset.to_string());

    let config = if preset.eq_ignore_ascii_case("msvc-release") {
        "Release"
    } else {
        // VS generators use "Debug" by default.
        "Debug"
    };

    // Path to the DLL build directory
    let dll_dir = workspace_root.join("out").join("build").join(&preset).join(config);
    let dll_path = dll_dir.join("spoutdx_ffi.dll");
    
    // Link against spoutdx_ffi.dll
    println!("cargo:rustc-link-search={}", dll_dir.display());
    println!("cargo:rustc-link-lib=spoutdx_ffi");
    
    // Copy DLL to output directory for runtime
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = PathBuf::from(&out_dir)
        .ancestors()
        .nth(3)  // Navigate up to target/debug or target/release
        .unwrap()
        .to_path_buf();
    
    let target_dll = target_dir.join("spoutdx_ffi.dll");
    if dll_path.exists() {
        std::fs::copy(&dll_path, &target_dll).expect("Failed to copy DLL");
        println!("cargo:warning=Copied DLL to: {}", target_dll.display());
    } else {
        println!(
            "cargo:warning=DLL not found at: {} (preset: {}, config: {}, cargo PROFILE: {})",
            dll_path.display(),
            preset,
            config,
            cargo_profile
        );
    }
    
    // Rerun if DLL changes
    println!("cargo:rerun-if-changed={}", dll_path.display());
}
