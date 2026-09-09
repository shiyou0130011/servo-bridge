# Servo embedding API research

This document records the embedding API available at the Servo revision pinned
by this project:

```text
c038a19febfd7ab687de71caf04cfe180462b70f
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

## Consequence for this project

The current `src/lib.rs` only stores a URL string and never calls
`ServoBuilder`, creates a rendering context, creates a `WebView`, or pumps the
Servo event loop. Its existing DLL is consequently not yet a Servo renderer.

The next implementation step should be a Rust-only headless prototype based on
the pinned servoshell `HeadlessWindow` and `SoftwareRenderingContext`. Once it
can load a small HTML document and expose a stable framebuffer, the C ABI can
be added for Go/Ebiten.
