use std::ffi::{c_char, c_int, CStr};

// FFI declarations matching spoutdx_ffi.h
unsafe extern "C" {
    fn spoutdx_ffi_version() -> *const c_char;
    fn spoutdx_ffi_get_sdk_version() -> c_int;
    fn spoutdx_ffi_test_dx11_init() -> bool;
}

fn main() {
    unsafe {
        // Get FFI version
        let version_ptr = spoutdx_ffi_version();
        let version = CStr::from_ptr(version_ptr).to_string_lossy();
        println!("{}", version);

        // Get SDK version
        let sdk_ver = spoutdx_ffi_get_sdk_version();
        if sdk_ver > 0 {
            println!("Spout SDK version: {}", sdk_ver);
        } else {
            println!("Spout SDK version: unknown");
        }

        // Test DirectX 11 initialization
        print!("Testing DirectX 11 initialization... ");
        if spoutdx_ffi_test_dx11_init() {
            println!("OK");
        } else {
            println!("PENDING (awaiting Spout source integration)");
        }
    }
}

