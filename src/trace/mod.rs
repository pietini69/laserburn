//! Bitmap to Vector (Image Trace) engine for LaserBurn.
//! Converts raster images into clean cutting/engraving vector polylines using
//! adaptive thresholding, multi-scale smoothing, and sub-pixel Marching Squares contour extraction.

use crate::geometry::{Point2D, Polyline};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TraceOptions {
    /// Grayscale threshold (0..=255). Default is 128.
    pub threshold: u8,
    /// Invert image (if true, bright pixels become vectors; if false, dark pixels become vectors).
    pub invert: bool,
    /// Box blur radius (0..=5) to filter out noise, dither specks, or JPEG artifacts.
    pub blur_radius: usize,
    /// Minimum contour bounding-box area in mm² to ignore small dust/speckles.
    pub min_area_mm2: f64,
    /// Collinear vertex simplification tolerance in mm.
    pub simplify_tolerance_mm: f64,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            threshold: 128,
            invert: false,
            blur_radius: 1,
            min_area_mm2: 0.2,
            simplify_tolerance_mm: 0.08,
        }
    }
}

/// Traces a raster image into closed vector polylines in document millimeter coordinates.
pub fn trace_image_to_polylines(
    raw_pixels: &[u8],
    width: usize,
    height: usize,
    channels: usize,
    origin_x: f64,
    origin_y: f64,
    width_mm: f64,
    height_mm: f64,
    options: &TraceOptions,
) -> Vec<Polyline> {
    if width < 3 || height < 3 || raw_pixels.is_empty() {
        return Vec::new();
    }

    // 1. Convert to grayscale float grid [0.0, 255.0]
    let total_pixels = width * height;
    let mut gray = Vec::with_capacity(total_pixels);

    for i in 0..total_pixels {
        let px = match channels {
            1 => raw_pixels.get(i).copied().unwrap_or(255) as f32,
            3 => {
                let off = i * 3;
                let r = raw_pixels.get(off).copied().unwrap_or(255) as f32;
                let g = raw_pixels.get(off + 1).copied().unwrap_or(255) as f32;
                let b = raw_pixels.get(off + 2).copied().unwrap_or(255) as f32;
                0.299 * r + 0.587 * g + 0.114 * b
            }
            4 => {
                let off = i * 4;
                let a = raw_pixels.get(off + 3).copied().unwrap_or(255);
                if a < 128 {
                    255.0 // Transparent treated as white/background
                } else {
                    let r = raw_pixels.get(off).copied().unwrap_or(255) as f32;
                    let g = raw_pixels.get(off + 1).copied().unwrap_or(255) as f32;
                    let b = raw_pixels.get(off + 2).copied().unwrap_or(255) as f32;
                    0.299 * r + 0.587 * g + 0.114 * b
                }
            }
            _ => 255.0,
        };
        gray.push(px);
    }

    // 2. Optional Box Blur for noise suppression
    if options.blur_radius > 0 {
        let r = options.blur_radius;
        // Horizontal pass
        let mut temp = vec![0.0f32; total_pixels];
        for y in 0..height {
            let row_off = y * width;
            for x in 0..width {
                let mut sum = 0.0f32;
                let mut count = 0;
                let x_min = x.saturating_sub(r);
                let x_max = (x + r).min(width - 1);
                for kx in x_min..=x_max {
                    sum += gray[row_off + kx];
                    count += 1;
                }
                temp[row_off + x] = sum / (count as f32);
            }
        }
        // Vertical pass
        for y in 0..height {
            let y_min = y.saturating_sub(r);
            let y_max = (y + r).min(height - 1);
            for x in 0..width {
                let mut sum = 0.0f32;
                let mut count = 0;
                for ky in y_min..=y_max {
                    sum += temp[ky * width + x];
                    count += 1;
                }
                gray[y * width + x] = sum / (count as f32);
            }
        }
    }

    // 3. Compute signed distance field relative to threshold
    // Foreground (vectors) will be <= 0.0
    let thresh = options.threshold as f32;
    let mut field = Vec::with_capacity(total_pixels);
    for val in gray {
        let dist = if options.invert {
            val - thresh // If inverted: bright pixels (> thresh) are inside (<= 0)
        } else {
            thresh - val // Standard: dark pixels (< thresh) are inside (<= 0)
        };
        field.push(dist);
    }

    // 4. Sub-pixel Marching Squares
    let cell_x = width_mm / ((width - 1) as f64);
    let cell_y = height_mm / ((height - 1) as f64);

    let interp = |v0: f32, v1: f32| -> f64 {
        let denom = (v0 - v1) as f64;
        if denom.abs() < 1e-6 {
            0.5
        } else {
            ((v0 as f64) / denom).clamp(0.0, 1.0)
        }
    };

    let mut next_edge: HashMap<usize, usize> = HashMap::new();
    let mut edge_pos: HashMap<usize, Point2D> = HashMap::new();

    for iy in 0..(height - 1) {
        let y0 = origin_y + (iy as f64) * cell_y;
        let y1 = y0 + cell_y;
        for ix in 0..(width - 1) {
            let x0 = origin_x + (ix as f64) * cell_x;
            let x1 = x0 + cell_x;

            let v00 = field[iy * width + ix];
            let v10 = field[iy * width + ix + 1];
            let v11 = field[(iy + 1) * width + ix + 1];
            let v01 = field[(iy + 1) * width + ix];

            let mut mask = 0u8;
            if v00 <= 0.0 { mask |= 1; }
            if v10 <= 0.0 { mask |= 2; }
            if v11 <= 0.0 { mask |= 4; }
            if v01 <= 0.0 { mask |= 8; }

            if mask == 0 || mask == 15 {
                continue;
            }

            let e_top = (iy * width + ix) * 2;
            let e_right = (iy * width + (ix + 1)) * 2 + 1;
            let e_bottom = ((iy + 1) * width + ix) * 2;
            let e_left = (iy * width + ix) * 2 + 1;

            if (mask & 1 != 0) != (mask & 2 != 0) {
                edge_pos.entry(e_top).or_insert_with(|| Point2D::new(x0 + interp(v00, v10) * cell_x, y0));
            }
            if (mask & 2 != 0) != (mask & 4 != 0) {
                edge_pos.entry(e_right).or_insert_with(|| Point2D::new(x1, y0 + interp(v10, v11) * cell_y));
            }
            if (mask & 8 != 0) != (mask & 4 != 0) {
                edge_pos.entry(e_bottom).or_insert_with(|| Point2D::new(x0 + interp(v01, v11) * cell_x, y1));
            }
            if (mask & 1 != 0) != (mask & 8 != 0) {
                edge_pos.entry(e_left).or_insert_with(|| Point2D::new(x0, y0 + interp(v00, v01) * cell_y));
            }

            match mask {
                1 => { next_edge.insert(e_top, e_left); }
                2 => { next_edge.insert(e_right, e_top); }
                3 => { next_edge.insert(e_right, e_left); }
                4 => { next_edge.insert(e_bottom, e_right); }
                5 => {
                    let center = 0.25 * (v00 + v10 + v11 + v01);
                    if center <= 0.0 {
                        next_edge.insert(e_top, e_right);
                        next_edge.insert(e_bottom, e_left);
                    } else {
                        next_edge.insert(e_top, e_left);
                        next_edge.insert(e_bottom, e_right);
                    }
                }
                6 => { next_edge.insert(e_bottom, e_top); }
                7 => { next_edge.insert(e_bottom, e_left); }
                8 => { next_edge.insert(e_left, e_bottom); }
                9 => { next_edge.insert(e_top, e_bottom); }
                10 => {
                    let center = 0.25 * (v00 + v10 + v11 + v01);
                    if center <= 0.0 {
                        next_edge.insert(e_right, e_top);
                        next_edge.insert(e_left, e_bottom);
                    } else {
                        next_edge.insert(e_right, e_bottom);
                        next_edge.insert(e_left, e_top);
                    }
                }
                11 => { next_edge.insert(e_right, e_bottom); }
                12 => { next_edge.insert(e_left, e_right); }
                13 => { next_edge.insert(e_top, e_right); }
                14 => { next_edge.insert(e_left, e_top); }
                _ => {}
            }
        }
    }

    // 5. Connect edges into closed loops
    let mut visited: HashSet<usize> = HashSet::new();
    let mut start_keys: Vec<usize> = next_edge.keys().copied().collect();
    start_keys.sort_unstable();

    let mut result_polylines = Vec::new();
    for start_e in start_keys {
        if visited.contains(&start_e) {
            continue;
        }

        let mut loop_pts = Vec::new();
        let mut curr = start_e;
        while !visited.contains(&curr) {
            visited.insert(curr);
            if let Some(&pt) = edge_pos.get(&curr) {
                loop_pts.push(pt);
            }
            if let Some(&nxt) = next_edge.get(&curr) {
                curr = nxt;
            } else {
                break;
            }
        }

        if loop_pts.len() >= 4 {
            // Laplacian fairing pass
            let mut smoothed = loop_pts;
            for _ in 0..2 {
                let m = smoothed.len();
                let mut next_pass = Vec::with_capacity(m);
                for i in 0..m {
                    let prev = smoothed[(i + m - 1) % m];
                    let cur = smoothed[i];
                    let nxt = smoothed[(i + 1) % m];
                    next_pass.push(Point2D::new(
                        0.25 * prev.x + 0.50 * cur.x + 0.25 * nxt.x,
                        0.25 * prev.y + 0.50 * cur.y + 0.25 * nxt.y,
                    ));
                }
                smoothed = next_pass;
            }

            // Collinear simplification
            let simplified = simplify_contour_polyline(&smoothed, options.simplify_tolerance_mm);
            if simplified.len() >= 3 {
                let poly = Polyline::new(simplified, true);
                let bounds = poly.bounds();
                let area = bounds.width() * bounds.height();
                if area >= options.min_area_mm2 {
                    result_polylines.push(poly);
                }
            }
        }
    }

    result_polylines
}

fn simplify_contour_polyline(pts: &[Point2D], tol: f64) -> Vec<Point2D> {
    if pts.len() <= 4 {
        return pts.to_vec();
    }
    let mut out = Vec::with_capacity(pts.len());
    out.push(pts[0]);
    for i in 1..pts.len() {
        let prev = *out.last().unwrap();
        let curr = pts[i];
        let next = pts[(i + 1) % pts.len()];
        let d = point_to_segment_dist(&curr, &prev, &next);
        if d > tol || prev.distance_to(&curr) > tol * 10.0 {
            out.push(curr);
        }
    }
    out
}

fn point_to_segment_dist(p: &Point2D, a: &Point2D, b: &Point2D) -> f64 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let apx = p.x - a.x;
    let apy = p.y - a.y;
    let len_sq = abx * abx + aby * aby;
    if len_sq < 1e-9 {
        return (apx * apx + apy * apy).sqrt();
    }
    let t = ((apx * abx + apy * aby) / len_sq).clamp(0.0, 1.0);
    let proj_x = a.x + t * abx;
    let proj_y = a.y + t * aby;
    let dx = p.x - proj_x;
    let dy = p.y - proj_y;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_image_square() {
        let w = 40;
        let h = 40;
        let mut pixels = vec![255u8; w * h]; // White background

        // Draw a solid black 20x20 square in the center
        for y in 10..30 {
            for x in 10..30 {
                pixels[y * w + x] = 0; // Black
            }
        }

        let opts = TraceOptions {
            threshold: 128,
            invert: false,
            blur_radius: 0,
            min_area_mm2: 1.0,
            simplify_tolerance_mm: 0.1,
        };

        let polylines = trace_image_to_polylines(
            &pixels,
            w,
            h,
            1,
            0.0,
            0.0,
            40.0,
            40.0,
            &opts,
        );

        assert_eq!(polylines.len(), 1, "Should detect exactly one outer contour for solid square");
        let b = polylines[0].bounds();
        // The square spans x ~ [10, 30] and y ~ [10, 30]
        assert!((b.min_x - 10.0).abs() < 1.0);
        assert!((b.max_x - 30.0).abs() < 1.0);
        assert!((b.min_y - 10.0).abs() < 1.0);
        assert!((b.max_y - 30.0).abs() < 1.0);
    }
}
