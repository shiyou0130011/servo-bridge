use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_uchar, c_uint};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use dpi::PhysicalSize;
use euclid::{Point2D, Size2D};
use servo::{
    DeviceIntRect, DevicePoint, InputEvent, Key, KeyState, KeyboardEvent, Modifiers,
    MouseButton as ServoMouseButton, MouseButtonAction, MouseButtonEvent, MouseMoveEvent,
    NamedKey, RenderingContext, Servo, ServoBuilder, SoftwareRenderingContext, WebView,
    WebViewBuilder, WheelDelta, WheelEvent, WheelMode,
};
use url::Url;

/// ABI 版本號，每次不相容 C ABI 變更時遞增。
pub const SERVO_BRIDGE_ABI_VERSION: u32 = 2;

const ERR_NULL_WEBVIEW: i32 = -1;
const ERR_INVALID_ARGUMENT: i32 = -2;
const ERR_INVALID_URL: i32 = -3;
const ERR_INVALID_UTF8: i32 = -4;
const ERR_RENDER_UNAVAILABLE: i32 = -5;
const ERR_PANIC: i32 = -99;

pub struct ServoWebView {
    servo: Servo,
    rendering_context: Rc<SoftwareRenderingContext>,
    webview: WebView,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    last_error: Option<CString>,
}

fn set_error(webview: &mut ServoWebView, message: &str) {
    webview.last_error = CString::new(message).ok();
}

// ---------------------------------------------------------------------------
// ABI 版本
// ---------------------------------------------------------------------------

/// 回傳目前 C ABI 版本號。
///
/// Go / C 呼叫端可在載入 DLL 後呼叫此函式，確認 ABI 版本是否相容。
#[no_mangle]
pub extern "C" fn servo_bridge_version() -> u32 {
    SERVO_BRIDGE_ABI_VERSION
}

// ---------------------------------------------------------------------------
// 建立與釋放
// ---------------------------------------------------------------------------

/// 建立新的 `ServoWebView`。
///
/// 成功時回傳不透明指標；`width` 或 `height` 為 0，或初始化失敗時回傳 NULL。
#[no_mangle]
pub extern "C" fn servo_webview_new(width: u32, height: u32) -> *mut ServoWebView {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if width == 0 || height == 0 {
            return std::ptr::null_mut();
        }
        let pixel_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|len| len.checked_mul(4));
        let Some(pixel_len) = pixel_len else {
            return std::ptr::null_mut();
        };

        let rendering_context =
            match SoftwareRenderingContext::new(PhysicalSize::new(width, height)) {
                Ok(context) => Rc::new(context),
                Err(_) => return std::ptr::null_mut(),
            };

        let servo = ServoBuilder::default().build();
        let url = Url::parse("about:blank").expect("about:blank is a valid URL");
        let webview = WebViewBuilder::new(&servo, rendering_context.clone())
            .url(url)
            .build();

        Box::into_raw(Box::new(ServoWebView {
            servo,
            rendering_context,
            webview,
            width,
            height,
            pixels: vec![0; pixel_len],
            last_error: None,
        }))
    }));
    result.unwrap_or(std::ptr::null_mut())
}

/// 釋放 `ServoWebView`。
///
/// `webview` 為 NULL 時什麼都不做。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_free(webview: *mut ServoWebView) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !webview.is_null() {
            drop(Box::from_raw(webview));
        }
    }));
}

// ---------------------------------------------------------------------------
// 屬性
// ---------------------------------------------------------------------------

/// 回傳目前渲染寬度（像素）。`webview` 為 NULL 時回傳 0。
#[no_mangle]
pub extern "C" fn servo_webview_width(webview: *const ServoWebView) -> u32 {
    catch_unwind(AssertUnwindSafe(|| unsafe { 
        webview.as_ref().map_or(0, |wv| wv.width) 
    })).unwrap_or(0)
}

/// 回傳目前渲染高度（像素）。`webview` 為 NULL 時回傳 0。
#[no_mangle]
pub extern "C" fn servo_webview_height(webview: *const ServoWebView) -> u32 {
    catch_unwind(AssertUnwindSafe(|| unsafe { 
        webview.as_ref().map_or(0, |wv| wv.height) 
    })).unwrap_or(0)
}

/// 回傳最後一次錯誤訊息的 C 字串指標。
///
/// 無錯誤時回傳 NULL。回傳的指標只在 `webview` 有效且未被修改前有效；
/// 呼叫端不可釋放此指標。
#[no_mangle]
pub extern "C" fn servo_webview_last_error(webview: *const ServoWebView) -> *const c_char {
    catch_unwind(AssertUnwindSafe(|| unsafe {
        webview
            .as_ref()
            .and_then(|wv| wv.last_error.as_ref())
            .map_or(std::ptr::null(), |e| e.as_ptr())
    })).unwrap_or(std::ptr::null())
}

// ---------------------------------------------------------------------------
// 導覽
// ---------------------------------------------------------------------------

/// 載入指定 URL。
///
/// - `url` 必須是合法的 UTF-8、以 NUL 結尾的 C 字串。
/// - 成功回傳 `0`；失敗回傳負值，可用 `servo_webview_last_error` 取得訊息。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_load_url(
    webview: *mut ServoWebView,
    url: *const c_char,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(webview) = webview.as_mut() else {
            return ERR_NULL_WEBVIEW;
        };
        webview.last_error = None;
        if url.is_null() {
            set_error(webview, "URL 指標不可為空值");
            return ERR_INVALID_ARGUMENT;
        }
        let url = match CStr::from_ptr(url).to_str() {
            Ok(u) => u,
            Err(_) => {
                set_error(webview, "URL 必須是合法 UTF-8");
                return ERR_INVALID_UTF8;
            },
        };
        let url = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => {
                set_error(webview, "URL 格式無效");
                return ERR_INVALID_URL;
            },
        };
        webview.webview.load(url);
        0
    }))
    .unwrap_or(ERR_PANIC)
}

// ---------------------------------------------------------------------------
// 更新與渲染
// ---------------------------------------------------------------------------

/// 驅動 Servo 事件迴圈，處理一批待辦事件（網路、排版、動畫等）。
///
/// Ebiten 的 `Update()` 每幀應呼叫一次。
/// 成功回傳 `0`；失敗回傳負值。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_update(webview: *mut ServoWebView) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(webview) = webview.as_mut() else {
            return ERR_NULL_WEBVIEW;
        };
        webview.last_error = None;
        webview.servo.spin_event_loop();
        0
    }))
    .unwrap_or(ERR_PANIC)
}

/// 讀取最新渲染畫面到呼叫端提供的 RGBA8 緩衝區。
///
/// `output` 必須至少有 `width * height * 4` bytes。
/// 成功回傳 `0`；失敗回傳負值。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_render(
    webview: *mut ServoWebView,
    output: *mut c_uchar,
    output_len: usize,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(webview) = webview.as_mut() else {
            return ERR_NULL_WEBVIEW;
        };
        webview.last_error = None;
        if output.is_null() || output_len < webview.pixels.len() {
            set_error(webview, "輸出緩衝區太小");
            return ERR_INVALID_ARGUMENT;
        }

        webview.webview.paint();
        let rectangle = DeviceIntRect::from_origin_and_size(
            Point2D::zero(),
            Size2D::new(webview.width as i32, webview.height as i32),
        );
        let Some(image) = webview.rendering_context.read_to_image(rectangle) else {
            set_error(webview, "目前尚未有可讀取的渲染畫面");
            return ERR_RENDER_UNAVAILABLE;
        };
        if image.as_raw().len() != webview.pixels.len() {
            set_error(webview, "渲染畫面大小與 WebView 不一致");
            return ERR_RENDER_UNAVAILABLE;
        }
        webview.pixels.copy_from_slice(image.as_raw());
        std::ptr::copy_nonoverlapping(webview.pixels.as_ptr(), output, webview.pixels.len());
        0
    }))
    .unwrap_or(ERR_PANIC)
}

// ---------------------------------------------------------------------------
// Resize
// ---------------------------------------------------------------------------

/// 調整 WebView 的渲染尺寸。
///
/// `width` 與 `height` 至少需為 1。成功回傳 `0`；失敗回傳負值。
/// 呼叫後應呼叫 `servo_webview_update` 再呼叫 `servo_webview_render` 才能取得
/// 新尺寸的畫面。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_resize(
    webview: *mut ServoWebView,
    width: u32,
    height: u32,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(webview) = webview.as_mut() else {
            return ERR_NULL_WEBVIEW;
        };
        if width == 0 || height == 0 {
            set_error(webview, "寬度與高度必須大於 0");
            return ERR_INVALID_ARGUMENT;
        }
        let pixel_len = match (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
        {
            Some(n) => n,
            None => {
                set_error(webview, "尺寸溢位");
                return ERR_INVALID_ARGUMENT;
            },
        };
        webview.webview.resize(PhysicalSize::new(width, height));
        webview.width = width;
        webview.height = height;
        webview.pixels.resize(pixel_len, 0);
        0
    }))
    .unwrap_or(ERR_PANIC)
}

// ---------------------------------------------------------------------------
// 輸入事件
// ---------------------------------------------------------------------------

/// 通知 Servo 滑鼠游標移動事件。
///
/// `x` 與 `y` 為相對於 WebView 左上角的裝置像素座標。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_mouse_move(
    webview: *mut ServoWebView,
    x: c_float,
    y: c_float,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(webview) = webview.as_mut() else {
            return ERR_NULL_WEBVIEW;
        };
        let point: DevicePoint = DevicePoint::new(x, y);
        webview
            .webview
            .notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(point.into())));
        0
    }))
    .unwrap_or(ERR_PANIC)
}

/// 滑鼠按鍵代碼，與 `servo_webview_mouse_button` 搭配使用。
///
/// - `0` = 左鍵（Primary）
/// - `1` = 右鍵（Secondary）
/// - `2` = 中鍵（Auxiliary）
/// - 其他值視為 `Other(value)`
#[repr(C)]
pub enum MouseButtonCode {
    Primary = 0,
    Secondary = 1,
    Auxiliary = 2,
}

/// 通知 Servo 滑鼠按鍵事件。
///
/// - `button`：`0` = 左鍵、`1` = 右鍵、`2` = 中鍵。
/// - `pressed`：`1` = 按下、`0` = 放開。
/// - `x`, `y`：事件座標（裝置像素，相對於 WebView 左上角）。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_mouse_button(
    webview: *mut ServoWebView,
    button: c_uint,
    pressed: c_int,
    x: c_float,
    y: c_float,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(webview) = webview.as_mut() else {
            return ERR_NULL_WEBVIEW;
        };
        let servo_button = match button {
            0 => ServoMouseButton::Primary,
            1 => ServoMouseButton::Secondary,
            2 => ServoMouseButton::Auxiliary,
            other => ServoMouseButton::Other(other as u16),
        };
        let action = if pressed != 0 {
            MouseButtonAction::Down
        } else {
            MouseButtonAction::Up
        };
        let point: DevicePoint = DevicePoint::new(x, y);
        webview
            .webview
            .notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                action,
                servo_button,
                point.into(),
            )));
        0
    }))
    .unwrap_or(ERR_PANIC)
}

/// 通知 Servo 鍵盤事件。
///
/// `key_code` 為 Unicode 字碼點；若超出合法 Unicode 範圍或對應到特殊鍵
/// 則當作 `NamedKey::Unidentified` 處理。
///
/// `modifiers` 為位元旗標，使用 `SERVO_MOD_*` 常數組合：
///
/// ```c
/// #define SERVO_MOD_NONE    0x00
/// #define SERVO_MOD_SHIFT   0x01
/// #define SERVO_MOD_CTRL    0x02
/// #define SERVO_MOD_ALT     0x04
/// #define SERVO_MOD_META    0x08
/// ```
///
/// `pressed`：`1` = 按下（KeyDown）、`0` = 放開（KeyUp）。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_key(
    webview: *mut ServoWebView,
    key_code: u32,
    modifiers: u32,
    pressed: c_int,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(webview) = webview.as_mut() else {
            return ERR_NULL_WEBVIEW;
        };

        let key = char::from_u32(key_code)
            .map(|c| Key::Character(c.to_string().into()))
            .unwrap_or(Key::Named(NamedKey::Unidentified));

        let mut mods = Modifiers::empty();
        if modifiers & 0x01 != 0 {
            mods |= Modifiers::SHIFT;
        }
        if modifiers & 0x02 != 0 {
            mods |= Modifiers::CONTROL;
        }
        if modifiers & 0x04 != 0 {
            mods |= Modifiers::ALT;
        }
        if modifiers & 0x08 != 0 {
            mods |= Modifiers::META;
        }

        // For this Servo version, KeyboardEvent is from keyboard_types crate
        // It likely has a new() method or requires specific field initialization
        let state = if pressed != 0 {
            KeyState::Down
        } else {
            KeyState::Up
        };

        // Servo's InputEvent::Keyboard wraps keyboard_types::KeyboardEvent
        // We need to construct the inner keyboard_types event first
        // For now, create a minimal event with just the key and state
        // since the keyboard_types API may vary
        let ktype_event = ::keyboard_types::KeyboardEvent {
            key,
            code: ::keyboard_types::Code::Unidentified,
            location: ::keyboard_types::Location::Standard,
            state,
            modifiers: mods,
            repeat: false,
            is_composing: false,
        };

        let keyboard_event = InputEvent::Keyboard(KeyboardEvent::new(ktype_event));

        webview
            .webview
            .notify_input_event(keyboard_event);
        0
    }))
    .unwrap_or(ERR_PANIC)
}

/// 通知 Servo 滾輪事件。
///
/// `delta_x` 與 `delta_y` 為捲動量（裝置像素；向下/向右為正值）。
/// `x`, `y` 為滑鼠游標位置（裝置像素，相對於 WebView 左上角）。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_scroll(
    webview: *mut ServoWebView,
    delta_x: c_float,
    delta_y: c_float,
    x: c_float,
    y: c_float,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(webview) = webview.as_mut() else {
            return ERR_NULL_WEBVIEW;
        };
        let point: DevicePoint = DevicePoint::new(x, y);
        let delta = WheelDelta {
            x: delta_x as f64,
            y: delta_y as f64,
            z: 0.0,
            mode: WheelMode::DeltaPixel,
        };
        webview
            .webview
            .notify_input_event(InputEvent::Wheel(WheelEvent::new(delta, point.into())));
        0
    }))
    .unwrap_or(ERR_PANIC)
}
