# servo-lib

這個專案將官方 Servo 引擎連結進 Rust `cdylib`，產生可由 C/C++、C#、
Python 等外部程式載入的動態函式庫。

## 本機編譯

官方 Servo 目前要求 Rust `1.97.1`。在已安裝 Rust 的環境執行：

```text
cargo build --release --lib
```

產物位置：

- Windows：`target/release/servo_bridge.dll`
- Linux：`target/release/libservo_bridge.so`
- macOS：`target/release/libservo_bridge.dylib`

## C ABI

目前公開的函式為 `servo_webview_new`、`servo_webview_load_url` 與
`servo_webview_free`。外部程式必須在同一個函式庫邊界釋放由 Rust 配置的
`ServoWebView` 指標，不可使用外部語言的記憶體釋放器取代
`servo_webview_free`。