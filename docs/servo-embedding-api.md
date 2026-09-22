# Servo embedding API research

This document records the embedding API available at the Servo revision pinned
by this project:

```text
v0.5.0
```

## Entry point

The public Servo crate exports `Servo` and `ServoBuilder`. The builder accepts:

- `Opts`
- `Preferences`
- `ProtocolRegistry`
- an `EventLoopWaker`

Calling `ServoBuilder::build()` creates the internal paint, constellation,
network, storage, JavaScript, and embedder channels. The engine is advanced by
calling `Servo::spin_event_loop()`.

The builder does **not** create a platform window or a web view by itself. The
embedding application must create a rendering context and a platform-window
implementation, then create/open a `WebView` through the higher-level
servoshell flow.

## Reference servoshell flow

The desktop servoshell follows this sequence:

1. Build a `Servo` with `ServoBuilder`.
2. Create a `PlatformWindow`.
3. Obtain its `RenderingContext`.
4. Create/open a `WebView` using the platform-window and URL.
5. Pump `Servo::spin_event_loop()` from the host event loop.
6. Forward window, mouse, keyboard, resize, and repaint events to the
   `WebView`.

For headless mode, the pinned Servo source includes
`ports/servoshell/desktop/headless_window.rs`. It creates a
`SoftwareRenderingContext` and implements `PlatformWindow` without a native
window. This is the closest existing reference for an Ebiten integration.

## Compositor and framebuffer implications

Servo's `Paint` subsystem owns the rendered output and communicates through
`RenderingContext`. The public API exposes rendering-context abstractions such
as `SoftwareRenderingContext` and `OffscreenRenderingContext`, but the
`ServoBuilder` API does not expose a simple `render(url) -> RGBA bytes`
function.

Therefore the DLL must provide its own embedding layer. It needs to:

- create and retain a suitable software or off-screen rendering context;
- implement the required platform-window behavior;
- create a `WebView` against that context;
- pump `spin_event_loop()` regularly;
- observe repaint/output updates;
- copy or read back the rendered pixels into a stable RGBA/BGRA buffer;
- expose width, height, stride, pixel format, and buffer lifetime through C ABI.

The `SoftwareRenderingContext` path should be investigated first because it
matches the desired CPU framebuffer handoff to Ebiten. GPU/off-screen contexts
would require a platform-specific texture sharing or readback design.

## Embedder callbacks

Servo uses embedder-side traits and messages for platform behavior. The
embedding implementation must handle at least:

- event-loop wakeups;
- repaint requests;
- window resize requests;
- console messages;
- navigation/load status callbacks;
- dialogs, permissions, file pickers, and context menus (or explicit
  headless defaults);
- input events;
- clipboard and IME behavior if interactive editing is required.

`EventLoopWaker` is required by `ServoBuilder`. A DLL can implement it with a
thread-safe wake signal consumed by the host, or use a dedicated Servo thread
and a command channel.

## Implementation status

`src/lib.rs` now provides a complete C ABI embedding layer. The following have
all been implemented against Servo v0.5.0:

| Component | API used | Status |
|-----------|----------|--------|
| Rendering context | `SoftwareRenderingContext::new(PhysicalSize)` | ✅ implemented |
| Engine initialisation | `ServoBuilder::default().build()` | ✅ implemented |
| WebView creation | `WebViewBuilder::new(&servo, ctx).url(url).build()` | ✅ implemented |
| Event loop | `servo.spin_event_loop()` | ✅ implemented |
| Framebuffer readback | `webview.paint()` + `rendering_context.read_to_image(rect)` | ✅ implemented |
| Resize | `webview.resize(PhysicalSize)` | ✅ implemented |
| Navigation | `webview.load(url)` | ✅ implemented |
| Mouse move | `InputEvent::MouseMove(MouseMoveEvent::new(point))` | ✅ implemented |
| Mouse button | `InputEvent::MouseButton(MouseButtonEvent::new(action, button, point))` | ✅ implemented |
| Keyboard | `InputEvent::Keyboard(KeyboardEvent::new(keyboard_types::KeyboardEvent { … }))` | ✅ implemented |
| Scroll / wheel | `InputEvent::Wheel(WheelEvent::new(WheelDelta { … }, point))` | ✅ implemented |

### Key implementation notes

**`SoftwareRenderingContext`** is wrapped in `Rc` and shared between `Servo`
and `WebView`. After calling `webview.paint()`, the rendered pixels are read
back via `rendering_context.read_to_image(DeviceIntRect)` which returns an
`Option<image::RgbaImage>`. A `None` result means no frame has been composited
yet (e.g. the page is still loading).

**Keyboard events** require two layers. The inner event is constructed as a
`keyboard_types::KeyboardEvent` struct with direct field initialisation:

```rust
let ktype_event = keyboard_types::KeyboardEvent {
    key,
    code: keyboard_types::Code::Unidentified,
    location: keyboard_types::Location::Standard,
    state,          // KeyState::Down / KeyState::Up
    modifiers: mods,
    repeat: false,
    is_composing: false,
};
let event = InputEvent::Keyboard(KeyboardEvent::new(ktype_event));
```

**`WheelDelta`** is a plain struct with public fields (`x`, `y`, `z: f64`,
`mode: WheelMode`); it is initialised directly without a constructor.

**`DevicePoint`** (`euclid`-based) must be converted with `.into()` when passed
to `MouseMoveEvent::new`, `MouseButtonEvent::new`, and `WheelEvent::new`.

### C ABI surface

The current ABI version is **2**. All functions are exported with `#[no_mangle]`
and `extern "C"`. Panics are caught with `catch_unwind` so that a Rust panic
never unwinds into C code; the caught panic returns `SERVO_ERR_PANIC` (`-99`).

Error messages are stored as `CString` inside `ServoWebView` and are accessible
via `servo_webview_last_error()`. The pointer remains valid as long as the
`ServoWebView` is alive and no subsequent call has overwritten the error.

### Known limitations

- **Not thread-safe.** All calls must come from the same thread. No
  synchronisation primitives protect the internal `ServoWebView` state.
- **No IME support.** Keyboard input is limited to Unicode code points and the
  four standard modifier keys (Shift, Ctrl, Alt, Meta).
- **`about:blank` initial page.** `servo_webview_new` loads `about:blank`
  before the caller's URL, so the first `servo_webview_render` call may return
  `SERVO_ERR_RENDER` (`-5`) until the first real page frame is composited.
- **No GPU path.** Only `SoftwareRenderingContext` is used; GPU-accelerated
  rendering is not currently available through the C ABI.
