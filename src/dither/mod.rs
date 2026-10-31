use crate::document::DitherMode;

/// Converts an RGB or RGBA byte buffer to grayscale (0..255 where 0=black, 255=white)
pub fn to_grayscale(rgba: &[u8], width: usize, height: usize, channels: usize) -> Vec<f32> {
    let mut gray = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * channels;
            let r = rgba[idx] as f32;
            let g = rgba[idx + 1] as f32;
            let b = rgba[idx + 2] as f32;
            let a = if channels >= 4 { rgba[idx + 3] as f32 / 255.0 } else { 1.0 };
            // Standard luminance: 0.299 R + 0.587 G + 0.114 B
            let luma = 0.299 * r + 0.587 * g + 0.114 * b;
            // White background alpha blend
            let blended = luma * a + 255.0 * (1.0 - a);
            gray.push(blended);
        }
    }
    gray
}

/// Resamples a grayscale image buffer to target width and height using bilinear interpolation
pub fn resample_grayscale(
    src: &[f32],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
) -> Vec<f32> {
    if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
        return Vec::new();
    }
    if src_w == dst_w && src_h == dst_h {
        return src.to_vec();
    }
    let mut dst = Vec::with_capacity(dst_w * dst_h);
    let scale_x = src_w as f64 / dst_w as f64;
    let scale_y = src_h as f64 / dst_h as f64;

    for dy in 0..dst_h {
        let sy = (dy as f64 + 0.5) * scale_y - 0.5;
        let y0 = (sy.floor() as isize).clamp(0, src_h as isize - 1) as usize;
        let y1 = (y0 + 1).min(src_h - 1);
        let fy = (sy - y0 as f64).clamp(0.0, 1.0) as f32;

        for dx in 0..dst_w {
            let sx = (dx as f64 + 0.5) * scale_x - 0.5;
            let x0 = (sx.floor() as isize).clamp(0, src_w as isize - 1) as usize;
            let x1 = (x0 + 1).min(src_w - 1);
            let fx = (sx - x0 as f64).clamp(0.0, 1.0) as f32;

            let p00 = src[y0 * src_w + x0];
            let p10 = src[y0 * src_w + x1];
            let p01 = src[y1 * src_w + x0];
            let p11 = src[y1 * src_w + x1];

            let top = p00 * (1.0 - fx) + p10 * fx;
            let bottom = p01 * (1.0 - fx) + p11 * fx;
            let val = top * (1.0 - fy) + bottom * fy;

            dst.push(val);
        }
    }
    dst
}

/// Runs the selected dithering algorithm on a grayscale image buffer.
/// Returns a 1-bit boolean mask where `true` means laser fires (black dot / engrave)
/// and `false` means laser off (white dot).
pub fn dither(
    grayscale: &[f32],
    width: usize,
    height: usize,
    mode: DitherMode,
) -> Vec<bool> {
    match mode {
        DitherMode::Threshold => dither_threshold(grayscale, width, height, 128.0),
        DitherMode::FloydSteinberg => dither_floyd_steinberg(grayscale, width, height),
        DitherMode::Atkinson => dither_atkinson(grayscale, width, height),
        DitherMode::Jarvis => dither_jarvis(grayscale, width, height),
        DitherMode::Stucki => dither_stucki(grayscale, width, height),
        DitherMode::OrderedBayer => dither_ordered_bayer_8x8(grayscale, width, height),
        DitherMode::Halftone => dither_halftone(grayscale, width, height, 4.0),
        DitherMode::GrayscalePower => dither_threshold(grayscale, width, height, 128.0),
        DitherMode::Sliced3D => dither_threshold(grayscale, width, height, 128.0),
    }
}

pub fn dither_threshold(
    gray: &[f32],
    width: usize,
    height: usize,
    threshold: f32,
) -> Vec<bool> {
    let mut out = Vec::with_capacity(width * height);
    for &p in gray {
        // < threshold means darker than threshold -> fire laser!
        out.push(p < threshold);
    }
    out
}

pub fn dither_floyd_steinberg(
    gray: &[f32],
    width: usize,
    height: usize,
) -> Vec<bool> {
    let mut buffer = gray.to_vec();
    let mut out = vec![false; width * height];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let old_val = buffer[idx];
            let new_val = if old_val < 128.0 { 0.0 } else { 255.0 };
            out[idx] = new_val == 0.0; // laser fires for black
            let err = old_val - new_val;

            // Distribute error
            // (x+1, y)   : 7/16
            // (x-1, y+1) : 3/16
            // (x,   y+1) : 5/16
            // (x+1, y+1) : 1/16
            if x + 1 < width {
                buffer[y * width + (x + 1)] += err * (7.0 / 16.0);
            }
            if y + 1 < height {
                if x > 0 {
                    buffer[(y + 1) * width + (x - 1)] += err * (3.0 / 16.0);
                }
                buffer[(y + 1) * width + x] += err * (5.0 / 16.0);
                if x + 1 < width {
                    buffer[(y + 1) * width + (x + 1)] += err * (1.0 / 16.0);
                }
            }
        }
    }
    out
}

pub fn dither_atkinson(
    gray: &[f32],
    width: usize,
    height: usize,
) -> Vec<bool> {
    let mut buffer = gray.to_vec();
    let mut out = vec![false; width * height];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let old_val = buffer[idx];
            let new_val = if old_val < 128.0 { 0.0 } else { 255.0 };
            out[idx] = new_val == 0.0;
            let err = old_val - new_val;
            let share = err * (1.0 / 8.0); // 6 neighbors each get 1/8th (2/8 is discarded for crisp highlights)

            if x + 1 < width {
                buffer[y * width + (x + 1)] += share;
            }
            if x + 2 < width {
                buffer[y * width + (x + 2)] += share;
            }
            if y + 1 < height {
                if x > 0 {
                    buffer[(y + 1) * width + (x - 1)] += share;
                }
                buffer[(y + 1) * width + x] += share;
                if x + 1 < width {
                    buffer[(y + 1) * width + (x + 1)] += share;
                }
            }
            if y + 2 < height {
                buffer[(y + 2) * width + x] += share;
            }
        }
    }
    out
}

pub fn dither_jarvis(
    gray: &[f32],
    width: usize,
    height: usize,
) -> Vec<bool> {
    let mut buffer = gray.to_vec();
    let mut out = vec![false; width * height];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let old_val = buffer[idx];
            let new_val = if old_val < 128.0 { 0.0 } else { 255.0 };
            out[idx] = new_val == 0.0;
            let err = old_val - new_val;
            let div = 1.0 / 48.0;

            let offsets = [
                (1, 0, 7.0), (2, 0, 5.0),
                (-2, 1, 3.0), (-1, 1, 5.0), (0, 1, 7.0), (1, 1, 5.0), (2, 1, 3.0),
                (-2, 2, 1.0), (-1, 2, 3.0), (0, 2, 5.0), (1, 2, 3.0), (2, 2, 1.0),
            ];

            for (dx, dy, weight) in offsets {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                    buffer[ny as usize * width + nx as usize] += err * (weight * div);
                }
            }
        }
    }
    out
}

pub fn dither_stucki(
    gray: &[f32],
    width: usize,
    height: usize,
) -> Vec<bool> {
    let mut buffer = gray.to_vec();
    let mut out = vec![false; width * height];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let old_val = buffer[idx];
            let new_val = if old_val < 128.0 { 0.0 } else { 255.0 };
            out[idx] = new_val == 0.0;
            let err = old_val - new_val;
            let div = 1.0 / 42.0;

            let offsets = [
                (1, 0, 8.0), (2, 0, 4.0),
                (-2, 1, 2.0), (-1, 1, 4.0), (0, 1, 8.0), (1, 1, 4.0), (2, 1, 2.0),
                (-2, 2, 1.0), (-1, 2, 2.0), (0, 2, 4.0), (1, 2, 2.0), (2, 2, 1.0),
            ];

            for (dx, dy, weight) in offsets {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                    buffer[ny as usize * width + nx as usize] += err * (weight * div);
                }
            }
        }
    }
    out
}

pub fn dither_ordered_bayer_8x8(
    gray: &[f32],
    width: usize,
    height: usize,
) -> Vec<bool> {
    #[rustfmt::skip]
    const BAYER_8X8: [f32; 64] = [
         0.0/64.0, 32.0/64.0,  8.0/64.0, 40.0/64.0,  2.0/64.0, 34.0/64.0, 10.0/64.0, 42.0/64.0,
        48.0/64.0, 16.0/64.0, 56.0/64.0, 24.0/64.0, 50.0/64.0, 18.0/64.0, 58.0/64.0, 26.0/64.0,
        12.0/64.0, 44.0/64.0,  4.0/64.0, 36.0/64.0, 14.0/64.0, 46.0/64.0,  6.0/64.0, 38.0/64.0,
        60.0/64.0, 28.0/64.0, 52.0/64.0, 20.0/64.0, 62.0/64.0, 30.0/64.0, 54.0/64.0, 22.0/64.0,
         3.0/64.0, 35.0/64.0, 11.0/64.0, 43.0/64.0,  1.0/64.0, 33.0/64.0,  9.0/64.0, 41.0/64.0,
        51.0/64.0, 19.0/64.0, 59.0/64.0, 27.0/64.0, 49.0/64.0, 17.0/64.0, 57.0/64.0, 25.0/64.0,
        15.0/64.0, 47.0/64.0,  7.0/64.0, 39.0/64.0, 13.0/64.0, 45.0/64.0,  5.0/64.0, 37.0/64.0,
        63.0/64.0, 31.0/64.0, 55.0/64.0, 23.0/64.0, 61.0/64.0, 29.0/64.0, 53.0/64.0, 21.0/64.0,
    ];

    let mut out = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let matrix_threshold = BAYER_8X8[(y % 8) * 8 + (x % 8)] * 255.0;
            out.push(gray[idx] < matrix_threshold);
        }
    }
    out
}

pub fn dither_halftone(
    gray: &[f32],
    width: usize,
    height: usize,
    cell_size: f32,
) -> Vec<bool> {
    let mut out = Vec::with_capacity(width * height);
    let half = cell_size * 0.5;

    for y in 0..height {
        for x in 0..width {
            let cx = (x as f32 % cell_size) - half;
            let cy = (y as f32 % cell_size) - half;
            let dist = (cx * cx + cy * cy).sqrt() / (half * 1.414);

            let intensity = 1.0 - (gray[y * width + x] / 255.0); // 1.0 = pitch black
            out.push(dist < intensity);
        }
    }
    out
}

/// Applies non-destructive Brightness (-100..100), Contrast (-100..100), Gamma (0.1..5.0),
/// and Unsharp-Mask Sharpening (0..100) to a grayscale buffer (0..255).
pub fn preprocess_grayscale(
    gray: &[f32],
    width: usize,
    height: usize,
    brightness: f32,
    contrast: f32,
    gamma: f32,
    sharpen: f32,
) -> Vec<f32> {
    if gray.is_empty() {
        return Vec::new();
    }
    let is_neutral = brightness.abs() < 1e-3
        && contrast.abs() < 1e-3
        && (gamma - 1.0).abs() < 1e-3
        && sharpen < 1e-3;
    if is_neutral {
        return gray.to_vec();
    }

    // 1. Optional 3x3 Unsharp Mask / Laplacian sharpening for crisp engraving edges
    let mut buf = if sharpen > 1e-3 && width >= 3 && height >= 3 {
        let amount = (sharpen / 100.0).clamp(0.0, 2.0) * 1.5;
        let mut sharpened = gray.to_vec();
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let idx = y * width + x;
                let c = gray[idx];
                let top = gray[(y - 1) * width + x];
                let bot = gray[(y + 1) * width + x];
                let left = gray[y * width + (x - 1)];
                let right = gray[y * width + (x + 1)];
                let lap = 4.0 * c - top - bot - left - right;
                sharpened[idx] = (c + amount * lap).clamp(0.0, 255.0);
            }
        }
        sharpened
    } else {
        gray.to_vec()
    };

    // 2. Brightness, Contrast, and Gamma
    let c_scaled = (contrast.clamp(-100.0, 100.0) * 2.54).clamp(-254.0, 254.0);
    let factor = (259.0 * (c_scaled + 255.0)) / (255.0 * (259.0 - c_scaled));
    let b_offset = brightness.clamp(-100.0, 100.0) * 2.55;
    let g_clamped = gamma.clamp(0.1, 5.0);
    let apply_gamma = (g_clamped - 1.0).abs() > 1e-3;
    let inv_gamma = 1.0 / g_clamped;

    for v in &mut buf {
        let mut p = factor * (*v + b_offset - 128.0) + 128.0;
        p = p.clamp(0.0, 255.0);
        if apply_gamma {
            p = 255.0 * (p / 255.0).powf(inv_gamma);
        }
        *v = p.clamp(0.0, 255.0);
    }

    buf
}

fn parse_hex_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(200);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(200);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(200);
        (r, g, b)
    } else {
        (200, 200, 200)
    }
}

/// Generates an RGBA preview image showing live Brightness/Contrast/Gamma/Sharpen/Invert adjustments,
/// and optionally simulates multi-layer Ceramic Tile paint colors based on laser power thresholds.
pub fn render_processed_preview_rgba(
    rgba: &[u8],
    width: usize,
    height: usize,
    brightness: f32,
    contrast: f32,
    gamma: f32,
    sharpen: f32,
    invert_image: bool,
    ceramic_preview: bool,
    power_min: f64,
    power_max: f64,
    ceramic_bands: &[crate::document::CeramicColorBand],
) -> Vec<u8> {
    let is_neutral = brightness.abs() < 1e-3
        && contrast.abs() < 1e-3
        && (gamma - 1.0).abs() < 1e-3
        && sharpen < 1e-3
        && !invert_image
        && !ceramic_preview;
    if is_neutral {
        return rgba.to_vec();
    }

    let gray = to_grayscale(rgba, width, height, 4);
    let processed = preprocess_grayscale(&gray, width, height, brightness, contrast, gamma, sharpen);

    let parsed_bands: Vec<(f64, (u8, u8, u8))> = ceramic_bands
        .iter()
        .map(|b| (b.max_power_pct, parse_hex_rgb(&b.color_hex)))
        .collect();

    let p_min = power_min.clamp(0.0, 100.0);
    let p_max = power_max.clamp(0.0, 100.0);

    let mut out = Vec::with_capacity(width * height * 4);
    for (i, &val_raw) in processed.iter().enumerate() {
        let a = rgba.get(i * 4 + 3).copied().unwrap_or(255);
        let mut val = val_raw;
        if invert_image {
            val = 255.0 - val;
        }

        if ceramic_preview && !parsed_bands.is_empty() {
            let power_pct = if val >= 254.0 {
                0.0
            } else {
                let darkness = ((254.0 - val) / 254.0).clamp(0.0, 1.0) as f64;
                p_min + (p_max - p_min) * darkness
            };

            // Find matching band and smoothly blend within 3% around transitions for realistic paint ablation
            let mut chosen_rgb = parsed_bands.last().unwrap().1;
            let mut prev_max = 0.0;
            for (idx, &(band_max, rgb)) in parsed_bands.iter().enumerate() {
                if power_pct <= band_max {
                    // Check smooth transition from previous band
                    if idx > 0 && (power_pct - prev_max) < 3.0 {
                        let t = ((power_pct - prev_max) / 3.0).clamp(0.0, 1.0) as f32;
                        let prev_rgb = parsed_bands[idx - 1].1;
                        let r = (prev_rgb.0 as f32 * (1.0 - t) + rgb.0 as f32 * t).round() as u8;
                        let g = (prev_rgb.1 as f32 * (1.0 - t) + rgb.1 as f32 * t).round() as u8;
                        let b = (prev_rgb.2 as f32 * (1.0 - t) + rgb.2 as f32 * t).round() as u8;
                        chosen_rgb = (r, g, b);
                    } else {
                        chosen_rgb = rgb;
                    }
                    break;
                }
                prev_max = band_max;
            }
            out.extend_from_slice(&[chosen_rgb.0, chosen_rgb.1, chosen_rgb.2, a]);
        } else {
            let g = val.round().clamp(0.0, 255.0) as u8;
            out.extend_from_slice(&[g, g, g, a]);
        }
    }
    out
}
