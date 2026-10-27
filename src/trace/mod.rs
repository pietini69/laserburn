//! Bitmap to Vector (Image Trace) engine for LaserBurn.
//! Converts raster images into clean cutting/engraving vector polylines using
//! adaptive thresholding, multi-scale smoothing, and sub-pixel Marching Squares contour extraction.

use crate::geometry::{Point2D, Polyline};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TraceMode {
    /// Outer and inner boundary contours using Marching Squares (closed loops)
    Outline,
    /// Centerline single strokes using Zhang-Suen morphological skeletonization (open/closed strokes)
    Centerline,
}

impl Default for TraceMode {
    fn default() -> Self {
        Self::Outline
    }
}

impl TraceMode {
    pub const ALL: &'static [TraceMode] = &[TraceMode::Outline, TraceMode::Centerline];

    pub fn label(&self) -> &'static str {
        match self {
            TraceMode::Outline => "Outline (Ääriviivat / Contours)",
            TraceMode::Centerline => "Centerline (Keskiviivat / Single Stroke)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceOptions {
    /// Vectorization tracing mode (Outline vs Centerline)
    pub mode: TraceMode,
    /// Grayscale threshold (0..=255). Default is 128.
    pub threshold: u8,
    /// Invert image (if true, bright pixels become vectors; if false, dark pixels become vectors).
    pub invert: bool,
    /// Box blur radius (0..=5) to filter out noise, dither specks, or JPEG artifacts.
    pub blur_radius: usize,
    /// Minimum contour bounding-box area in mm² (or stroke length) to ignore small dust/speckles.
    pub min_area_mm2: f64,
    /// Collinear vertex simplification tolerance in mm.
    pub simplify_tolerance_mm: f64,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            mode: TraceMode::Outline,
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

    // Dispatch to Centerline tracing if selected
    if options.mode == TraceMode::Centerline {
        return trace_image_centerline(
            &gray,
            width,
            height,
            origin_x,
            origin_y,
            width_mm,
            height_mm,
            options,
        );
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

/// Traces a raster image into single-stroke centerlines using Zhang-Suen morphological thinning.
fn trace_image_centerline(
    gray: &[f32],
    width: usize,
    height: usize,
    origin_x: f64,
    origin_y: f64,
    width_mm: f64,
    height_mm: f64,
    options: &TraceOptions,
) -> Vec<Polyline> {
    let total_pixels = width * height;
    let mut grid = vec![0u8; total_pixels];
    let thresh = options.threshold as f32;

    for (i, &val) in gray.iter().enumerate() {
        let is_fg = if options.invert {
            val >= thresh
        } else {
            val <= thresh
        };
        if is_fg {
            grid[i] = 1;
        }
    }

    // Zhang-Suen morphological thinning
    let mut changed = true;
    let mut iter = 0;
    let max_iters = 300;
    let mut to_delete = Vec::new();

    while changed && iter < max_iters {
        changed = false;
        iter += 1;

        // Sub-iteration 1
        to_delete.clear();
        for y in 1..(height - 1) {
            let row = y * width;
            for x in 1..(width - 1) {
                let p1 = grid[row + x];
                if p1 == 0 {
                    continue;
                }

                let p2 = grid[(y - 1) * width + x];
                let p3 = grid[(y - 1) * width + (x + 1)];
                let p4 = grid[row + (x + 1)];
                let p5 = grid[(y + 1) * width + (x + 1)];
                let p6 = grid[(y + 1) * width + x];
                let p7 = grid[(y + 1) * width + (x - 1)];
                let p8 = grid[row + (x - 1)];
                let p9 = grid[(y - 1) * width + (x - 1)];

                let b = p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9;
                if b < 2 || b > 6 {
                    continue;
                }

                let mut a = 0;
                if p2 == 0 && p3 == 1 { a += 1; }
                if p3 == 0 && p4 == 1 { a += 1; }
                if p4 == 0 && p5 == 1 { a += 1; }
                if p5 == 0 && p6 == 1 { a += 1; }
                if p6 == 0 && p7 == 1 { a += 1; }
                if p7 == 0 && p8 == 1 { a += 1; }
                if p8 == 0 && p9 == 1 { a += 1; }
                if p9 == 0 && p2 == 1 { a += 1; }
                if a != 1 {
                    continue;
                }

                if p2 * p4 * p6 == 0 && p4 * p6 * p8 == 0 {
                    to_delete.push(row + x);
                }
            }
        }
        if !to_delete.is_empty() {
            for &idx in &to_delete {
                grid[idx] = 0;
            }
            changed = true;
        }

        // Sub-iteration 2
        to_delete.clear();
        for y in 1..(height - 1) {
            let row = y * width;
            for x in 1..(width - 1) {
                let p1 = grid[row + x];
                if p1 == 0 {
                    continue;
                }

                let p2 = grid[(y - 1) * width + x];
                let p3 = grid[(y - 1) * width + (x + 1)];
                let p4 = grid[row + (x + 1)];
                let p5 = grid[(y + 1) * width + (x + 1)];
                let p6 = grid[(y + 1) * width + x];
                let p7 = grid[(y + 1) * width + (x - 1)];
                let p8 = grid[row + (x - 1)];
                let p9 = grid[(y - 1) * width + (x - 1)];

                let b = p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9;
                if b < 2 || b > 6 {
                    continue;
                }

                let mut a = 0;
                if p2 == 0 && p3 == 1 { a += 1; }
                if p3 == 0 && p4 == 1 { a += 1; }
                if p4 == 0 && p5 == 1 { a += 1; }
                if p5 == 0 && p6 == 1 { a += 1; }
                if p6 == 0 && p7 == 1 { a += 1; }
                if p7 == 0 && p8 == 1 { a += 1; }
                if p8 == 0 && p9 == 1 { a += 1; }
                if p9 == 0 && p2 == 1 { a += 1; }
                if a != 1 {
                    continue;
                }

                if p2 * p4 * p8 == 0 && p2 * p6 * p8 == 0 {
                    to_delete.push(row + x);
                }
            }
        }
        if !to_delete.is_empty() {
            for &idx in &to_delete {
                grid[idx] = 0;
            }
            changed = true;
        }
    }

    // Build 8-connected adjacency graph
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    let offsets: [(isize, isize); 8] = [
        (-1, -1), (0, -1), (1, -1),
        (-1,  0),          (1,  0),
        (-1,  1), (0,  1), (1,  1),
    ];

    for y in 0..height {
        let y_isize = y as isize;
        for x in 0..width {
            let idx = y * width + x;
            if grid[idx] == 0 {
                continue;
            }

            let mut neighbors = Vec::new();
            for &(dx, dy) in &offsets {
                let nx = x as isize + dx;
                let ny = y_isize + dy;
                if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                    let n_idx = (ny as usize) * width + (nx as usize);
                    if grid[n_idx] == 1 {
                        neighbors.push(n_idx);
                    }
                }
            }
            adj.insert(idx, neighbors);
        }
    }

    let cell_x = width_mm / ((width - 1).max(1) as f64);
    let cell_y = height_mm / ((height - 1).max(1) as f64);

    let idx_to_point = |idx: usize| -> Point2D {
        let x = idx % width;
        let y = idx / width;
        Point2D::new(
            origin_x + (x as f64) * cell_x,
            origin_y + (y as f64) * cell_y,
        )
    };

    let mut visited_edges: HashSet<(usize, usize)> = HashSet::new();
    let mut polylines = Vec::new();

    #[inline]
    fn edge_key(u: usize, v: usize) -> (usize, usize) {
        if u < v { (u, v) } else { (v, u) }
    }

    // Phase 1: Walk from endpoints (degree 1) or junctions (degree != 2)
    let mut start_nodes: Vec<usize> = adj.keys().copied().collect();
    start_nodes.sort_by_key(|&u| {
        let deg = adj.get(&u).map(|v| v.len()).unwrap_or(0);
        if deg == 1 {
            0
        } else if deg > 2 {
            1
        } else {
            2
        }
    });

    for u in start_nodes {
        let neighbors = match adj.get(&u) {
            Some(n) => n.clone(),
            None => continue,
        };

        if neighbors.is_empty() {
            // Isolated single pixel (e.g. dot over an 'i' or period in handwriting)
            let pt = idx_to_point(u);
            let dot_poly = Polyline::new(vec![pt, Point2D::new(pt.x + cell_x * 0.5, pt.y)], false);
            polylines.push(dot_poly);
            continue;
        }

        let deg = neighbors.len();
        if deg == 2 {
            continue;
        }

        for &first_v in &neighbors {
            if visited_edges.contains(&edge_key(u, first_v)) {
                continue;
            }

            let mut stroke = vec![u, first_v];
            visited_edges.insert(edge_key(u, first_v));

            let mut curr = first_v;
            let mut prev = u;

            loop {
                let curr_neighbors = match adj.get(&curr) {
                    Some(n) => n,
                    None => break,
                };
                let curr_deg = curr_neighbors.len();
                if curr_deg != 2 {
                    break;
                }

                let next_candidate = curr_neighbors.iter().find(|&&nxt| nxt != prev && !visited_edges.contains(&edge_key(curr, nxt)));
                if let Some(&nxt) = next_candidate {
                    visited_edges.insert(edge_key(curr, nxt));
                    stroke.push(nxt);
                    prev = curr;
                    curr = nxt;
                } else {
                    break;
                }
            }

            if stroke.len() >= 2 {
                let pts: Vec<Point2D> = stroke.iter().map(|&idx| idx_to_point(idx)).collect();
                let simplified = ramer_douglas_peucker(&pts, options.simplify_tolerance_mm);
                let is_closed = stroke.first() == stroke.last() && stroke.len() > 3;
                let poly = Polyline::new(simplified, is_closed);
                if poly.length() >= options.simplify_tolerance_mm.max(0.2) {
                    polylines.push(poly);
                }
            }
        }
    }

    // Phase 2: Trace remaining unvisited closed loops / cycles (all degree == 2)
    let all_nodes: Vec<usize> = adj.keys().copied().collect();
    for u in all_nodes {
        let neighbors = match adj.get(&u) {
            Some(n) => n.clone(),
            None => continue,
        };

        for &first_v in &neighbors {
            if visited_edges.contains(&edge_key(u, first_v)) {
                continue;
            }

            let mut stroke = vec![u, first_v];
            visited_edges.insert(edge_key(u, first_v));

            let mut curr = first_v;
            let mut prev = u;

            loop {
                let curr_neighbors = match adj.get(&curr) {
                    Some(n) => n,
                    None => break,
                };

                let next_candidate = curr_neighbors.iter().find(|&&nxt| nxt != prev && !visited_edges.contains(&edge_key(curr, nxt)));
                if let Some(&nxt) = next_candidate {
                    visited_edges.insert(edge_key(curr, nxt));
                    stroke.push(nxt);
                    if nxt == u {
                        break;
                    }
                    prev = curr;
                    curr = nxt;
                } else {
                    break;
                }
            }

            if stroke.len() >= 2 {
                let pts: Vec<Point2D> = stroke.iter().map(|&idx| idx_to_point(idx)).collect();
                let simplified = ramer_douglas_peucker(&pts, options.simplify_tolerance_mm);
                let is_closed = stroke.first() == stroke.last() && stroke.len() > 3;
                let poly = Polyline::new(simplified, is_closed);
                if poly.length() >= options.simplify_tolerance_mm.max(0.2) {
                    polylines.push(poly);
                }
            }
        }
    }

    polylines
}

fn ramer_douglas_peucker(pts: &[Point2D], tol: f64) -> Vec<Point2D> {
    if pts.len() <= 2 {
        return pts.to_vec();
    }

    let mut max_dist = 0.0;
    let mut index = 0;
    let first = pts[0];
    let last = *pts.last().unwrap();

    for i in 1..(pts.len() - 1) {
        let d = point_to_segment_dist(&pts[i], &first, &last);
        if d > max_dist {
            max_dist = d;
            index = i;
        }
    }

    if max_dist > tol {
        let mut rec1 = ramer_douglas_peucker(&pts[..=index], tol);
        let rec2 = ramer_douglas_peucker(&pts[index..], tol);
        rec1.pop();
        rec1.extend(rec2);
        rec1
    } else {
        vec![first, last]
    }
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
            mode: TraceMode::Outline,
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

    #[test]
    fn test_trace_centerline_stroke() {
        let w = 50;
        let h = 50;
        let mut pixels = vec![255u8; w * h]; // White background

        // Draw a thick horizontal line (5 pixels tall, spanning x=10..40 at y=23..27)
        for y in 23..=27 {
            for x in 10..=40 {
                pixels[y * w + x] = 0; // Black
            }
        }

        let opts = TraceOptions {
            mode: TraceMode::Centerline,
            threshold: 128,
            invert: false,
            blur_radius: 0,
            min_area_mm2: 0.1,
            simplify_tolerance_mm: 0.1,
        };

        let polylines = trace_image_to_polylines(
            &pixels,
            w,
            h,
            1,
            0.0,
            0.0,
            50.0,
            50.0,
            &opts,
        );

        assert_eq!(polylines.len(), 1, "Centerline should extract exactly one single-stroke polyline");
        let poly = &polylines[0];
        println!("Extracted poly points: {:?}", poly.points);
        let b = poly.bounds();
        println!("Bounds: min_x={}, max_x={}, min_y={}, max_y={}", b.min_x, b.max_x, b.min_y, b.max_y);
        assert!(!poly.is_closed, "Line stroke must be open");
        let b = poly.bounds();
        // The centerline is located at y ~ 25.5 mm and spans x ~ [12.2, 37.8] mm (symmetrically trimmed by 2px end-cap)
        assert!((b.min_x - 12.2).abs() <= 1.0);
        assert!((b.max_x - 37.8).abs() <= 1.0);
        assert!((b.min_y - 25.5).abs() <= 1.0);
        assert!((b.max_y - 25.5).abs() <= 1.0);
    }
}
