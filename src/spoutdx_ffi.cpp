#define SPOUTDX_FFI_EXPORTS
#include <spoutdx_ffi/spoutdx_ffi.h>

#include <Windows.h>

// Spout source integration
#include <SpoutDX.h>

#include <string>

const char* spoutdx_ffi_version() {
  return "spoutdx-ffi/0.1.0";
}

int spoutdx_ffi_get_sdk_version() {
  // Get SDK version from Spout source (using int* to avoid std::string ABI issues)
  int version = 0;
  spoututils::GetSDKversion(&version);
  return version;
}

int spoutdx_ffi_test_dx11_init() {
  // Test DirectX 11 initialization using Spout source
  try {
    spoutDX dx;
    if (dx.OpenDirectX11()) {
      dx.CloseDirectX11();
      return 1;
    }
  } catch (...) {
    return 0;
  }
  return 0;
}
