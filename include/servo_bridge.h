#ifndef SERVO_BRIDGE_H
#define SERVO_BRIDGE_H

#include <stdint.h>
#include <stddef.h>

#ifdef _WIN32
#define SERVO_BRIDGE_API __declspec(dllimport)
#else
#define SERVO_BRIDGE_API
#endif

typedef struct ServoWebView ServoWebView;

SERVO_BRIDGE_API ServoWebView *servo_webview_new(uint32_t width, uint32_t height);
SERVO_BRIDGE_API int32_t servo_webview_load_url(ServoWebView *webview, const char *url);
SERVO_BRIDGE_API int32_t servo_webview_update(ServoWebView *webview);
SERVO_BRIDGE_API int32_t servo_webview_render(
    ServoWebView *webview,
    uint8_t *output,
    size_t output_len
);
SERVO_BRIDGE_API uint32_t servo_webview_width(const ServoWebView *webview);
SERVO_BRIDGE_API uint32_t servo_webview_height(const ServoWebView *webview);
SERVO_BRIDGE_API const char *servo_webview_last_error(const ServoWebView *webview);
SERVO_BRIDGE_API void servo_webview_free(ServoWebView *webview);

#endif
