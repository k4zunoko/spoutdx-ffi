#include <iostream>
#include <spoutdx_ffi/spoutdx_ffi.h>

int main() {
  std::cout << spoutdx_ffi_version() << "\n";
  
  int sdk_ver = spoutdx_ffi_get_sdk_version();
  if (sdk_ver > 0) {
    std::cout << "Spout SDK version: " << sdk_ver << "\n";
  } else {
    std::cout << "Spout SDK version: unknown\n";
  }

  std::cout << "Testing DirectX 11 initialization... ";
  if (spoutdx_ffi_test_dx11_init()) {
    std::cout << "OK\n";
  } else {
    std::cout << "PENDING (awaiting Spout source integration)\n";
  }
  
  return 0;
}
