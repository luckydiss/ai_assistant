# Design: Screenshots + Vision

## 1. Workspace Cargo.toml

```toml
png = "0.17"
# windows features дополнить: "Win32_Graphics_Gdi"
```

## 2. apps/desktop/src/capture.rs (GDI)

```rust
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct Rgba { pub data: Vec<u8>, pub w: i32, pub h: i32 }

pub fn capture_virtual_screen() -> anyhow::Result<Rgba> {
    unsafe {
        let hdc = GetDC(None);
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let w = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let h = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        let mem = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, w, h);
        let old = SelectObject(mem, bmp);
        let _ = BitBlt(mem, 0, 0, w, h, hdc, x, y, SRCCOPY);

        let mut bi = BITMAPINFO::default();
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = w;
        bi.bmiHeader.biHeight = -h; // top-down
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = 0; // BI_RGB
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let _ = GetDIBits(mem, bmp, 0, h as u32, buf.as_mut_ptr() as *mut _, &mut bi, DIB_RGB_COLORS);
        for px in buf.chunks_exact_mut(4) { px.swap(0, 2); } // BGRA -> RGBA

        let _ = SelectObject(mem, old);
        let _ = DeleteObject(bmp);
        let _ = DeleteDC(mem);
        let _ = ReleaseDC(None, hdc);
        Ok(Rgba { data: buf, w, h })
    }
}

pub fn capture_active_window() -> anyhow::Result<Rgba> {
    let full = capture_virtual_screen()?;
    unsafe {
        let fg = GetForegroundWindow();
        let mut r = RECT::default();
        let _ = GetWindowRect(fg, &mut r);
        let ox = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let oy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        Ok(crop(&full, r.left - ox, r.top - oy, r.right - r.left, r.bottom - r.top))
    }
}

/// Чистый кроп; тестируется без Windows-API.
pub fn crop(img: &Rgba, x: i32, y: i32, w: i32, h: i32) -> Rgba {
    let (x, y) = (x.clamp(0, img.w - 1), y.clamp(0, img.h - 1));
    let w = w.min(img.w - x); let h = h.min(img.h - y);
    let mut out = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        let src = (((y + row) * img.w + x) * 4) as usize;
        let dst = (row * w * 4) as usize;
        out[dst..dst + (w * 4) as usize].copy_from_slice(&img.data[src..src + (w * 4) as usize]);
    }
    Rgba { data: out, w, h }
}

pub fn encode_png(img: &Rgba) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut enc = png::Encoder::new(&mut data, img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header()?;
    wr.write_image_data(&img.data)?;
    Ok(data)
}
```

## 3. Мультимодальность (engine-context)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent { Text(String), Parts(Vec<Part>) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    #[serde(rename = "type")] pub kind: String,           // "text" | "image_url"
    #[serde(skip_serializing_if = "Option::is_none")] pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub image_url: Option<ImageUrl>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl { pub url: String }

// ChatMessage.content: MessageContent
```

ContextBuilder::build(..., image_b64: Option<&str>): если Some — user-сообщение = Parts([image_url(data:image/png;base64,..), text(user_content)]), иначе Text как раньше. Все старые вызовы передают None.

## 4. Orchestrator

`manual(&self, note: Option<String>, image_b64: Option<String>)`; fire(note, image) → ctx.build(..., image). Старые вызовы manual(note) → manual(note, None).

## 5. Команды и хоткеи

```rust
#[tauri::command] async fn screen_analyze(orch: State<'_, Arc<Orchestrator>>, window_only: bool) -> Result<(), String> {
    let rgba = if window_only { capture_active_window()? } else { capture_virtual_screen()? };
    let png = encode_png(&rgba)?;
    let b64 = base64_encode(&png); // base64::engine::general_purpose::STANDARD (crate base64 = "0.21")
    orch.manual(Some("Проанализируй скриншот и помоги с задачей на экране".into()), Some(b64));
    Ok(())
}
```

dispatch hotkeys: screenshot_full → screen_analyze(false), screenshot_region → screen_analyze(true).
Overlay: кнопка «Анализ экрана» → invoke screen_analyze(false).

## 6. Mock-сервер с захватом тела (tests)

Расширить mock.rs: `spawn_mock_sse_capture(body)` возвращает (url, Arc<Mutex<String>> last_body). Тест vision_payload_sent: manual(None, Some("QUJD")) → last_body содержит "image_url".

## Рассмотрено и отклонено
- **xcap/image crate:** отклонено — GDI+png уже в матрице, меньше неизвестного API
- **Drag-select:** отклонено (MVP: активное окно)
