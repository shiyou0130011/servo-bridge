#ifndef SERVO_BRIDGE_H
#define SERVO_BRIDGE_H

#include <stdint.h>
#include <stddef.h>

#ifdef _WIN32
#define SERVO_BRIDGE_API __declspec(dllimport)
#else
#define SERVO_BRIDGE_API
#endif

/* -------------------------------------------------------------------------
 * ABI 版本
 * -------------------------------------------------------------------------
 * 在載入 DLL 後第一件事應呼叫此函式，確認版本相容性。
 * 目前版本為 2。
 */
SERVO_BRIDGE_API uint32_t servo_bridge_version(void);

/* -------------------------------------------------------------------------
 * 不透明型別
 * -------------------------------------------------------------------------
 * 不可直接存取其欄位；所有操作都必須透過下方的函式進行。
 * 由 servo_webview_new() 建立的指標必須交由 servo_webview_free() 釋放。
 */
typedef struct ServoWebView ServoWebView;

/* -------------------------------------------------------------------------
 * 建立與釋放
 * -------------------------------------------------------------------------
 * servo_webview_new()  : 建立新 WebView，width/height 必須大於 0。
 *                        失敗時回傳 NULL。
 * servo_webview_free() : 釋放 WebView，傳入 NULL 時無作用。
 */
SERVO_BRIDGE_API ServoWebView *servo_webview_new(uint32_t width, uint32_t height);
SERVO_BRIDGE_API void servo_webview_free(ServoWebView *webview);

/* -------------------------------------------------------------------------
 * 屬性
 * -------------------------------------------------------------------------
 * servo_webview_width()      : 回傳目前渲染寬度（像素），webview 為 NULL 時回傳 0。
 * servo_webview_height()     : 回傳目前渲染高度（像素），webview 為 NULL 時回傳 0。
 * servo_webview_last_error() : 最後一次錯誤訊息，無錯誤時回傳 NULL。
 *                              回傳的指標由 DLL 管理，呼叫端不可釋放。
 */
SERVO_BRIDGE_API uint32_t servo_webview_width(const ServoWebView *webview);
SERVO_BRIDGE_API uint32_t servo_webview_height(const ServoWebView *webview);
SERVO_BRIDGE_API const char *servo_webview_last_error(const ServoWebView *webview);

/* -------------------------------------------------------------------------
 * 導覽
 * -------------------------------------------------------------------------
 * url 必須是合法的 UTF-8、以 NUL 結尾的 C 字串。
 * 成功回傳 0；失敗回傳負值（可用 servo_webview_last_error 取得訊息）。
 */
SERVO_BRIDGE_API int32_t servo_webview_load_url(ServoWebView *webview, const char *url);

/* -------------------------------------------------------------------------
 * 更新與渲染
 * -------------------------------------------------------------------------
 * servo_webview_update() : 驅動 Servo 事件迴圈。Ebiten 的 Update() 每幀呼叫一次。
 *                          成功回傳 0；失敗回傳負值。
 *
 * servo_webview_render() : 將最新畫面讀入 output 緩衝區（RGBA8 格式，每像素 4 bytes）。
 *                          output_len 必須至少為 width * height * 4。
 *                          成功回傳 0；失敗回傳負值。
 */
SERVO_BRIDGE_API int32_t servo_webview_update(ServoWebView *webview);
SERVO_BRIDGE_API int32_t servo_webview_render(
    ServoWebView *webview,
    uint8_t *output,
    size_t output_len
);

/* -------------------------------------------------------------------------
 * Resize
 * -------------------------------------------------------------------------
 * 調整 WebView 的渲染尺寸。width 與 height 必須大於 0。
 * 呼叫後應再呼叫 update 與 render 才能取得新尺寸的畫面。
 * 成功回傳 0；失敗回傳負值。
 */
SERVO_BRIDGE_API int32_t servo_webview_resize(
    ServoWebView *webview,
    uint32_t width,
    uint32_t height
);

/* -------------------------------------------------------------------------
 * 輸入事件
 * -------------------------------------------------------------------------
 * 所有輸入事件函式成功回傳 0；失敗回傳負值。
 * x, y 座標均為相對於 WebView 左上角的裝置像素。
 */

/*
 * 滑鼠移動事件
 */
SERVO_BRIDGE_API int32_t servo_webview_mouse_move(
    ServoWebView *webview,
    float x,
    float y
);

/*
 * 滑鼠按鍵事件
 *   button : 0 = 左鍵, 1 = 右鍵, 2 = 中鍵
 *   pressed: 1 = 按下, 0 = 放開
 */
SERVO_BRIDGE_API int32_t servo_webview_mouse_button(
    ServoWebView *webview,
    uint32_t button,
    int32_t pressed,
    float x,
    float y
);

/*
 * 鍵盤事件
 *   key_code  : Unicode 字碼點（例如 'A' = 0x41）
 *   modifiers : 位元旗標組合
 *     #define SERVO_MOD_NONE    0x00
 *     #define SERVO_MOD_SHIFT   0x01
 *     #define SERVO_MOD_CTRL    0x02
 *     #define SERVO_MOD_ALT     0x04
 *     #define SERVO_MOD_META    0x08
 *   pressed   : 1 = 按下 (KeyDown), 0 = 放開 (KeyUp)
 */
#define SERVO_MOD_NONE  0x00
#define SERVO_MOD_SHIFT 0x01
#define SERVO_MOD_CTRL  0x02
#define SERVO_MOD_ALT   0x04
#define SERVO_MOD_META  0x08

SERVO_BRIDGE_API int32_t servo_webview_key(
    ServoWebView *webview,
    uint32_t key_code,
    uint32_t modifiers,
    int32_t pressed
);

/*
 * 滾輪事件
 *   delta_x, delta_y : 捲動量（裝置像素；向下/向右為正值）
 *   x, y             : 滑鼠游標位置
 */
SERVO_BRIDGE_API int32_t servo_webview_scroll(
    ServoWebView *webview,
    float delta_x,
    float delta_y,
    float x,
    float y
);

/* -------------------------------------------------------------------------
 * 錯誤碼
 * -------------------------------------------------------------------------
 * 所有函式在成功時回傳 0，失敗時回傳以下負值之一。
 */
#define SERVO_OK                  0
#define SERVO_ERR_NULL_WEBVIEW   -1
#define SERVO_ERR_INVALID_ARG    -2
#define SERVO_ERR_INVALID_URL    -3
#define SERVO_ERR_INVALID_UTF8   -4
#define SERVO_ERR_RENDER         -5
#define SERVO_ERR_PANIC          -99

#endif /* SERVO_BRIDGE_H */
