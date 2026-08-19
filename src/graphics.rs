use base64::prelude::*;
use clap::ValueEnum;
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageReader, RgbaImage};
use std::env;
use std::fs;
use std::path::Path;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GraphicsProtocol {
    #[default]
    #[value(name = "auto", help = "Automatically detect terminal graphics protocol")]
    Auto,
    #[value(name = "kitty", help = "Kitty graphics protocol (Kitty, Ghostty, WezTerm)")]
    Kitty,
    #[value(name = "iterm2", help = "iTerm2 inline image protocol (iTerm2, WezTerm, Mintty)")]
    Iterm2,
    #[value(name = "sixel", help = "Sixel graphics protocol (WezTerm, Foot, Windows Terminal)")]
    Sixel,
    #[value(name = "blocks", help = "Half-block TrueColor Unicode rendering (universal fallback)")]
    Blocks,
}

pub fn detect_graphics_protocol() -> GraphicsProtocol {
    if env::var("KITTY_WINDOW_ID").is_ok() || env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
        return GraphicsProtocol::Kitty;
    }

    if let Ok(term_prog) = env::var("TERM_PROGRAM") {
        match term_prog.to_lowercase().as_str() {
            "iterm.app" => return GraphicsProtocol::Iterm2,
            "wezterm" => return GraphicsProtocol::Iterm2,
            "vscode" => return GraphicsProtocol::Iterm2,
            "ghostty" => return GraphicsProtocol::Kitty,
            _ => {}
        }
    }

    if let Ok(term) = env::var("TERM") {
        if term.contains("sixel") || term.contains("foot") || term.contains("mlterm") {
            return GraphicsProtocol::Sixel;
        }
    }

    // Default to high-contrast Half-Block rendering for universal compatibility
    GraphicsProtocol::Blocks
}

pub fn encode_iterm2_image(path: &Path, max_cols: usize, max_rows: usize) -> Option<String> {
    let file_bytes = fs::read(path).ok()?;
    let encoded = BASE64_STANDARD.encode(&file_bytes);
    Some(format!(
        "\x1b]1337;File=inline=1;width={}cell;height={}cell;preserveAspectRatio=1:{}\x07",
        max_cols, max_rows, encoded
    ))
}

pub fn encode_kitty_image(
    img: &DynamicImage,
    target_width_px: u32,
    target_height_px: u32,
) -> Option<String> {
    let resized = img.resize(target_width_px, target_height_px, FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let (w, h) = rgba.dimensions();
    let encoded = BASE64_STANDARD.encode(rgba.as_raw());

    Some(format!(
        "\x1b_Ga=T,f=32,s={},v={},m=0;{}\x1b\\",
        w, h, encoded
    ))
}

pub fn encode_sixel_image(
    img: &DynamicImage,
    target_width_px: u32,
    target_height_px: u32,
) -> Option<String> {
    let resized = img.resize(target_width_px, target_height_px, FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    let (w, h) = rgba.dimensions();

    let mut sixel = String::with_capacity((w * h) as usize);
    // Sixel Header: DCS P q
    sixel.push_str("\x1bPq\"1;1;");
    sixel.push_str(&w.to_string());
    sixel.push(';');
    sixel.push_str(&h.to_string());

    // Color quantization for 16 standard Sixel palette colors
    for color_idx in 0..16 {
        let r = (color_idx & 1) * 100;
        let g = ((color_idx >> 1) & 1) * 100;
        let b = ((color_idx >> 2) & 1) * 100;
        sixel.push_str(&format!("#{};2;{};{};{}", color_idx, r, g, b));
    }

    // Process six horizontal rows at a time
    for y_group in (0..h).step_by(6) {
        for color_idx in 0..16 {
            let mut row_has_color = false;
            let mut pattern_str = String::new();

            for x in 0..w {
                let mut byte_val = 0u8;
                for bit in 0..6 {
                    let y = y_group + bit;
                    if y < h {
                        let p = rgba.get_pixel(x, y);
                        if p[3] > 64 {
                            let lum = (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3;
                            let col = ((lum / 16) % 16) as usize;
                            if col == color_idx {
                                byte_val |= 1 << bit;
                            }
                        }
                    }
                }

                if byte_val != 0 {
                    row_has_color = true;
                }
                pattern_str.push((63 + byte_val) as char);
            }

            if row_has_color {
                sixel.push_str(&format!("#{}", color_idx));
                sixel.push_str(&pattern_str);
                sixel.push('$'); // Carriage return
            }
        }
        sixel.push('-'); // Next sixel line
    }

    sixel.push_str("\x1b\\"); // Sixel Terminator
    Some(sixel)
}

/// Enhance image contrast and apply adaptive sharpening for crisp thumbnail rendering
pub fn sharpen_and_enhance_thumbnail(img: &DynamicImage, width: u32, height: u32) -> RgbaImage {
    // 1. High-fidelity Lanczos3 Resampling
    let resized = img.resize(width, height, FilterType::Lanczos3);
    let mut rgba = resized.to_rgba8();

    // 2. Micro-contrast enhancement for terminal readability
    for pixel in rgba.pixels_mut() {
        if pixel[3] > 0 {
            // Apply gentle S-curve to boost midtone contrast
            for c in 0..3 {
                let val = pixel[c] as f32 / 255.0;
                let enhanced = if val < 0.5 {
                    2.0 * val * val
                } else {
                    1.0 - 2.0 * (1.0 - val) * (1.0 - val)
                };
                pixel[c] = (enhanced * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
    }

    rgba
}

pub fn get_image_dimensions_and_format(path: &Path) -> Option<(u32, u32, String)> {
    let reader = ImageReader::open(path).ok()?.with_guessed_format().ok()?;
    let format_name = reader
        .format()
        .map(|f| format!("{:?}", f).to_uppercase())
        .unwrap_or_else(|| "IMAGE".to_string());
    let dyn_img = reader.decode().ok()?;
    let (w, h) = dyn_img.dimensions();
    Some((w, h, format_name))
}
