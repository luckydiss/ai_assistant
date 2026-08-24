use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowDisplayAffinity, SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
};

fn hwnd(window: &tauri::WebviewWindow) -> anyhow::Result<HWND> {
    let raw = window.window_handle()?.as_raw();
    match raw {
        RawWindowHandle::Win32(h) => Ok(HWND(h.hwnd.get())),
        _ => anyhow::bail!("not a win32 window"),
    }
}

pub fn apply_affinity(window: &tauri::WebviewWindow) -> anyhow::Result<()> {
    let hwnd = hwnd(window)?;
    unsafe { SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)? };

    let mut current = 0u32;
    unsafe { GetWindowDisplayAffinity(hwnd, &mut current)? };
    anyhow::ensure!(current == WDA_EXCLUDEFROMCAPTURE.0, "affinity not applied");

    tracing::info!("stealth: WDA_EXCLUDEFROMCAPTURE applied");
    Ok(())
}

pub fn clear_affinity(window: &tauri::WebviewWindow) -> anyhow::Result<()> {
    let hwnd = hwnd(window)?;
    unsafe { SetWindowDisplayAffinity(hwnd, WDA_NONE)? };
    tracing::info!("stealth: WDA_EXCLUDEFROMCAPTURE cleared");
    Ok(())
}
