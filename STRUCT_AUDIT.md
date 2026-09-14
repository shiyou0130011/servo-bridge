# Servo API 結構體欄位審計

## 檢查清單

### 1. KeyboardEvent
**位置**: `src/lib.rs` 426-436
**狀態**: 已修正
**修正方式**:
1. 以 `keyboard_types::KeyboardEvent` 的欄位建立底層鍵盤事件。
2. 將底層事件傳入 Servo 匯出的 `KeyboardEvent::new(ktype_event)`。
3. 以 `InputEvent::Keyboard` 傳送至 WebView。
**結論**: ✅ 已依照 Servo c038a19 的 API 建構鍵盤事件。

### 2. MouseMoveEvent.new()
**位置**: `src/lib.rs` 308
**構造代碼**: `MouseMoveEvent::new(point.into())`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ⚠️ 原始碼已採用工廠方法，但尚未在目前環境執行 cargo check。

### 3. MouseButtonEvent.new()
**位置**: `src/lib.rs` 358-362
**構造代碼**: `MouseButtonEvent::new(action, servo_button, point.into())`
**構造方式**: 使用 `.new()` 工廠方法，三個參數
**參數**:
  - action: MouseButtonAction
  - servo_button: MouseButton
  - point.into(): Point
**狀態**: ⚠️ 原始碼已採用工廠方法，但尚未在目前環境執行 cargo check。

### 4. WheelEvent.new()
**位置**: `src/lib.rs` 471
**構造代碼**: `WheelEvent::new(delta, point.into())`
**構造方式**: 使用 `.new()` 工廠方法，兩個參數
**參數**:
  - delta: WheelDelta
  - point.into(): Point
**狀態**: ⚠️ 原始碼已採用工廠方法，但尚未在目前環境執行 cargo check。

## 其他潛在問題區域

### 5. SoftwareRenderingContext.new()
**位置**: `src/lib.rs` 72
**構造代碼**: `SoftwareRenderingContext::new(PhysicalSize::new(width, height))`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ⚠️ 尚未在目前環境執行 cargo check。

### 6. ServoBuilder.default().build()
**位置**: `src/lib.rs` 81
**構造代碼**: `ServoBuilder::default().build()`
**構造方式**: Builder pattern
**狀態**: ⚠️ 尚未在目前環境執行 cargo check。

### 7. WebViewBuilder.new()
**位置**: `src/lib.rs` 83-85
**構造代碼**: `WebViewBuilder::new(&servo, rendering_context.clone()).url(url).build()`
**構造方式**: Builder pattern with fluent API
**狀態**: ⚠️ 尚未在目前環境執行 cargo check。

### 8. PhysicalSize.new()
**位置**: Line 76, 333
**構造代碼**: `PhysicalSize::new(width, height)`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ✅ 來自 dpi crate。

### 9. DevicePoint.new()
**位置**: Line 305, 347, 459
**構造代碼**: `DevicePoint::new(x, y)`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ✅ 來自 euclid crate。

### 10. Point2D.zero()
**位置**: Line 201
**構造代碼**: `Point2D::zero()`
**構造方式**: 工廠方法
**狀態**: ✅ 來自 euclid crate。

### 11. Size2D.new()
**位置**: Line 202
**構造代碼**: `Size2D::new(webview.width as i32, webview.height as i32)`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ✅ 來自 euclid crate。

### 12. WheelDelta {}
**位置**: Line 457-461
**構造代碼**:
```rust
WheelDelta {
    x: delta_x as f64,
    y: delta_y as f64,
    z: 0.0,
    mode: WheelMode::DeltaPixel,
}
```
**構造方式**: 結構體欄位初始化
**狀態**: ⚠️ 目前尚未在本機執行 cargo check，待 CI 或具備 Rust 工具鏈的環境確認。

## 尚未完成的驗證

目前環境未安裝 Cargo，因此尚未能執行 `cargo check`。以下項目需要在具備 Rust
工具鏈的環境中驗證：

1. `MouseMoveEvent::new()` 的參數簽名。
2. `MouseButtonEvent::new()` 的參數數量與順序。
3. `WheelEvent::new()` 的參數順序。
4. `WheelDelta` 直接欄位初始化是否符合 Servo c038a19。
5. `SoftwareRenderingContext`、`ServoBuilder` 與 `WebViewBuilder` 的建構流程。

## 修正跟蹤

- [x] KeyboardEvent - 使用 `keyboard_types::KeyboardEvent` 欄位建立事件，再呼叫 Servo 的 `KeyboardEvent::new()`。
- [ ] MouseMoveEvent - 待 cargo check 驗證
- [ ] MouseButtonEvent - 待 cargo check 驗證
- [ ] WheelEvent - 待 cargo check 驗證
- [ ] WheelDelta - 待 cargo check 驗證

## 最終驗證狀態

鍵盤事件的直接依賴與建構方式已更新；其他 Servo API 呼叫仍需在具備 Rust
工具鏈的環境中執行 cargo check，才能完成編譯層級的最終確認。
