# servo-bridge 
![版本](https://img.shields.io/badge/version-0.0.3--2-blue)
![授權](https://img.shields.io/badge/License-MIT-red.svg?labelColor=white&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGhlaWdodD0iMzIxIiB3aWR0aD0iMzIxIj4KPGcgc3Ryb2tlLXdpZHRoPSIzNSIgc3Ryb2tlPSIjQTMxRjM0IiB0cmFuc2Zvcm09InRyYW5zbGF0ZSgwLCA3NykiPgo8cGF0aCBkPSJtMTcuNSwwdjE2Nm01Ny0xNjZ2MTEzbTU3LTExM3YxNjZtNTctMTY2djMzbTU4LDIwdjExMyIvPgo8cGF0aCBkPSJtMTg4LjUsNTN2MTEzIiBzdHJva2U9IiM4QThCOEMiLz4KPHBhdGggZD0ibTIyOSwxNi41aDkyIiBzdHJva2Utd2lkdGg9IjMzIi8+CjwvZz4KPC9zdmc+Cg==)

本專案是一個以 Rust 撰寫的 C ABI 橋接函式庫，將固定版本的
[Servo](https://github.com/servo/servo) 引擎納入建置，並輸出可由其他語言載入的
動態函式庫，讓 C/C++、Go、C#、Python 等程式可以透過 FFI 呼叫 Servo 的
headless off-screen 渲染能力。

主要使用場景是讓 **Go + Ebiten** 應用程式在 Ebiten 的 draw loop 中嵌入 Servo
網頁渲染結果，並將滑鼠、鍵盤、捲動等輸入事件回傳給 Servo。

固定 Servo commit 的 embedding API 調查結果請參閱
[`docs/servo-embedding-api.md`](docs/servo-embedding-api.md)。

## 架構

```text
Go / Ebiten game loop
       │
       │  FFI (cgo 或 syscall)
       ▼
  Rust DLL (servo-bridge)
       │
       ├── Servo engine (ServoBuilder)
       ├── SoftwareRenderingContext  ← off-screen CPU framebuffer
       ├── WebView + event loop
       └── RGBA8 pixel buffer
               │
               ▼
       Go []byte → ebiten.Image
```

## 編譯

官方 Servo 目前要求 Rust `1.97.1`。在已安裝 Rust 與 Cargo 的環境執行：

```bash
cargo build --release --lib
```

編譯產物位於 `target/release`：

| 平台    | 動態函式庫              |
| ------- | ----------------------- |
| Windows | `servo_bridge.dll`      |
| Linux   | `libservo_bridge.so`    |
| macOS   | `libservo_bridge.dylib` |

Windows 的 Git 需要啟用長路徑支援，GitHub Actions 已在建置前自動設定：

```powershell
git config --global core.longpaths true
```

## C ABI

### ABI 版本

```c
uint32_t servo_bridge_version(void);
```

在載入 DLL 後第一件事應呼叫此函式確認相容性，目前版本為 `2`。

### 完整函式一覽

```c
typedef struct ServoWebView ServoWebView;

/* 建立與釋放 */
ServoWebView *servo_webview_new(uint32_t width, uint32_t height);
void          servo_webview_free(ServoWebView *webview);

/* 屬性 */
uint32_t    servo_webview_width(const ServoWebView *webview);
uint32_t    servo_webview_height(const ServoWebView *webview);
const char *servo_webview_last_error(const ServoWebView *webview);

/* 導覽 */
int32_t servo_webview_load_url(ServoWebView *webview, const char *url);

/* 更新與渲染 */
int32_t servo_webview_update(ServoWebView *webview);
int32_t servo_webview_render(ServoWebView *webview,
                             uint8_t *output, size_t output_len);

/* Resize */
int32_t servo_webview_resize(ServoWebView *webview,
                             uint32_t width, uint32_t height);

/* 輸入事件 */
int32_t servo_webview_mouse_move(ServoWebView *webview,
                                 float x, float y);
int32_t servo_webview_mouse_button(ServoWebView *webview,
                                   uint32_t button, int32_t pressed,
                                   float x, float y);
int32_t servo_webview_key(ServoWebView *webview,
                          uint32_t key_code, uint32_t modifiers,
                          int32_t pressed);
int32_t servo_webview_scroll(ServoWebView *webview,
                             float delta_x, float delta_y,
                             float x, float y);
```

### 滑鼠按鍵代碼

| `button` 值 | 按鍵    |
| ----------- | ------- |
| `0`         | 左鍵    |
| `1`         | 右鍵    |
| `2`         | 中鍵    |

### 修飾鍵旗標

```c
#define SERVO_MOD_NONE  0x00
#define SERVO_MOD_SHIFT 0x01
#define SERVO_MOD_CTRL  0x02
#define SERVO_MOD_ALT   0x04
#define SERVO_MOD_META  0x08
```

可以用位元 OR 組合多個修飾鍵，例如 Ctrl+Shift = `SERVO_MOD_CTRL | SERVO_MOD_SHIFT`。

### 錯誤碼

```c
#define SERVO_OK                  0
#define SERVO_ERR_NULL_WEBVIEW   -1
#define SERVO_ERR_INVALID_ARG    -2
#define SERVO_ERR_INVALID_URL    -3
#define SERVO_ERR_INVALID_UTF8   -4
#define SERVO_ERR_RENDER         -5
#define SERVO_ERR_PANIC          -99
```

所有回傳 `int32_t` 的函式：成功回傳 `0`，失敗回傳負值。可透過
`servo_webview_last_error` 取得目前錯誤訊息。

### 一般使用注意事項

- `ServoWebView` 是不透明型別，外部程式不可直接存取其欄位。
- `url` 必須是合法的 UTF-8、以 NUL 結尾的 C 字串。
- 由 `servo_webview_new` 建立的指標必須交由 `servo_webview_free` 釋放。
- `servo_webview_render` 輸出 RGBA8 緩衝區（每像素 4 bytes），大小至少需為
  `width * height * 4`。
- 目前介面未承諾執行緒安全，所有函式應在同一個執行緒呼叫。

## Go 使用方式

### 搭配 Ebiten 的 game loop

以下是一個使用 cgo 搭配 Ebiten 的最小範例。假設目錄結構如下：

```text
go-app/
├── main.go
└── native/
    ├── servo_bridge.h
    └── servo_bridge.dll   # Windows
```

```go
package main

/*
#cgo CFLAGS: -I./native
#cgo windows LDFLAGS: -L./native -lservo_bridge
#cgo linux LDFLAGS: -L./native -lservo_bridge
#cgo darwin LDFLAGS: -L./native -lservo_bridge
#include "servo_bridge.h"
#include <stdlib.h>
*/
import "C"

import (
    "image"
    "unsafe"

    "github.com/hajimehoshi/ebiten/v2"
    "github.com/hajimehoshi/ebiten/v2/inpututil"
)

const (
    screenWidth  = 1280
    screenHeight = 720
)

type Game struct {
    view    *C.ServoWebView
    img     *ebiten.Image
    buf     []byte
    width   int
    height  int
}

func NewGame() *Game {
    // 確認 ABI 版本
    if C.servo_bridge_version() < 2 {
        panic("servo_bridge.dll ABI 版本過舊")
    }

    view := C.servo_webview_new(screenWidth, screenHeight)
    if view == nil {
        panic("servo_webview_new failed")
    }

    url := C.CString("https://example.com")
    defer C.free(unsafe.Pointer(url))
    C.servo_webview_load_url(view, url)

    return &Game{
        view: view,
        img:  ebiten.NewImage(screenWidth, screenHeight),
        buf:  make([]byte, screenWidth*screenHeight*4),
        width: screenWidth,
        height: screenHeight,
    }
}

func (g *Game) Update() error {
    // 驅動 Servo 事件迴圈
    C.servo_webview_update(g.view)

    // 傳遞滑鼠位置與按鍵事件
    cx, cy := ebiten.CursorPosition()
    C.servo_webview_mouse_move(g.view, C.float(cx), C.float(cy))
    for _, button := range []struct {
        ebitenButton ebiten.MouseButton
        servoButton  C.uint
    }{
        {ebiten.MouseButtonLeft, 0},
        {ebiten.MouseButtonRight, 1},
        {ebiten.MouseButtonMiddle, 2},
    } {
        if inpututil.IsMouseButtonJustPressed(button.ebitenButton) {
            C.servo_webview_mouse_button(g.view, button.servoButton, 1,
                C.float(cx), C.float(cy))
        }
        if inpututil.IsMouseButtonJustReleased(button.ebitenButton) {
            C.servo_webview_mouse_button(g.view, button.servoButton, 0,
                C.float(cx), C.float(cy))
        }
    }

    // 傳遞常用文字鍵；應用程式可依需求擴充這個映射表。
    for _, key := range []struct {
        ebitenKey ebiten.Key
        code      C.uint
    }{
        {ebiten.KeyA, 'A'}, {ebiten.KeyB, 'B'}, {ebiten.KeyC, 'C'},
        {ebiten.KeyD, 'D'}, {ebiten.KeyE, 'E'}, {ebiten.KeyF, 'F'},
        {ebiten.KeyG, 'G'}, {ebiten.KeyH, 'H'}, {ebiten.KeyI, 'I'},
        {ebiten.KeyJ, 'J'}, {ebiten.KeyK, 'K'}, {ebiten.KeyL, 'L'},
        {ebiten.KeyM, 'M'}, {ebiten.KeyN, 'N'}, {ebiten.KeyO, 'O'},
        {ebiten.KeyP, 'P'}, {ebiten.KeyQ, 'Q'}, {ebiten.KeyR, 'R'},
        {ebiten.KeyS, 'S'}, {ebiten.KeyT, 'T'}, {ebiten.KeyU, 'U'},
        {ebiten.KeyV, 'V'}, {ebiten.KeyW, 'W'}, {ebiten.KeyX, 'X'},
        {ebiten.KeyY, 'Y'}, {ebiten.KeyZ, 'Z'},
    } {
        if inpututil.IsKeyJustPressed(key.ebitenKey) {
            C.servo_webview_key(g.view, key.code, 0, 1)
        }
        if inpututil.IsKeyJustReleased(key.ebitenKey) {
            C.servo_webview_key(g.view, key.code, 0, 0)
        }
    }

    // 傳遞滾輪
    sx, sy := ebiten.Wheel()
    if sx != 0 || sy != 0 {
        C.servo_webview_scroll(g.view,
            C.float(sx*16), C.float(-sy*16),
            C.float(cx), C.float(cy))
    }

    // 讓 Ebiten 的視窗大小變更同步到 Servo。
    width, height := ebiten.WindowSize()
    if width > 0 && height > 0 && (width != g.width || height != g.height) {
        if C.servo_webview_resize(g.view, C.uint(width), C.uint(height)) == 0 {
            g.width, g.height = width, height
            g.buf = make([]byte, width*height*4)
            g.img = ebiten.NewImage(width, height)
        }
    }

    return nil
}

func (g *Game) Draw(screen *ebiten.Image) {
    // 把 Servo 渲染結果讀進 buf
    ptr := (*C.uint8_t)(unsafe.Pointer(&g.buf[0]))
    ret := C.servo_webview_render(g.view, ptr, C.size_t(len(g.buf)))
    if ret != 0 {
        return
    }

    // 更新 Ebiten image（使用 image.NRGBA 避免每幀配置）
    nrgba := &image.NRGBA{Pix: g.buf, Stride: g.width * 4,
        Rect: image.Rect(0, 0, g.width, g.height)}
    g.img.WritePixels(nrgba.Pix)
    screen.DrawImage(g.img, nil)
}

func (g *Game) Layout(ow, oh int) (int, int) {
    return g.width, g.height
}

func main() {
    game := NewGame()
    defer C.servo_webview_free(game.view)

    ebiten.SetWindowSize(screenWidth, screenHeight)
    ebiten.SetWindowTitle("Servo + Ebiten")
    if err := ebiten.RunGame(game); err != nil {
        panic(err)
    }
}
```

這個範例涵蓋初始化、ABI 版本檢查、URL 載入、每幀更新與渲染、滑鼠移動/
按鍵、滾輪及視窗 resize。實際產品仍應補上完整的 Ebiten `Key`、修飾鍵與
輸入法映射，並檢查每個 FFI 回傳值。

### Windows：使用 syscall（不需要 cgo）

Windows 也可以不使用 cgo，直接以 `syscall.NewLazyDLL` 載入 DLL：

```go
package main

import (
    "fmt"
    "syscall"
    "unsafe"
)

var (
    servoDLL        = syscall.NewLazyDLL("servo_bridge.dll")
    versionProc     = servoDLL.NewProc("servo_bridge_version")
    newProc         = servoDLL.NewProc("servo_webview_new")
    loadProc        = servoDLL.NewProc("servo_webview_load_url")
    updateProc      = servoDLL.NewProc("servo_webview_update")
    renderProc      = servoDLL.NewProc("servo_webview_render")
    resizeProc      = servoDLL.NewProc("servo_webview_resize")
    mouseMoveProc   = servoDLL.NewProc("servo_webview_mouse_move")
    mouseButtonProc = servoDLL.NewProc("servo_webview_mouse_button")
    keyProc         = servoDLL.NewProc("servo_webview_key")
    scrollProc      = servoDLL.NewProc("servo_webview_scroll")
    freeProc        = servoDLL.NewProc("servo_webview_free")
)

func main() {
    ver, _, _ := versionProc.Call()
    fmt.Printf("servo_bridge ABI version: %d\n", ver)

    ret, _, _ := newProc.Call(1280, 720)
    if ret == 0 {
        panic("servo_webview_new failed")
    }
    view := unsafe.Pointer(ret)
    defer freeProc.Call(uintptr(view))

    urlPtr, _ := syscall.BytePtrFromString("https://example.com")
    loadProc.Call(uintptr(view), uintptr(unsafe.Pointer(urlPtr)))

    // game loop ...
    updateProc.Call(uintptr(view))
    buf := make([]byte, 1280*720*4)
    renderProc.Call(uintptr(view),
        uintptr(unsafe.Pointer(&buf[0])),
        uintptr(len(buf)))
}
```

執行時 `servo_bridge.dll` 必須能被 Windows DLL 搜尋路徑找到（最簡單是放在
`.exe` 同目錄）。

### Linux

將產物放在 `native/` 後，把 cgo 標頭與連結設定改為：

```go
/*
#cgo linux CFLAGS:  -I./native
#cgo linux LDFLAGS: -L./native -lservo_bridge
*/
```

執行程式前設定共享函式庫搜尋路徑：

```bash
LD_LIBRARY_PATH="$PWD/native:$LD_LIBRARY_PATH" go run .
```

### macOS

```go
/*
#cgo darwin CFLAGS:  -I./native
#cgo darwin LDFLAGS: -L./native -lservo_bridge
*/
```

執行前可設定：

```bash
DYLD_LIBRARY_PATH="$PWD/native:$DYLD_LIBRARY_PATH" go run .
```

## 使用注意事項

- Go 的 `C.CString` 會配置 C 記憶體，使用完後應以 `C.free` 釋放。
- 不要在不同執行緒同時操作同一個 `ServoWebView`；目前介面未承諾執行緒安全。
- DLL、SO 或 dylib 必須與 Go 程式使用相同的 CPU 架構（例如都使用 `amd64`）。
- Ebiten 的 `WritePixels`（或舊版 `ReplacePixels`）接受 RGBA 格式的 `[]byte`，
  與 `servo_webview_render` 輸出格式相符，可直接傳入。
- 避免在每幀建立新的 `ebiten.Image`；建議在 `NewGame` 時建立一次後重用。
- 本專案使用 MIT 授權；Servo 及其相依元件的授權亦須一併遵守。
