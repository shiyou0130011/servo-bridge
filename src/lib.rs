use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar};
use std::rc::Rc;

use dpi::PhysicalSize;
use euclid::{Point2D, Size2D};
use servo::{
    DeviceIntRect, DevicePixel, Servo, ServoBuilder, SoftwareRenderingContext, WebView,
    WebViewBuilder,
};
use url::Url;

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

#[no_mangle]
pub extern "C" fn servo_webview_new(width: u32, height: u32) -> *mut ServoWebView {
    if width == 0 || height == 0 {
        return std::ptr::null_mut();
    }

    let rendering_context = match SoftwareRenderingContext::new(PhysicalSize::new(width, height))
    {
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
        pixels: vec![0; (width as usize) * (height as usize) * 4],
        last_error: None,
    }))
}

#[no_mangle]
pub unsafe extern "C" fn servo_webview_load_url(
    webview: *mut ServoWebView,
    url: *const c_char,
) -> i32 {
    let Some(webview) = webview.as_mut() else {
        return -1;
    };
    if url.is_null() {
        set_error(webview, "URL 指標不可為空值");
        return -2;
    }

    let url = match CStr::from_ptr(url).to_str() {
        Ok(url) => url,
        Err(_) => {
            set_error(webview, "URL 必須是合法 UTF-8");
            return -3;
        },
    };
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(_) => {
            set_error(webview, "URL 格式無效");
            return -4;
        },
    };

    webview.webview.load(url);
    0
}

#[no_mangle]
pub unsafe extern "C" fn servo_webview_update(webview: *mut ServoWebView) -> i32 {
    let Some(webview) = webview.as_mut() else {
        return -1;
    };
    webview.servo.spin_event_loop();
    0
}

#[no_mangle]
pub unsafe extern "C" fn servo_webview_render(
    webview: *mut ServoWebView,
    output: *mut c_uchar,
    output_len: usize,
) -> i32 {
    let Some(webview) = webview.as_mut() else {
        return -1;
    };
    if output.is_null() || output_len < webview.pixels.len() {
        set_error(webview, "輸出緩衝區太小");
        return -2;
    }

    webview.webview.paint();
    let rectangle = DeviceIntRect::from_origin_and_size(
        Point2D::zero(),
        Size2D::new(webview.width as i32, webview.height as i32),
    );
    let Some(image) = webview.rendering_context.read_to_image(rectangle) else {
        set_error(webview, "目前尚未有可讀取的渲染畫面");
        return -3;
    };
    webview.pixels.copy_from_slice(image.as_raw());
    std::ptr::copy_nonoverlapping(webview.pixels.as_ptr(), output, webview.pixels.len());
    0
}

#[no_mangle]
pub extern "C" fn servo_webview_width(webview: *const ServoWebView) -> u32 {
    unsafe { webview.as_ref().map_or(0, |webview| webview.width) }
}

#[no_mangle]
pub extern "C" fn servo_webview_height(webview: *const ServoWebView) -> u32 {
    unsafe { webview.as_ref().map_or(0, |webview| webview.height) }
}

#[no_mangle]
pub extern "C" fn servo_webview_last_error(webview: *const ServoWebView) -> *const c_char {
    unsafe {
        webview
            .as_ref()
            .and_then(|webview| webview.last_error.as_ref())
            .map_or(std::ptr::null(), |error| error.as_ptr())
    }
}

#[no_mangle]
pub unsafe extern "C" fn servo_webview_free(webview: *mut ServoWebView) {
    if !webview.is_null() {
        drop(Box::from_raw(webview));
    }
}
