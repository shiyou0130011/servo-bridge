use std::ffi::CStr;
use std::os::raw::c_char;

// Windows runners need Git long-path support to fetch Servo's WPT test files.
#[cfg(not(target_os = "windows"))]
use servo as _;

// ==========================================
// 1. 定義對外公開的結構體指標 (Opaque Pointer)
// ==========================================
// 這是標準的 FFI 作法：不讓外部直接看到內部結構，只透過一個指標操作。
pub struct ServoWebView {
    // 這裡未來可以放你的 Servo WebView 實體或相關控制代碼
    url: String,
}

// ==========================================
// 2. 實作 C-ABI 導出函式 (外部語言可直接呼叫)
// ==========================================

/// 建立一個新的 WebView 實例
#[no_mangle]
pub extern "C" fn servo_webview_new() *mut ServoWebView {
    let webview = ServoWebView {
        url: String::from("about:blank"),
    };
    // 將 Box 轉換為裸指標 (Raw Pointer) 傳遞給外部語言
    Box::into_raw(Box::new(webview))
}

/// 讓 WebView 載入指定的網址
/// # Safety
/// 外部傳入的 `webview` 指標必須有效，且 `url` 必須是合法的 Null-terminated C 字串。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_load_url(
    webview: *mut ServoWebView,
    url: *const c_char,
) {
    if webview.is_null() || url.is_null() {
        return;
    }

    // 將 C 字串轉換為 Rust 的 &str
    let c_str = CStr::from_ptr(url);
    if let Ok(rust_str) = c_str.to_str() {
        let wv = &mut *webview;
        wv.url = rust_str.to_string();
        
        // TODO: 這裡未來替換為真正的 Servo 載入網址邏輯
        println!("Servo Bridge: Loading URL -> {}", wv.url);
    }
}

/// 釋放 WebView 資源，避免記憶體洩漏
/// # Safety
/// 外部呼叫時必須確保這個指標只被釋放一次。
#[no_mangle]
pub unsafe extern "C" fn servo_webview_free(webview: *mut ServoWebView) {
    if !webview.is_null() {
        // 將裸指標轉回 Box，當函式結束離開作用域時會自動釋放記憶體
        let _ = Box::from_raw(webview);
    }
}