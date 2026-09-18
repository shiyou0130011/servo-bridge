# Servo API 結構體欄位審計

## 檢查清單

### 1. KeyboardEvent
**位置**: `src/lib.rs` 426–436  
**狀態**: 已實作  
**實作方式**:
1. 以 `keyboard_types::KeyboardEvent` 的欄位建立底層鍵盤事件。
2. 將底層事件傳入 Servo 匯出的 `KeyboardEvent::new(ktype_event)`。
3. 以 `InputEvent::Keyboard` 傳送至 WebView。
**結論**: ✅ 已依照 Servo c038a19 的 API 建構鍵盤事件。

### 2. MouseMoveEvent.new()
**位置**: `src/lib.rs` 308  
**構造代碼**: `MouseMoveEvent::new(point.into())`  
**構造方式**: 使用 `.new()` 工廠方法  
**狀態**: ✅ 已實作並在 GitHub Actions CI（cargo-check.yml）中驗證。

### 3. MouseButtonEvent.new()
**位置**: `src/lib.rs` 358–362  
**構造代碼**: `MouseButtonEvent::new(action, servo_button, point.into())`  
**構造方式**: 使用 `.new()` 工廠方法，三個參數  
**參數**:
  - `action`: `MouseButtonAction`（`Down` / `Up`）
  - `servo_button`: `MouseButton`（`Primary` / `Secondary` / `Auxiliary` / `Other(u16)`）
  - `point.into()`: `Point`（由 `DevicePoint` 轉換）
**狀態**: ✅ 已實作並在 GitHub Actions CI 中驗證。

### 4. WheelEvent.new()
**位置**: `src/lib.rs` 471  
**構造代碼**: `WheelEvent::new(delta, point.into())`  
**構造方式**: 使用 `.new()` 工廠方法，兩個參數  
**參數**:
  - `delta`: `WheelDelta`
  - `point.into()`: `Point`
**狀態**: ✅ 已實作並在 GitHub Actions CI 中驗證。

## 其他 API 區域

### 5. SoftwareRenderingContext::new()
**位置**: `src/lib.rs` 72  
**構造代碼**: `SoftwareRenderingContext::new(PhysicalSize::new(width, height))`  
**構造方式**: 使用 `.new()` 工廠方法  
**狀態**: ✅ 已實作。

### 6. ServoBuilder::default().build()
**位置**: `src/lib.rs` 81  
**構造代碼**: `ServoBuilder::default().build()`  
**構造方式**: Builder pattern  
**狀態**: ✅ 已實作。

### 7. WebViewBuilder::new()
**位置**: `src/lib.rs` 83–85  
**構造代碼**: `WebViewBuilder::new(&servo, rendering_context.clone()).url(url).build()`  
**構造方式**: Builder pattern with fluent API  
**狀態**: ✅ 已實作。

### 8. PhysicalSize::new()
**位置**: `src/lib.rs` 76, 333  
**構造代碼**: `PhysicalSize::new(width, height)`  
**構造方式**: 使用 `.new()` 工廠方法  
**狀態**: ✅ 來自 `dpi` crate。

### 9. DevicePoint::new()
**位置**: `src/lib.rs` 305, 347, 459  
**構造代碼**: `DevicePoint::new(x, y)`  
**構造方式**: 使用 `.new()` 工廠方法  
**狀態**: ✅ 來自 `euclid` crate。

### 10. Point2D::zero()
**位置**: `src/lib.rs` 201  
**構造代碼**: `Point2D::zero()`  
**構造方式**: 工廠方法  
**狀態**: ✅ 來自 `euclid` crate。

### 11. Size2D::new()
**位置**: `src/lib.rs` 202  
**構造代碼**: `Size2D::new(webview.width as i32, webview.height as i32)`  
**構造方式**: 使用 `.new()` 工廠方法  
**狀態**: ✅ 來自 `euclid` crate。

### 12. WheelDelta {}
**位置**: `src/lib.rs` 457–461  
**構造代碼**:
```rust
WheelDelta {
    x: delta_x as f64,
    y: delta_y as f64,
    z: 0.0,
    mode: WheelMode::DeltaPixel,
}
```
**構造方式**: 結構體欄位直接初始化（Servo c038a19 的 `WheelDelta` 為公開欄位結構體，不需要工廠方法）  
**狀態**: ✅ 已實作。

## 驗證方式

本機環境未安裝 Cargo，所有 API 相容性透過 GitHub Actions CI
（`.github/workflows/cargo-check.yml`）在具備 Rust 工具鏈的環境驗證。

## 修正跟蹤

- [x] KeyboardEvent — 使用 `keyboard_types::KeyboardEvent` 欄位建立事件，再呼叫 Servo 的 `KeyboardEvent::new()`。
- [x] MouseMoveEvent — `MouseMoveEvent::new(point.into())` 已在 CI 驗證。
- [x] MouseButtonEvent — `MouseButtonEvent::new(action, button, point.into())` 已在 CI 驗證。
- [x] WheelEvent — `WheelEvent::new(delta, point.into())` 已在 CI 驗證。
- [x] WheelDelta — 欄位直接初始化，符合 Servo c038a19。

## 最終驗證狀態

所有 Servo API 呼叫均已在 `src/lib.rs` 完整實作，並透過 GitHub Actions CI
（`cargo-check.yml`）在 Servo c038a19 上驗證相容性。固定的 Servo commit
（`c038a19`）確保 API 不會隨上游變動而失效。
