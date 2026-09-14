# Servo API 結構體欄位審計

## 檢查清單

### 1. KeyboardEvent ❌ ALREADY FIXED
**位置**: Line 421-428
**狀態**: 已修正
**原始代碼**: 使用分離欄位 (state, key, code, location, modifiers, repeat, is_composing)
**修正代碼**: 使用 `event` 欄位 `(Key, KeyState)` tuple
**結論**: ✅ 已修正為符合 Servo c038a19 API

### 2. MouseMoveEvent.new()
**位置**: Line 308
**構造代碼**: `MouseMoveEvent::new(point.into())`
**構造方式**: 使用 `.new()` 工廠方法
**潛在問題**: 需驗證 `new()` 方法簽名
**狀態**: ⚠️ 需檢查

### 3. MouseButtonEvent.new()
**位置**: Line 358-362
**構造代碼**: `MouseButtonEvent::new(action, servo_button, point.into())`
**構造方式**: 使用 `.new()` 工廠方法，三個參數
**參數**:
  - action: MouseButtonAction
  - servo_button: MouseButton
  - point.into(): Point
**狀態**: ⚠️ 需檢查

### 4. WheelEvent.new()
**位置**: Line 462
**構造代碼**: `WheelEvent::new(delta, point.into())`
**構造方式**: 使用 `.new()` 工廠方法，兩個參數
**參數**:
  - delta: WheelDelta
  - point.into(): Point
**狀態**: ⚠️ 需檢查

## 其他潛在問題區域

### 5. SoftwareRenderingContext.new()
**位置**: Line 76
**構造代碼**: `SoftwareRenderingContext::new(PhysicalSize::new(width, height))`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ⚠️ 需檢查

### 6. ServoBuilder.default().build()
**位置**: Line 81-82
**構造代碼**: `ServoBuilder::default().build()`
**構造方式**: Builder pattern
**狀態**: ⚠️ 需檢查

### 7. WebViewBuilder.new()
**位置**: Line 83-85
**構造代碼**: `WebViewBuilder::new(&servo, rendering_context.clone()).url(url).build()`
**構造方式**: Builder pattern with fluent API
**狀態**: ⚠️ 需檢查

### 8. PhysicalSize.new()
**位置**: Line 76, 333
**構造代碼**: `PhysicalSize::new(width, height)`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ✅ 標準 DPI crate，應該沒問題

### 9. DevicePoint.new()
**位置**: Line 305, 347, 459
**構造代碼**: `DevicePoint::new(x, y)`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ✅ 標準 euclid crate，應該沒問題

### 10. Point2D.zero()
**位置**: Line 201
**構造代碼**: `Point2D::zero()`
**構造方式**: 工廠方法
**狀態**: ✅ 標準 euclid crate，應該沒問題

### 11. Size2D.new()
**位置**: Line 202
**構造代碼**: `Size2D::new(webview.width as i32, webview.height as i32)`
**構造方式**: 使用 `.new()` 工廠方法
**狀態**: ✅ 標準 euclid crate，應該沒問題

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
**潛在問題**: 需驗證 WheelDelta 是否有這些欄位
**狀態**: ⚠️ 需檢查

## 建議的檢查優先級

### 高優先級（最可能有問題）
1. WheelDelta 結構體欄位 - 直接使用欄位初始化
2. MouseButtonEvent.new() - 參數數量和順序需確認
3. MouseMoveEvent.new() - 參數簽名需確認

### 中優先級
4. WheelEvent.new() - 參數順序需確認
5. WebViewBuilder 的 fluent API - 方法名稱和順序

### 低優先級
6. SoftwareRenderingContext.new() - 標準工廠方法
7. ServoBuilder pattern - Builder pattern 通常穩定

## 驗證方法

要驗證這些結構是否正確，需要：
1. 查看 Servo commit c038a19 的源代碼
2. 檢查每個結構體的 `pub struct` 定義
3. 檢查 `.new()` 方法的簽名
4. 確認所有欄位名稱和類型

## 修正跟蹤

- [x] KeyboardEvent - 已修正為使用 `event` 欄位
- [ ] MouseMoveEvent - 待驗證 (使用 .new() 工廠方法)
- [ ] MouseButtonEvent - 待驗證 (使用 .new() 工廠方法)
- [ ] WheelEvent - 待驗證 (使用 .new() 工廠方法)
- [ ] WheelDelta - 待驗證 (直接欄位初始化)

## 驗證計劃

### 當前狀態
1. 已修正 KeyboardEvent 的 E0560 編譯錯誤
2. 其他 Event 結構使用工廠方法 (`.new()`)，比直接欄位初始化更安全
3. 將通過 GitHub Actions cargo-check 進行最終驗證

### 下一步
如果 GitHub Actions 仍然報告錯誤，將按照以下優先級修正：
1. WheelDelta - 最可能有欄位名稱問題（直接初始化）
2. MouseButtonEvent - 參數順序或數量問題
3. MouseMoveEvent - 參數簽名問題
4. WheelEvent - 參數順序問題
