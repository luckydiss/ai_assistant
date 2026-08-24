use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct Rgba {
    pub data: Vec<u8>,
    pub w: i32,
    pub h: i32,
}

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
        bi.bmiHeader.biHeight = -h;
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = 0;
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let _ = GetDIBits(
            mem,
            bmp,
            0,
            h as u32,
            Some(buf.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );
        for px in buf.chunks_exact_mut(4) {
            px.swap(0, 2);
        }

        let _ = SelectObject(mem, old);
        let _ = DeleteObject(bmp);
        let _ = DeleteDC(mem);
        let _ = ReleaseDC(HWND::default(), hdc);
        Ok(Rgba { data: buf, w, h })
    }
}

#[allow(dead_code)] // used by bin (screen_analyze), not by example "shot"
pub fn capture_active_window() -> anyhow::Result<Rgba> {
    let full = capture_virtual_screen()?;
    unsafe {
        let fg = GetForegroundWindow();
        let mut r = RECT::default();
        let _ = GetWindowRect(fg, &mut r);
        let ox = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let oy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        Ok(crop(
            &full,
            r.left - ox,
            r.top - oy,
            r.right - r.left,
            r.bottom - r.top,
        ))
    }
}

#[allow(dead_code)] // used by capture_active_window (bin)
pub fn crop(img: &Rgba, x: i32, y: i32, w: i32, h: i32) -> Rgba {
    let (x, y) = (x.clamp(0, img.w - 1), y.clamp(0, img.h - 1));
    let w = w.min(img.w - x);
    let h = h.min(img.h - y);
    let mut out = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        let src = (((y + row) * img.w + x) * 4) as usize;
        let dst = (row * w * 4) as usize;
        out[dst..dst + (w * 4) as usize]
            .copy_from_slice(&img.data[src..src + (w * 4) as usize]);
    }
    Rgba { data: out, w, h }
}

pub fn encode_png(img: &Rgba) -> anyhow::Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.w as u32, img.h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header()?;
        wr.write_image_data(&img.data)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_window_region() {
        let img = Rgba {
            data: (0..16 * 16 * 4).map(|i| (i % 256) as u8).collect(),
            w: 16,
            h: 16,
        };
        let c = crop(&img, 4, 4, 8, 8);
        assert_eq!(c.w, 8);
        assert_eq!(c.h, 8);
        assert_eq!(c.data.len(), 8 * 8 * 4);
        for row in 0..8usize {
            let src = ((4 + row) * 16 + 4) * 4;
            let dst = row * 8 * 4;
            assert_eq!(&c.data[dst..dst + 8 * 4], &img.data[src..src + 8 * 4]);
        }
    }

    #[test]
    fn crop_clamps_bounds() {
        let img = Rgba {
            data: vec![0u8; 10 * 10 * 4],
            w: 10,
            h: 10,
        };
        let c = crop(&img, 8, 8, 10, 10);
        assert_eq!(c.w, 2);
        assert_eq!(c.h, 2);
    }

    #[test]
    fn crop_empty_when_out_of_bounds() {
        let img = Rgba {
            data: vec![0u8; 10 * 10 * 4],
            w: 10,
            h: 10,
        };
        let c = crop(&img, -5, -5, 0, 0);
        assert_eq!(c.w, 0);
        assert_eq!(c.h, 0);
    }
}