#[path = "../src/capture.rs"]
mod capture;

fn main() -> anyhow::Result<()> {
    let img = capture::capture_virtual_screen()?;
    let png = capture::encode_png(&img)?;
    std::fs::write("shot.png", &png)?;
    println!("wrote shot.png {}x{} ({} bytes)", img.w, img.h, png.len());
    Ok(())
}