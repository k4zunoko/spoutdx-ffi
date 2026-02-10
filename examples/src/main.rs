use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
use std::path::Path;
use windows::{
    core::PCWSTR,
    Win32::Graphics::{
        Direct3D::*,
        Direct3D11::*,
        Dxgi::Common::*,
        Imaging::*,
    },
    Win32::System::Com::*,
};

// FFI declarations matching spoutdx_ffi.h

// Existing API
unsafe extern "C" {
    fn spoutdx_ffi_version() -> *const c_char;
    fn spoutdx_ffi_get_sdk_version() -> c_int;
    fn spoutdx_ffi_test_dx11_init() -> c_int;
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
    fn spoutdx_receiver_receive(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_get_received_texture(handle: SpoutDxReceiverHandle) -> *mut c_void;
    fn spoutdx_receiver_get_dx11_context(handle: SpoutDxReceiverHandle) -> *mut c_void;
    fn spoutdx_receiver_release(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_get_sender_info(handle: SpoutDxReceiverHandle, out_info: *mut SpoutDxSenderInfo) -> c_int;
    fn spoutdx_receiver_is_updated(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_is_connected(handle: SpoutDxReceiverHandle) -> c_int;
    fn spoutdx_receiver_is_frame_new(handle: SpoutDxReceiverHandle) -> c_int;
}

// ============================================================
// 画像診断・ダンプ機能
// ============================================================

/// 平均色を計算（BGRA フォーマット）
fn calculate_average_color(data: &[u8], width: u32, height: u32, row_pitch: u32) -> (f64, f64, f64, f64) {
    let mut total_b: u64 = 0;
    let mut total_g: u64 = 0;
    let mut total_r: u64 = 0;
    let mut total_a: u64 = 0;
    let pixel_count = (width * height) as u64;

    for y in 0..height {
        let row_start = (y * row_pitch) as usize;
        for x in 0..width {
            let pixel_offset = row_start + (x * 4) as usize;
            total_b += data[pixel_offset] as u64;
            total_g += data[pixel_offset + 1] as u64;
            total_r += data[pixel_offset + 2] as u64;
            total_a += data[pixel_offset + 3] as u64;
        }
    }

    let avg_b = total_b as f64 / pixel_count as f64;
    let avg_g = total_g as f64 / pixel_count as f64;
    let avg_r = total_r as f64 / pixel_count as f64;
    let avg_a = total_a as f64 / pixel_count as f64;

    (avg_r, avg_g, avg_b, avg_a)
}

/// 画像診断を実行
fn diagnose_image(data: &[u8], width: u32, height: u32, row_pitch: u32) {
    let (avg_r, avg_g, avg_b, avg_a) = calculate_average_color(data, width, height, row_pitch);

    println!("\n  [Image Diagnostics]");
    println!("    Average color (RGBA): ({:.1}, {:.1}, {:.1}, {:.1})", avg_r, avg_g, avg_b, avg_a);

    // 全黒チェック
    if avg_r < 1.0 && avg_g < 1.0 && avg_b < 1.0 {
        println!("    ⚠️  WARNING: Image appears to be ALL BLACK!");
    }

    // 全白チェック
    if avg_r > 254.0 && avg_g > 254.0 && avg_b > 254.0 {
        println!("    ⚠️  WARNING: Image appears to be ALL WHITE!");
    }

    // アルファ誤りチェック（アルファが0または非常に低い）
    if avg_a < 1.0 {
        println!("    ⚠️  WARNING: Alpha channel is near ZERO - image may be fully transparent!");
    } else if avg_a < 128.0 {
        println!("    ⚠️  WARNING: Alpha channel average is low ({:.1}) - possible alpha issue", avg_a);
    } else if avg_a > 254.0 {
        println!("    ✓ Alpha channel is fully opaque (expected)");
    }

    // ガンマ/並べ替えミスチェック（極端な色偏り）
    let color_diff_rg = (avg_r - avg_g).abs();
    let color_diff_rb = (avg_r - avg_b).abs();
    let color_diff_gb = (avg_g - avg_b).abs();

    // 極端に一色だけ違う場合は RGB/BGR 入れ替わりの可能性
    if color_diff_rg > 100.0 || color_diff_rb > 100.0 || color_diff_gb > 100.0 {
        println!("    ⚠️  NOTE: Large color channel imbalance detected");
        println!("       This could indicate RGB/BGR swap or gamma issues");
        println!("       R-G diff: {:.1}, R-B diff: {:.1}, G-B diff: {:.1}",
                 color_diff_rg, color_diff_rb, color_diff_gb);
    }

    // サンプルピクセル表示（四隅と中心）
    println!("\n    Sample pixels (BGRA format in memory):");
    let show_pixel = |name: &str, x: u32, y: u32| {
        let offset = (y * row_pitch + x * 4) as usize;
        let b = data[offset];
        let g = data[offset + 1];
        let r = data[offset + 2];
        let a = data[offset + 3];
        println!("      {}: R={:3} G={:3} B={:3} A={:3}", name, r, g, b, a);
    };

    show_pixel("Top-Left    ", 0, 0);
    show_pixel("Top-Right   ", width - 1, 0);
    show_pixel("Center      ", width / 2, height / 2);
    show_pixel("Bottom-Left ", 0, height - 1);
    show_pixel("Bottom-Right", width - 1, height - 1);
}

/// WIC を使って PNG 形式で保存（BGRA -> RGBA 変換）
fn save_as_png_wic(path: &Path, data: &[u8], width: u32, height: u32, row_pitch: u32) -> windows::core::Result<()> {
    unsafe {
        // COM 初期化（既に初期化されている場合は S_FALSE が返るが、ok() で無視）
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        // WIC Factory 作成
        let factory: IWICImagingFactory = CoCreateInstance(
            &CLSID_WICImagingFactory,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        // ストリーム作成
        let stream = factory.CreateStream()?;

        // パスを PCWSTR に変換
        let path_str = path.to_string_lossy().to_string();
        let wide_path: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
        const GENERIC_WRITE: u32 = 0x40000000;
        stream.InitializeFromFilename(PCWSTR(wide_path.as_ptr()), GENERIC_WRITE)?;

        // PNG エンコーダー作成
        let encoder = factory.CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null())?;
        encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;

        // フレーム作成
        let mut frame: Option<IWICBitmapFrameEncode> = None;
        let mut property_bag: Option<windows::Win32::System::Com::StructuredStorage::IPropertyBag2> = None;
        encoder.CreateNewFrame(&mut frame, &mut property_bag)?;
        let frame = frame.unwrap();
        frame.Initialize(property_bag.as_ref())?;
        frame.SetSize(width, height)?;

        // ピクセルフォーマット設定（WIC は要求を変更する可能性がある）
        let requested_format = GUID_WICPixelFormat32bppRGBA;
        let mut pixel_format = requested_format;
        frame.SetPixelFormat(&mut pixel_format)?;

        // WIC が実際に採用したフォーマットを確認
        let format_name = if pixel_format == GUID_WICPixelFormat32bppRGBA {
            "RGBA"
        } else if pixel_format == GUID_WICPixelFormat32bppBGRA {
            "BGRA"
        } else {
            "Unknown"
        };
        println!("  [WIC] Requested: RGBA, Actual: {}", format_name);

        // 実際のフォーマットに応じてデータを準備
        let write_data: Vec<u8> = if pixel_format == GUID_WICPixelFormat32bppBGRA {
            // WIC が BGRA を要求 → そのまま渡す（変換不要）
            println!("  [WIC] Using BGRA directly (no conversion)");
            let mut bgra_data = vec![0u8; (width * height * 4) as usize];
            for y in 0..height {
                let src_row_start = (y * row_pitch) as usize;
                let dst_row_start = (y * width * 4) as usize;
                for x in 0..width {
                    let src_offset = src_row_start + (x * 4) as usize;
                    let dst_offset = dst_row_start + (x * 4) as usize;
                    if src_offset + 3 < data.len() && dst_offset + 3 < bgra_data.len() {
                        bgra_data[dst_offset..dst_offset + 4].copy_from_slice(&data[src_offset..src_offset + 4]);
                    }
                }
            }
            bgra_data
        } else {
            // WIC が RGBA を要求 → BGRA から変換
            println!("  [WIC] Converting BGRA -> RGBA");
            let mut rgba_data = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let src_row_start = (y * row_pitch) as usize;
            let dst_row_start = (y * width * 4) as usize;
            for x in 0..width {
                let src_offset = src_row_start + (x * 4) as usize;
                let dst_offset = dst_row_start + (x * 4) as usize;

                    if src_offset + 3 < data.len() && dst_offset + 3 < rgba_data.len() {
                        // BGRA (memory) -> RGBA (PNG)
                        rgba_data[dst_offset] = data[src_offset + 2];     // R
                        rgba_data[dst_offset + 1] = data[src_offset + 1]; // G
                        rgba_data[dst_offset + 2] = data[src_offset];     // B
                        rgba_data[dst_offset + 3] = data[src_offset + 3]; // A
                    }
                }
            }
            rgba_data
        };

        // 書き込み
        frame.WritePixels(height, width * 4, &write_data)?;
        frame.Commit()?;
        encoder.Commit()?;

        // COM は自動的に解放される（IUnknown の Drop）

        Ok(())
    }
}

/// Staging テクスチャを作成
fn create_staging_texture(
    device: &ID3D11Device,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
) -> windows::core::Result<ID3D11Texture2D> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_STAGING,
        BindFlags: 0,
        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
        MiscFlags: 0,
    };

    let mut texture: Option<ID3D11Texture2D> = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture))? };
    texture.ok_or_else(|| windows::core::Error::from_win32())
}

/// テクスチャから CPU メモリへコピーして画像を取得
fn read_texture_to_cpu(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    source_texture: &ID3D11Texture2D,
    width: u32,
    height: u32,
) -> windows::core::Result<(Vec<u8>, u32)> {
    unsafe {
        // ソーステクスチャのフォーマットを取得
        let mut src_desc = std::mem::zeroed::<D3D11_TEXTURE2D_DESC>();
        source_texture.GetDesc(&mut src_desc);
        let format = src_desc.Format;

        let format_name = match format.0 {
            87 => "DXGI_FORMAT_B8G8R8A8_UNORM (BGRA)",
            28 => "DXGI_FORMAT_R8G8B8A8_UNORM (RGBA)",
            _ => "Unknown",
        };
        println!("  [D3D11] Source texture format: {} ({})", format.0, format_name);

        // Staging テクスチャ作成（同じフォーマットで）
        let staging = create_staging_texture(device, width, height, format)?;

        // GPU テクスチャから Staging テクスチャへコピー
        context.CopyResource(&staging, source_texture);

        // GPU コマンドの完了を待つ（重要！）
        context.Flush();

        // Map して CPU からアクセス
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;

        let row_pitch = mapped.RowPitch;
        let data_size = (row_pitch * height) as usize;

        // データをコピー
        let src_ptr = mapped.pData as *const u8;
        let mut data = vec![0u8; data_size];
        std::ptr::copy_nonoverlapping(src_ptr, data.as_mut_ptr(), data_size);

        // Unmap
        context.Unmap(&staging, 0);

        Ok((data, row_pitch))
    }
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
        if spoutdx_ffi_test_dx11_init() != 0 {
            println!("OK");
        } else {
            println!("PENDING (awaiting Spout source integration)");
        }

        println!();

        // ============================================================
        // Receiver API tests with image dump
        // ============================================================

        println!("Testing Receiver API with image capture:");

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

        if result.is_err() || device.is_none() || context.is_none() {
            println!("  Failed to create D3D11 device");
            return;
        }

        let device = device.unwrap();
        let _context = context.unwrap(); // Rust側のコンテキストは使わない（SpoutDXのコンテキストを使う）
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

        // ============================================================
        // Spout 受信ループ（新方式：内部テクスチャ使用）
        // SpoutDX の受信フロー:
        // 1. spoutdx_receiver_receive() を呼ぶとセンダーを検索し、内部テクスチャに受信
        // 2. 新規接続/サイズ変更時は IsUpdated()=true
        // 3. IsUpdated() を呼ぶとフラグがリセットされる
        // 4. spoutdx_receiver_get_received_texture() で内部テクスチャを取得
        // 5. spoutdx_receiver_get_dx11_context() でコンテキストを取得してコピー
        // ============================================================

        println!("  Probing for sender...");

        let mut current_width = 0u32;
        let mut current_height = 0u32;

        // 受信ループ（最大10回試行）
        let mut frame_received = false;
        for attempt in 1..=10 {
            // 内部テクスチャへ受信
            let receive_result = spoutdx_receiver_receive(receiver);

            if receive_result != 0 {
                if receive_result == -3 {
                    println!("  No sender available (SPOUTDX_ERROR_NOT_CONNECTED)");
                    println!("  Please start a Spout sender and try again.");
                    spoutdx_receiver_destroy(receiver);
                    return;
                }
                println!("  Attempt {}: receive failed (error code: {})", attempt, receive_result);
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }

            // センダー変更チェック（初回接続時も含む）
            let is_updated = spoutdx_receiver_is_updated(receiver);

            // センダー情報取得（IsUpdated に関わらず）
            let mut info = std::mem::zeroed::<SpoutDxSenderInfo>();
            if spoutdx_receiver_get_sender_info(receiver, &mut info) == 0 {
                if current_width != info.width || current_height != info.height {
                    let sender_name = CStr::from_ptr(info.name.as_ptr()).to_string_lossy();
                    println!("  Attempt {}: Connected to sender: {}", attempt, sender_name);
                    println!("    Size: {}x{}", info.width, info.height);
                    println!("    Format: {} (DXGI_FORMAT)", info.format);
                    current_width = info.width;
                    current_height = info.height;
                }
            }

            if is_updated != 0 {
                // IsUpdated が true だったので、もう一度受信を試す
                println!("  Sender updated, retrying receive...");
                continue;
            }

            // サイズが取得できていなければスキップ
            if current_width == 0 || current_height == 0 {
                println!("  Attempt {}: No valid size yet, retrying...", attempt);
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }

            // フレーム受信成功！
            let is_frame_new = spoutdx_receiver_is_frame_new(receiver);
            if is_frame_new != 0 {
                println!("  Attempt {}: Frame received successfully! (new frame)", attempt);
                frame_received = true;
                break;
            } else {
                println!("  Attempt {}: Frame received but not new, retrying...", attempt);
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
        }

        if !frame_received {
            println!("  Failed to receive frame after multiple attempts");
            spoutdx_receiver_destroy(receiver);
            return;
        }

        // 内部テクスチャとコンテキストを取得
        let received_texture_ptr = spoutdx_receiver_get_received_texture(receiver);
        let spout_context_ptr = spoutdx_receiver_get_dx11_context(receiver);

        if received_texture_ptr.is_null() {
            println!("  Failed to get received texture");
            spoutdx_receiver_destroy(receiver);
            return;
        }

        if spout_context_ptr.is_null() {
            println!("  Failed to get DX11 context");
            spoutdx_receiver_destroy(receiver);
            return;
        }

        println!("  Got internal texture and context from SpoutDX");

        // SpoutDX のコンテキストを使って CPU コピー
        println!("  Copying to CPU memory (using SpoutDX context)...");
        let spout_context: ID3D11DeviceContext = std::mem::transmute(spout_context_ptr);
        let received_texture: ID3D11Texture2D = std::mem::transmute(received_texture_ptr);

        match read_texture_to_cpu(&device, &spout_context, &received_texture, current_width, current_height) {
            Ok((data, row_pitch)) => {
                println!("  CPU copy successful (row_pitch: {})", row_pitch);

                // 画像診断
                diagnose_image(&data, current_width, current_height, row_pitch);

                // PNG 保存
                let output_dir = Path::new(".");
                let png_path = output_dir.join("spout_capture.png");
                match save_as_png_wic(&png_path.as_path(), &data, current_width, current_height, row_pitch) {
                    Ok(()) => println!("\n  ✓ Saved PNG: {} (RGBA with alpha)", png_path.display()),
                    Err(e) => println!("\n  ✗ Failed to save PNG: {:?}", e),
                }
            }
            Err(e) => {
                println!("  Failed to copy to CPU: {:?}", e);
            }
        }

        // 状態表示
        println!("\n  [Connection Status]");
        let is_connected = spoutdx_receiver_is_connected(receiver);
        println!("    Connected: {}", is_connected != 0);
        let is_updated = spoutdx_receiver_is_updated(receiver);
        println!("    Updated: {}", is_updated != 0);
        let is_frame_new = spoutdx_receiver_is_frame_new(receiver);
        println!("    New frame: {}", is_frame_new != 0);

        // Cleanup
        println!("\n  Cleaning up...");
        spoutdx_receiver_close_dx11(receiver);
        spoutdx_receiver_destroy(receiver);
        println!("  Receiver destroyed successfully");

        println!("\n========================================");
        println!("Receiver API test completed.");
        println!("Check spout_capture.png for visual verification.");
        println!("========================================");
    }
}

/// レンダーターゲット用テクスチャを作成（将来の receive_texture API 用）
#[allow(dead_code)]
fn create_render_texture(device: &ID3D11Device, width: u32, height: u32) -> Option<ID3D11Texture2D> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
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
    unsafe {
        if device.CreateTexture2D(&desc, None, Some(&mut texture)).is_ok() {
            texture
        } else {
            None
        }
    }
}
