# servo-lib

`servo-lib` 是一個以 Rust 撰寫的 C ABI 橋接函式庫。專案將固定版本的
[Servo](https://github.com/servo/servo) 引擎納入建置，並輸出可由其他語言載入的
動態函式庫，讓 C/C++、Go、C#、Python 等程式可以透過 FFI 呼叫 Rust 函式。

目前提供的是 WebView 橋接介面骨架：可以建立與釋放不透明的
`ServoWebView` 指標，並更新其 URL。`servo_webview_load_url` 目前會記錄載入的
URL；實際的 Servo 視窗、渲染與事件迴圈尚未接入。

固定 Servo commit 的 embedding API 調查結果請參閱
[`docs/servo-embedding-api.md`](docs/servo-embedding-api.md)。下一步應先以
`SoftwareRenderingContext` 完成 Rust 端 headless framebuffer prototype，再
設計供 Go/Ebiten 使用的 C ABI。

## 編譯

官方 Servo 目前要求 Rust `1.97.1`。在已安裝 Rust 與 Cargo 的環境執行：

```bash
cargo build --release --lib
```

編譯產物位於 `target/release`：

| 平台 | 動態函式庫 |
| --- | --- |
| Windows | `servo_bridge.dll` |
| Linux | `libservo_bridge.so` |
| macOS | `libservo_bridge.dylib` |

Windows 的 Git 需要啟用長路徑支援，GitHub Actions 已在建置前自動設定：

```powershell
git config --global core.longpaths true
```

## C ABI

公開函式如下：

```c
typedef struct ServoWebView ServoWebView;

ServoWebView *servo_webview_new(void);
void servo_webview_load_url(ServoWebView *webview, const char *url);
void servo_webview_free(ServoWebView *webview);
```

`ServoWebView` 是不透明型別，外部程式不可直接存取其欄位。`url` 必須是
合法的 UTF-8、以 NUL 結尾的 C 字串。由 `servo_webview_new` 建立的指標必須
交由 `servo_webview_free` 釋放，不能使用 Go、C 或其他執行環境的記憶體釋放器。

## Go 使用方式

Go 可以透過 `cgo` 呼叫 C ABI；在 Windows 上也可以不使用 cgo，直接以
`syscall` 動態載入 DLL。以下範例假設 Go 程式的目錄結構如下：

```text
go-app/
├── main.go
└── native/
    └── servo_bridge.dll       # Windows
```

### Windows：使用 syscall（不需要 cgo）

Windows 可使用 `syscall.NewLazyDLL` 載入 DLL，再以 `NewProc` 取得公開函式。
這種方式不需要 C 編譯器，也不需要在 Go 專案中啟用 cgo：

```go
package main

import (
	"fmt"
	"syscall"
	"unsafe"
)

var (
	servoDLL = syscall.NewLazyDLL("servo_bridge.dll")
	newProc  = servoDLL.NewProc("servo_webview_new")
	loadProc = servoDLL.NewProc("servo_webview_load_url")
	freeProc = servoDLL.NewProc("servo_webview_free")
)

func newWebView() unsafe.Pointer {
	ret, _, callErr := newProc.Call()
	if ret == 0 {
		panic(fmt.Sprintf("servo_webview_new failed: %v", callErr))
	}
	return unsafe.Pointer(ret)
}

func loadURL(view unsafe.Pointer, url string) {
	urlPtr, err := syscall.BytePtrFromString(url)
	if err != nil {
		panic(err)
	}
	loadProc.Call(
		uintptr(view),
		uintptr(unsafe.Pointer(urlPtr)),
	)
}

func freeWebView(view unsafe.Pointer) {
	if view != nil {
		freeProc.Call(uintptr(view))
	}
}

func main() {
	view := newWebView()
	defer freeWebView(view)
	loadURL(view, "https://example.com")
}
```

執行時 `servo_bridge.dll` 必須能被 Windows DLL 搜尋路徑找到；最簡單的方式是
將它放在產生的 `.exe` 同一個目錄，或加入 `PATH`。Go 程式與 DLL 必須使用相同
的 CPU 架構，例如都使用 `amd64`。`syscall` 的 `Call` 參數使用 `uintptr`，
而 `ServoWebView` 指標只應透過 DLL 提供的函式操作與釋放。

### Windows：使用 cgo

如果專案本身已經使用 cgo，也可以透過編譯期連結方式呼叫 DLL：

```go
package main

/*
#cgo windows LDFLAGS: -L./native -lservo_bridge
#include <stdlib.h>

typedef struct ServoWebView ServoWebView;
ServoWebView *servo_webview_new(void);
void servo_webview_load_url(ServoWebView *webview, const char *url);
void servo_webview_free(ServoWebView *webview);
*/
import "C"

import (
	"unsafe"
)

func main() {
	view := C.servo_webview_new()
	if view == nil {
		panic("failed to create ServoWebView")
	}
	defer C.servo_webview_free(view)

	url := C.CString("https://example.com")
	defer C.free(unsafe.Pointer(url))
	C.servo_webview_load_url(view, url)
}
```

執行時 `servo_bridge.dll` 必須能被 Windows DLL 搜尋路徑找到。Windows 使用 cgo
需要安裝可用的 GCC 工具鏈，例如 MinGW-w64。

### Linux

將產物放在 `native/` 後，將 cgo 標頭與連結設定改為：

```go
/*
#cgo linux CFLAGS: -I./native
#cgo linux LDFLAGS: -L./native -lservo_bridge
*/
```

執行程式前設定共享函式庫搜尋路徑：

```bash
LD_LIBRARY_PATH="$PWD/native:$LD_LIBRARY_PATH" go run .
```

也可以將 `libservo_bridge.so` 安裝到系統共享函式庫目錄並執行
`sudo ldconfig`。Linux 使用 cgo 需要安裝 GCC、Rust/Servo 所需的系統開發套件。

### macOS

將產物放在 `native/`，並使用：

```go
/*
#cgo darwin CFLAGS: -I./native
#cgo darwin LDFLAGS: -L./native -lservo_bridge
*/
```

執行前可設定：

```bash
DYLD_LIBRARY_PATH="$PWD/native:$DYLD_LIBRARY_PATH" go run .
```

macOS 使用 cgo 需要安裝 Xcode Command Line Tools。

## 使用注意事項

- Go 的 `C.CString` 會配置 C 記憶體，使用完後應以 `C.free` 釋放。
- 不要在不同執行緒或不同函式庫邊界任意操作 `ServoWebView` 指標；目前介面未承諾
  執行緒安全。
- DLL、SO 或 dylib 必須與 Go 程式使用相同的 CPU 架構，例如都使用
  `x86_64` 或都使用 `arm64`。
- 本專案使用 MPL-2.0 授權；Servo 及其相依元件的授權亦須一併遵守。
