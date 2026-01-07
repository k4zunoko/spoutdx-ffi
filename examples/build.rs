use std::env;
use std::path::PathBuf;

fn main() {
    // Get the workspace root (two levels up from examples/)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(&manifest_dir);
    let workspace_root = manifest_path.parent().unwrap();
    
    // Path to the DLL build directory
    let dll_dir = workspace_root.join("out").join("build").join("msvc-debug").join("Debug");
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
        println!("cargo:warning=DLL not found at: {}", dll_path.display());
    }
    
    // Rerun if DLL changes
    println!("cargo:rerun-if-changed={}", dll_path.display());
}
