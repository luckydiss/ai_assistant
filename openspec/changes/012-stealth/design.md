# Design: Stealth

## 1. Обновление workspace Cargo.toml

Заменить features у `windows`:

```toml
windows = { version = "0.52", features = [
  "Win32_UI_WindowsAndMessaging",
  "Win32_Foundation",
] }
```

## 2. apps/desktop/src/stealth.rs

```rust
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowDisplayAffinity, SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
};

pub fn apply_affinity(window: &tauri::WebviewWindow) -> anyhow::Result<()> {
    let raw = window.raw_window_handle()?;
    let hwnd = match raw {
        RawWindowHandle::Win32(h) => HWND(h.hwnd.get() as isize),
        _ => anyhow::bail!("not a win32 window"),
    };

    let ok = unsafe { SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) };
    anyhow::ensure!(ok.as_bool(), "SetWindowDisplayAffinity failed");

    let mut current = Default::default();
    let ok = unsafe { GetWindowDisplayAffinity(hwnd, &mut current) };
    anyhow::ensure!(ok.as_bool(), "GetWindowDisplayAffinity failed");
    anyhow::ensure!(current == WDA_EXCLUDEFROMCAPTURE, "affinity not applied");

    tracing::info!("stealth: WDA_EXCLUDEFROMCAPTURE applied");
    Ok(())
}
```

## 3. Подключение в main.rs setup (после создания окна)

```rust
mod stealth;
// в setup, после hotkeys:
let win = app.get_webview_window("main").unwrap();
stealth::apply_affinity(&win)?;
```

## 4. Manual verification checklist (выполняет человек)

1. Запустить приложение, открыть оверлей с тестовым ответом.
2. Zoom: Start Share → Screen → проверить с другого устройства: оверлея нет.
3. OBS: Display Capture → проверить превью: оверлея нет.
4. Win+Shift+S (snipping): оверлей не попадает в снимок.

## Рассмотрено и отклонено
- **DXGI Desktop Duplication self-check:** отклонено для MVP (требует D3D11-обвязки ~150 строк); достаточно ручного чек-листа
- **WDA_MONITOR вместо EXCLUDEFROMCAPTURE:** отклонено — MONITOR показывает чёрный квадрат, EXCLUDE полностью исключает
