#pragma once

#ifdef _WIN32
  #ifdef SPOUTDX_FFI_EXPORTS
    #define SPOUTDX_FFI_API __declspec(dllexport)
  #else
    #define SPOUTDX_FFI_API __declspec(dllimport)
  #endif
#else
  #define SPOUTDX_FFI_API
#endif

extern "C" {

// Pure C ABI for Spout DirectX functionality (Rust FFI ready)
SPOUTDX_FFI_API const char* spoutdx_ffi_version();

// Returns Spout SDK version number (e.g. 2007 for "2.007").
// Built from Spout source code - no DLL ABI issues.
SPOUTDX_FFI_API int spoutdx_ffi_get_sdk_version();

// Test DirectX 11 device creation (initialization check).
// Returns 1 on success, 0 on failure.
SPOUTDX_FFI_API int spoutdx_ffi_test_dx11_init();

}
