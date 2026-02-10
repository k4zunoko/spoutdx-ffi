#define SPOUTDX_FFI_EXPORTS
#include <spoutdx_ffi/spoutdx_ffi.h>

#include <Windows.h>

// Spout source integration
#include <SpoutDX.h>

#include <string>

// ============================================================
// Existing API
// ============================================================

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

// ============================================================
// Receiver API implementation
// ============================================================

// Internal receiver wrapper class (C++ only)
class SpoutDxReceiver {
public:
    spoutDX dx;  // SpoutDX instance

    SpoutDxReceiver() = default;
    ~SpoutDxReceiver() {
        dx.ReleaseReceiver();
        dx.CloseDirectX11();
    }
};

// -- Lifecycle --

SpoutDxReceiverHandle spoutdx_receiver_create() {
    try {
        return new SpoutDxReceiver();
    } catch (...) {
        return nullptr;
    }
}

int spoutdx_receiver_destroy(SpoutDxReceiverHandle handle) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        delete static_cast<SpoutDxReceiver*>(handle);
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}

// -- DirectX initialization --

int spoutdx_receiver_open_dx11(SpoutDxReceiverHandle handle, void* device) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    if (!device) return SPOUTDX_ERROR_NULL_DEVICE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        ID3D11Device* pDevice = static_cast<ID3D11Device*>(device);
        if (!rx->dx.OpenDirectX11(pDevice)) {
            return SPOUTDX_ERROR_INIT_FAILED;
        }
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}

int spoutdx_receiver_close_dx11(SpoutDxReceiverHandle handle) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        rx->dx.CloseDirectX11();
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}

// -- Receive configuration --

int spoutdx_receiver_set_sender_name(SpoutDxReceiverHandle handle, const char* sender_name) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        rx->dx.SetReceiverName(sender_name);  // NULL for active sender
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}

// -- Receive --

int spoutdx_receiver_receive_texture(SpoutDxReceiverHandle handle, void* dst_texture) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        ID3D11Texture2D* pTexture = static_cast<ID3D11Texture2D*>(dst_texture);
        ID3D11Texture2D* pTexturePtr = pTexture;

        // Use ReceiveTexture(ID3D11Texture2D**)
        // This function searches for sender and copies from shared texture to dst
        if (!rx->dx.ReceiveTexture(&pTexturePtr)) {
            if (!rx->dx.IsConnected()) {
                return SPOUTDX_ERROR_NOT_CONNECTED;
            }
            return SPOUTDX_ERROR_RECEIVE_FAILED;
        }
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}

int spoutdx_receiver_receive(SpoutDxReceiverHandle handle) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);

        // Use ReceiveTexture() - receives to internal class texture
        if (!rx->dx.ReceiveTexture()) {
            if (!rx->dx.IsConnected()) {
                return SPOUTDX_ERROR_NOT_CONNECTED;
            }
            return SPOUTDX_ERROR_RECEIVE_FAILED;
        }
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}

void* spoutdx_receiver_get_received_texture(SpoutDxReceiverHandle handle) {
    if (!handle) return nullptr;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        // GetSenderTexture returns the internally received class texture
        return static_cast<void*>(rx->dx.GetSenderTexture());
    } catch (...) {
        return nullptr;
    }
}

void* spoutdx_receiver_get_dx11_context(SpoutDxReceiverHandle handle) {
    if (!handle) return nullptr;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        return static_cast<void*>(rx->dx.GetDX11Context());
    } catch (...) {
        return nullptr;
    }
}

int spoutdx_receiver_release(SpoutDxReceiverHandle handle) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        rx->dx.ReleaseReceiver();
        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}

// -- State query --

int spoutdx_receiver_get_sender_info(SpoutDxReceiverHandle handle, SpoutDxSenderInfo* out_info) {
    if (!handle) return SPOUTDX_ERROR_NULL_HANDLE;
    if (!out_info) return SPOUTDX_ERROR_INTERNAL;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);

        // Get sender name
        const char* name = rx->dx.GetSenderName();
        if (name) {
            strncpy_s(out_info->name, sizeof(out_info->name), name, _TRUNCATE);
        } else {
            out_info->name[0] = '\0';
        }

        // Get size and format
        out_info->width = rx->dx.GetSenderWidth();
        out_info->height = rx->dx.GetSenderHeight();
        out_info->format = static_cast<unsigned int>(rx->dx.GetSenderFormat());

        return SPOUTDX_OK;
    } catch (...) {
        return SPOUTDX_ERROR_INTERNAL;
    }
}

int spoutdx_receiver_is_updated(SpoutDxReceiverHandle handle) {
    if (!handle) return 0;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        return rx->dx.IsUpdated() ? 1 : 0;
    } catch (...) {
        return 0;
    }
}

int spoutdx_receiver_is_connected(SpoutDxReceiverHandle handle) {
    if (!handle) return 0;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        return rx->dx.IsConnected() ? 1 : 0;
    } catch (...) {
        return 0;
    }
}

int spoutdx_receiver_is_frame_new(SpoutDxReceiverHandle handle) {
    if (!handle) return 0;
    try {
        auto* rx = static_cast<SpoutDxReceiver*>(handle);
        return rx->dx.IsFrameNew() ? 1 : 0;
    } catch (...) {
        return 0;
    }
}
