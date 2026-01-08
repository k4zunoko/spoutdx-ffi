use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
use windows::{
    Win32::Graphics::{
        Direct3D::*,
        Direct3D11::*,
        Dxgi::Common::*,
    },
};

// FFI declarations matching spoutdx_ffi.h

// Existing API
unsafe extern "C" {
    fn spoutdx_ffi_version() -> *const c_char;
    fn spoutdx_ffi_get_sdk_version() -> c_int;
    fn spoutdx_ffi_test_dx11_init() -> bool;
}

// Receiver API types
type SpoutDxReceiverHandle = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum SpoutDxResult {
    Ok = 0,
    ErrorNullHandle = -1,
    ErrorNullDevice = -2,
    ErrorNotConnected = -3,
    ErrorInitFailed = -4,
    ErrorReceiveFailed = -5,
    ErrorInternal = -99,
}

#[repr(C)]
#[derive(Debug)]
struct SpoutDxSenderInfo {
    name: [c_char; 256],
    width: c_uint,
    height: c_uint,
    format: c_uint,
}

// Receiver API functions
#[allow(dead_code)]
unsafe extern "C" {
    fn spoutdx_receiver_create() -> SpoutDxReceiverHandle;
    fn spoutdx_receiver_destroy(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_open_dx11(handle: SpoutDxReceiverHandle, device: *mut c_void) -> c_int;
    fn spoutdx_receiver_close_dx11(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_set_sender_name(handle: SpoutDxReceiverHandle, sender_name: *const c_char) -> c_int;
    fn spoutdx_receiver_receive_texture(handle: SpoutDxReceiverHandle, dst_texture: *mut c_void) -> c_int;
    fn spoutdx_receiver_release(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_get_sender_info(handle: SpoutDxReceiverHandle, out_info: *mut SpoutDxSenderInfo) -> c_int;
    fn spoutdx_receiver_is_updated(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_is_connected(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_is_frame_new(handle: SpoutDxReceiverHandle) -> c_int;
}

fn main() {
    unsafe {
        // ============================================================
        // Existing API tests
        // ============================================================
        
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

        println!();

        // ============================================================
        // Receiver API tests
        // ============================================================

        println!("Testing Receiver API:");

        // Create D3D11 device
        println!("  Creating D3D11 device...");
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        let feature_levels = [
            D3D_FEATURE_LEVEL_11_1,
            D3D_FEATURE_LEVEL_11_0,
        ];
        
        let result = D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_FLAG(0),
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        );

        if result.is_err() || device.is_none() {
            println!("  Failed to create D3D11 device");
            return;
        }

        let device = device.unwrap();
        println!("  D3D11 device created successfully");

        // Create receiver
        println!("  Creating receiver...");
        let receiver = spoutdx_receiver_create();
        if receiver.is_null() {
            println!("  Failed to create receiver");
            return;
        }
        println!("  Receiver created successfully");

        // Initialize receiver with external device
        println!("  Initializing receiver with D3D11 device...");
        let device_ptr = std::mem::transmute_copy(&device);
        let init_result = spoutdx_receiver_open_dx11(receiver, device_ptr);
        if init_result != 0 {
            println!("  Failed to initialize receiver (error code: {})", init_result);
            spoutdx_receiver_destroy(receiver);
            return;
        }
        println!("  Receiver initialized successfully");

        // Create a test texture (1920x1080 BGRA)
        println!("  Creating test texture (1920x1080)...");
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: 1920,
            Height: 1080,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: (D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_RENDER_TARGET.0) as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };

        let mut texture: Option<ID3D11Texture2D> = None;
        let result = device.CreateTexture2D(&texture_desc, None, Some(&mut texture));
        if result.is_err() || texture.is_none() {
            println!("  Failed to create texture");
            spoutdx_receiver_destroy(receiver);
            return;
        }
        let texture = texture.unwrap();
        println!("  Texture created successfully");

        // Try to receive texture
        println!("  Attempting to receive texture...");
        let texture_ptr: *mut c_void = std::mem::transmute_copy(&texture);
        let receive_result = spoutdx_receiver_receive_texture(receiver, texture_ptr);
        
        match receive_result {
            0 => {
                println!("  Successfully received texture!");
                
                // Get sender info
                let mut info = std::mem::zeroed::<SpoutDxSenderInfo>();
                if spoutdx_receiver_get_sender_info(receiver, &mut info) == 0 {
                    let name = CStr::from_ptr(info.name.as_ptr()).to_string_lossy();
                    println!("  Sender info:");
                    println!("    Name: {}", name);
                    println!("    Size: {}x{}", info.width, info.height);
                    println!("    Format: {}", info.format);
                }
            }
            -3 => {
                println!("  No sender available (SPOUTDX_ERROR_NOT_CONNECTED)");
                println!("  This is expected if no Spout sender is running.");
            }
            code => {
                println!("  Failed to receive texture (error code: {})", code);
            }
        }

        // Check connection status
        let is_connected = spoutdx_receiver_is_connected(receiver);
        println!("  Receiver connected: {}", is_connected != 0);

        // Check if updated (for size/format changes)
        let is_updated = spoutdx_receiver_is_updated(receiver);
        println!("  Receiver updated: {}", is_updated != 0);

        // Check if new frame
        let is_frame_new = spoutdx_receiver_is_frame_new(receiver);
        println!("  New frame: {}", is_frame_new != 0);

        // Test setting specific sender name (optional)
        // spoutdx_receiver_set_sender_name(receiver, std::ptr::null());

        // Test release (for reconnecting to different sender)
        // spoutdx_receiver_release(receiver);

        // Cleanup
        println!("  Cleaning up...");
        spoutdx_receiver_close_dx11(receiver);
        spoutdx_receiver_destroy(receiver);
        println!("  Receiver destroyed successfully");

        println!("\nReceiver API test completed.");
    }
}

