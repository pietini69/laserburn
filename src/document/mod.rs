use crate::geometry::{BoundingBox, Point2D, Polyline};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CutMode {
    Line,       // Vector cut or score along the path
    Fill,       // Raster hatch / pocketing inside closed shapes
    OffsetFill, // Concentric inward/outward offset fill
    Image,      // Raster photographic engraving with dithering
}

impl Default for CutMode {
    fn default() -> Self {
        CutMode::Line
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DitherMode {
    Threshold,
    FloydSteinberg,
    Atkinson,
    Jarvis,
    Stucki,
    OrderedBayer,
    Halftone,
    GrayscalePower,
}

impl Default for DitherMode {
    fn default() -> Self {
        DitherMode::Atkinson
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GalvoTiming {
    pub laser_on_tc: i32,    // microseconds delay when turning laser on
    pub laser_off_tc: i32,   // microseconds delay when turning laser off
    pub end_tc: i32,         // microseconds delay at the end of cut vectors
    pub polygon_tc: i32,     // microseconds corner delay
    pub jump_speed: f64,     // rapid galvo travel speed mm/s
    pub jump_delay: i32,     // microseconds delay after rapid jump
    pub wobble_enable: bool, // galvo beam wobble for wide kerf or welding
    pub wobble_diameter: f64,// mm wobble circle diameter
    pub wobble_step: f64,    // mm wobble pitch
}

impl Default for GalvoTiming {
    fn default() -> Self {
        Self {
            laser_on_tc: 100,
            laser_off_tc: 100,
            end_tc: 50,
            polygon_tc: 50,
            jump_speed: 3000.0,
            jump_delay: 200,
            wobble_enable: false,
            wobble_diameter: 0.5,
            wobble_step: 0.05,
        }
    }
}

fn default_overscan() -> f64 {
    2.5
}

fn default_gamma() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CeramicColorBand {
    pub max_power_pct: f64, // Power threshold % up to which this layer is visible (0..=100)
    pub color_hex: String,  // Revealed paint/tile color in "#RRGGBB"
    pub label: String,      // Layer description, e.g. "Top Coat (Black)"
}

pub fn default_ceramic_bands() -> Vec<CeramicColorBand> {
    vec![
        CeramicColorBand {
            max_power_pct: 18.0,
            color_hex: "#1C1C1C".to_string(),
            label: "1. Top Coat (Black)".to_string(),
        },
        CeramicColorBand {
            max_power_pct: 45.0,
            color_hex: "#1E64C8".to_string(),
            label: "2. Mid Coat (Blue)".to_string(),
        },
        CeramicColorBand {
            max_power_pct: 75.0,
            color_hex: "#E6A817".to_string(),
            label: "3. Base Coat (Yellow/Gold)".to_string(),
        },
        CeramicColorBand {
            max_power_pct: 100.0,
            color_hex: "#F5F5F0".to_string(),
            label: "4. Ceramic Tile (White)".to_string(),
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CutSetting {
    pub name: String,
    pub mode: CutMode,
    pub speed: f64,             // mm/s
    pub power_min: f64,         // 0.0 .. 100.0 %
    pub power_max: f64,         // 0.0 .. 100.0 %
    pub num_passes: u32,        // number of repeated cut cycles
    pub z_offset: f64,          // mm focus shift
    pub z_step_per_pass: f64,   // mm focus drop per pass
    pub air_assist: bool,       // air assist valve on/off
    pub kerf: f64,              // beam kerf offset in mm
    pub overcut: f64,           // overlap past start point in mm
    pub line_interval: f64,     // scanline spacing for Fill mode in mm (e.g. 0.08mm = 318 DPI)
    pub scan_angle: f64,        // raster angle in degrees (0.0 = horizontal)
    pub cross_hatch: bool,      // enable perpendicular second pass for Fill
    pub dither_mode: DitherMode,// dithering algorithm for Image mode
    pub galvo: GalvoTiming,     // galvo / fiber timing parameters
    #[serde(default)]
    pub invert_image: bool,     // invert light/dark in image engraving (negative)
    #[serde(default = "default_overscan")]
    pub overscan: f64,          // overscan acceleration/deceleration margin in mm (e.g. 2.5mm)
    #[serde(default)]
    pub ceramic_preview: bool,  // simulate multi-layer ceramic paint colors on canvas
    #[serde(default = "default_ceramic_bands")]
    pub ceramic_bands: Vec<CeramicColorBand>,
}

impl Default for CutSetting {
    fn default() -> Self {
        Self {
            name: "Default Cut".to_string(),
            mode: CutMode::Line,
            speed: 50.0,
            power_min: 20.0,
            power_max: 80.0,
            num_passes: 1,
            z_offset: 0.0,
            z_step_per_pass: 0.0,
            air_assist: true,
            kerf: 0.0,
            overcut: 0.0,
            line_interval: 0.08,
            scan_angle: 0.0,
            cross_hatch: false,
            dither_mode: DitherMode::Atkinson,
            galvo: GalvoTiming::default(),
            invert_image: false,
            overscan: 2.5,
            ceramic_preview: false,
            ceramic_bands: default_ceramic_bands(),
        }
    }
}

fn default_output() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub index: usize,
    pub name: String,
    pub color_hex: String, // e.g. "#0000FF", "#FF0000"
    pub priority: i32,     // lower numbers execute first
    pub visible: bool,
    pub locked: bool,
    #[serde(default = "default_output")]
    pub output: bool,      // whether this layer is cut / sent to laser
    pub cut_setting: CutSetting,
}

impl Layer {
    pub fn new(index: usize, name: &str, color_hex: &str) -> Self {
        let is_tool = index >= 30 || name.starts_with('T');
        Self {
            index,
            name: name.to_string(),
            color_hex: color_hex.to_string(),
            priority: index as i32,
            visible: true,
            locked: false,
            output: !is_tool,
            cut_setting: CutSetting::default(),
        }
    }

    pub fn is_tool(&self) -> bool {
        self.index >= 30 || self.name.starts_with('T')
    }

    pub fn display_tag(&self) -> String {
        if self.is_tool() {
            let t_num = if self.index >= 30 { self.index - 29 } else { self.index };
            format!("[T{:02}]", t_num)
        } else {
            format!("[L{:02}]", self.index)
        }
    }

    pub fn display_label(&self) -> String {
        let tag = self.display_tag();
        let raw = tag.trim_matches(|c| c == '[' || c == ']');
        if self.name.is_empty() || self.name == tag || self.name == raw {
            tag
        } else {
            format!("{} {}", tag, self.name)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape {
    Path {
        layer_index: usize,
        polyline: Polyline,
    },
    Rect {
        layer_index: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        corner_radius: f64,
    },
    Circle {
        layer_index: usize,
        center: Point2D,
        radius: f64,
    },
    Group {
        layer_index: usize,
        shapes: Vec<Shape>,
        locked: bool,
    },
    Image {
        layer_index: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        image_path: Option<String>,
        #[serde(skip)]
        raw_pixels: Option<Vec<u8>>, // Grayscale or RGBA bytes
        raw_width: u32,
        raw_height: u32,
        #[serde(default)]
        brightness: f32,
        #[serde(default)]
        contrast: f32,
        #[serde(default = "default_gamma")]
        gamma: f32,
        #[serde(default)]
        sharpen: f32,
    },
}

impl Shape {
    pub fn layer_index(&self) -> usize {
        match self {
            Shape::Path { layer_index, .. } => *layer_index,
            Shape::Rect { layer_index, .. } => *layer_index,
            Shape::Circle { layer_index, .. } => *layer_index,
            Shape::Group { layer_index, .. } => *layer_index,
            Shape::Image { layer_index, .. } => *layer_index,
        }
    }

    pub fn set_layer_index(&mut self, idx: usize) {
        match self {
            Shape::Path { layer_index, .. } => *layer_index = idx,
            Shape::Rect { layer_index, .. } => *layer_index = idx,
            Shape::Circle { layer_index, .. } => *layer_index = idx,
            Shape::Group { layer_index, shapes, .. } => {
                *layer_index = idx;
                for s in shapes {
                    s.set_layer_index(idx);
                }
            }
            Shape::Image { layer_index, .. } => *layer_index = idx,
        }
    }

    pub fn is_locked(&self) -> bool {
        match self {
            Shape::Group { locked, .. } => *locked,
            _ => false,
        }
    }

    pub fn set_locked(&mut self, is_locked: bool) {
        if let Shape::Group { locked, .. } = self {
            *locked = is_locked;
        }
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        match self {
            Shape::Path { polyline, .. } => polyline.translate(dx, dy),
            Shape::Rect { x, y, .. } => {
                *x += dx;
                *y += dy;
            }
            Shape::Circle { center, .. } => {
                center.x += dx;
                center.y += dy;
            }
            Shape::Group { shapes, locked, .. } => {
                if !*locked {
                    for s in shapes {
                        s.translate(dx, dy);
                    }
                }
            }
            Shape::Image { x, y, .. } => {
                *x += dx;
                *y += dy;
            }
        }
    }

    pub fn scale(&mut self, origin: Point2D, sx: f64, sy: f64) {
        if sx.abs() < 1e-6 || sy.abs() < 1e-6 {
            return;
        }
        match self {
            Shape::Path { polyline, .. } => polyline.scale(origin, sx, sy),
            Shape::Rect { x, y, width, height, corner_radius, .. } => {
                let new_x1 = origin.x + (*x - origin.x) * sx;
                let new_x2 = origin.x + (*x + *width - origin.x) * sx;
                let new_y1 = origin.y + (*y - origin.y) * sy;
                let new_y2 = origin.y + (*y + *height - origin.y) * sy;
                *x = new_x1.min(new_x2);
                *y = new_y1.min(new_y2);
                *width = (new_x2 - new_x1).abs();
                *height = (new_y2 - new_y1).abs();
                *corner_radius *= sx.abs().min(sy.abs());
            }
            Shape::Circle { center, radius, layer_index } => {
                if (sx.abs() - sy.abs()).abs() < 1e-4 {
                    center.x = origin.x + (center.x - origin.x) * sx;
                    center.y = origin.y + (center.y - origin.y) * sy;
                    *radius *= sx.abs();
                } else {
                    // Non-uniform scaling: convert Circle into a smooth Ellipse / Oval Polyline Path!
                    let segments = 48;
                    let mut pts = Vec::with_capacity(segments);
                    for i in 0..segments {
                        let a = (i as f64 / segments as f64) * std::f64::consts::TAU;
                        let px = center.x + *radius * a.cos();
                        let py = center.y + *radius * a.sin();
                        let scaled_x = origin.x + (px - origin.x) * sx;
                        let scaled_y = origin.y + (py - origin.y) * sy;
                        pts.push(Point2D::new(scaled_x, scaled_y));
                    }
                    *self = Shape::Path {
                        layer_index: *layer_index,
                        polyline: Polyline::new(pts, true),
                    };
                }
            }
            Shape::Group { shapes, locked, .. } => {
                if !*locked {
                    for s in shapes {
                        s.scale(origin, sx, sy);
                    }
                }
            }
            Shape::Image { x, y, width, height, .. } => {
                let new_x1 = origin.x + (*x - origin.x) * sx;
                let new_x2 = origin.x + (*x + *width - origin.x) * sx;
                let new_y1 = origin.y + (*y - origin.y) * sy;
                let new_y2 = origin.y + (*y + *height - origin.y) * sy;
                *x = new_x1.min(new_x2);
                *y = new_y1.min(new_y2);
                *width = (new_x2 - new_x1).abs();
                *height = (new_y2 - new_y1).abs();
            }
        }
    }

    pub fn rotate(&mut self, origin: Point2D, angle_rad: f64) {
        match self {
            Shape::Path { polyline, .. } => polyline.rotate(origin, angle_rad),
            Shape::Rect { x, y, width, height, layer_index, .. } => {
                let mut poly = Polyline::new(
                    vec![
                        Point2D::new(*x, *y),
                        Point2D::new(*x + *width, *y),
                        Point2D::new(*x + *width, *y + *height),
                        Point2D::new(*x, *y + *height),
                    ],
                    true,
                );
                poly.rotate(origin, angle_rad);
                *self = Shape::Path {
                    layer_index: *layer_index,
                    polyline: poly,
                };
            }
            Shape::Circle { center, .. } => {
                let (sin_a, cos_a) = angle_rad.sin_cos();
                let dx = center.x - origin.x;
                let dy = center.y - origin.y;
                center.x = origin.x + dx * cos_a - dy * sin_a;
                center.y = origin.y + dx * sin_a + dy * cos_a;
            }
            Shape::Group { shapes, locked, .. } => {
                if !*locked {
                    for s in shapes {
                        s.rotate(origin, angle_rad);
                    }
                }
            }
            Shape::Image { .. } => {}
        }
    }

    pub fn set_position(&mut self, new_x: f64, new_y: f64) {
        let b = self.bounds();
        let dx = new_x - b.min_x;
        let dy = new_y - b.min_y;
        self.translate(dx, dy);
    }

    pub fn set_size(&mut self, new_w: f64, new_h: f64) {
        let b = self.bounds();
        let cur_w = b.width();
        let cur_h = b.height();
        if cur_w > 1e-4 && cur_h > 1e-4 && new_w > 1e-4 && new_h > 1e-4 {
            let sx = new_w / cur_w;
            let sy = new_h / cur_h;
            self.scale(Point2D::new(b.min_x, b.min_y), sx, sy);
        }
    }

    pub fn hit_test(&self, pt: &Point2D, tolerance: f64) -> bool {
        match self {
            Shape::Path { polyline, .. } => {
                if polyline.is_closed && polyline.contains_point(pt) {
                    true
                } else {
                    polyline.distance_to_point(pt) <= tolerance
                }
            }
            Shape::Rect { x, y, width, height, .. } => {
                pt.x >= x - tolerance && pt.x <= x + width + tolerance &&
                pt.y >= y - tolerance && pt.y <= y + height + tolerance
            }
            Shape::Circle { center, radius, .. } => {
                center.distance_to(pt) <= radius + tolerance
            }
            Shape::Group { shapes, .. } => {
                shapes.iter().any(|s| s.hit_test(pt, tolerance)) || self.bounds().contains_point(pt)
            }
            Shape::Image { x, y, width, height, .. } => {
                pt.x >= x - tolerance && pt.x <= x + width + tolerance &&
                pt.y >= y - tolerance && pt.y <= y + height + tolerance
            }
        }
    }

    pub fn to_polylines(&self) -> Vec<Polyline> {
        match self {
            Shape::Path { polyline, .. } => vec![polyline.clone()],
            Shape::Group { shapes, .. } => shapes.iter().flat_map(|s| s.to_polylines()).collect(),
            Shape::Rect {
                x,
                y,
                width,
                height,
                corner_radius,
                ..
            } => {
                if *corner_radius <= 1e-4 {
                    vec![Polyline::new(
                        vec![
                            Point2D::new(*x, *y),
                            Point2D::new(*x + *width, *y),
                            Point2D::new(*x + *width, *y + *height),
                            Point2D::new(*x, *y + *height),
                        ],
                        true,
                    )]
                } else {
                    let r = corner_radius.min(width * 0.5).min(height * 0.5);
                    let mut pts = Vec::new();
                    let steps = 8;
                    // Top-right
                    for i in 0..=steps {
                        let a = (i as f64 / steps as f64) * std::f64::consts::FRAC_PI_2;
                        pts.push(Point2D::new(x + width - r + r * a.sin(), y + r - r * a.cos()));
                    }
                    // Bottom-right
                    for i in 0..=steps {
                        let a = (i as f64 / steps as f64) * std::f64::consts::FRAC_PI_2;
                        pts.push(Point2D::new(x + width - r + r * a.cos(), y + height - r + r * a.sin()));
                    }
                    // Bottom-left
                    for i in 0..=steps {
                        let a = (i as f64 / steps as f64) * std::f64::consts::FRAC_PI_2;
                        pts.push(Point2D::new(x + r - r * a.sin(), y + height - r + r * a.cos()));
                    }
                    // Top-left
                    for i in 0..=steps {
                        let a = (i as f64 / steps as f64) * std::f64::consts::FRAC_PI_2;
                        pts.push(Point2D::new(x + r - r * a.cos(), y + r - r * a.sin()));
                    }
                    vec![Polyline::new(pts, true)]
                }
            }
            Shape::Circle { center, radius, .. } => {
                let segments = 36;
                let mut pts = Vec::with_capacity(segments);
                for i in 0..segments {
                    let a = (i as f64 / segments as f64) * std::f64::consts::TAU;
                    pts.push(Point2D::new(
                        center.x + radius * a.cos(),
                        center.y + radius * a.sin(),
                    ));
                }
                vec![Polyline::new(pts, true)]
            }
            Shape::Image { x, y, width, height, .. } => {
                vec![Polyline::new(
                    vec![
                        Point2D::new(*x, *y),
                        Point2D::new(*x + *width, *y),
                        Point2D::new(*x + *width, *y + *height),
                        Point2D::new(*x, *y + *height),
                    ],
                    true,
                )]
            }
        }
    }

    /// Generates high-efficiency serpentine horizontal raster scanlines for photographic image engraving
    pub fn to_raster_polylines(&self, dither_mode: DitherMode, line_interval_mm: f64) -> Vec<Polyline> {
        match self {
            Shape::Image {
                x,
                y,
                width,
                height,
                raw_pixels,
                raw_width,
                raw_height,
                image_path,
                brightness,
                contrast,
                gamma,
                sharpen,
                ..
            } => {
                if *width <= 0.0 || *height <= 0.0 {
                    return vec![];
                }

                // Retrieve or load RGBA pixels
                let (pixels, w_orig, h_orig) = if let (Some(px), w, h) = (raw_pixels, *raw_width, *raw_height) {
                    if w > 0 && h > 0 {
                        (px.clone(), w as usize, h as usize)
                    } else {
                        return vec![];
                    }
                } else if let Some(path) = image_path {
                    if let Ok(dyn_img) = image::open(path) {
                        let rgba = dyn_img.to_rgba8();
                        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                        (rgba.into_raw(), w, h)
                    } else {
                        return vec![];
                    }
                } else {
                    return vec![];
                };

                // Convert to grayscale & apply Brightness/Contrast/Gamma/Sharpen
                let gray_raw = crate::dither::to_grayscale(&pixels, w_orig, h_orig, 4);
                let gray = crate::dither::preprocess_grayscale(
                    &gray_raw,
                    w_orig,
                    h_orig,
                    *brightness,
                    *contrast,
                    *gamma,
                    *sharpen,
                );

                // Calculate target raster resolution based on line_interval_mm
                let interval = if line_interval_mm > 0.005 { line_interval_mm } else { 0.08 };
                let target_h = ((*height / interval).round() as usize).clamp(1, 2000);
                let target_w = ((*width / interval).round() as usize).clamp(1, 2000);

                let resampled = crate::dither::resample_grayscale(&gray, w_orig, h_orig, target_w, target_h);
                let mask = crate::dither::dither(&resampled, target_w, target_h, dither_mode);

                let dx = *width / target_w as f64;
                let dy = *height / target_h as f64;

                let mut polylines = Vec::new();

                // Serpentine (bi-directional) horizontal scanlines
                for py in 0..target_h {
                    let cur_y = *y + (py as f64 + 0.5) * dy;
                    let reverse = (py % 2) == 1;

                    let row_start = py * target_w;
                    let row_mask = &mask[row_start..row_start + target_w];

                    let mut in_run = false;
                    let mut run_start = 0;
                    let mut segments: Vec<(f64, f64)> = Vec::new();

                    for px in 0..target_w {
                        if row_mask[px] {
                            if !in_run {
                                in_run = true;
                                run_start = px;
                            }
                        } else if in_run {
                            in_run = false;
                            let x0 = *x + run_start as f64 * dx;
                            let x1 = *x + px as f64 * dx;
                            segments.push((x0, x1));
                        }
                    }
                    if in_run {
                        let x0 = *x + run_start as f64 * dx;
                        let x1 = *x + *width;
                        segments.push((x0, x1));
                    }

                    if reverse {
                        for (x0, x1) in segments.into_iter().rev() {
                            polylines.push(Polyline::new(
                                vec![Point2D::new(x1, cur_y), Point2D::new(x0, cur_y)],
                                false,
                            ));
                        }
                    } else {
                        for (x0, x1) in segments {
                            polylines.push(Polyline::new(
                                vec![Point2D::new(x0, cur_y), Point2D::new(x1, cur_y)],
                                false,
                            ));
                        }
                    }
                }

                polylines
            }
            Shape::Group { shapes, .. } => {
                shapes.iter().flat_map(|s| s.to_raster_polylines(dither_mode, line_interval_mm)).collect()
            }
            _ => self.to_polylines(),
        }
    }

    /// Generates raster toolpaths, returning a list of polylines and optional per-segment laser powers.
    /// In binary dither modes (Atkinson, Floyd-Steinberg, etc.), laser power is constant (`None`).
    /// In GrayscalePower mode, laser power is dynamically modulated for each segment (`Some(Vec<f64>)`)
    /// between `cut_setting.power_min` and `cut_setting.power_max`, with support for negative inversion
    /// and overscan acceleration/deceleration margins.
    pub fn to_raster_paths(&self, cut_setting: &CutSetting) -> Vec<(Polyline, Option<Vec<f64>>)> {
        match self {
            Shape::Image {
                x,
                y,
                width,
                height,
                raw_pixels,
                raw_width,
                raw_height,
                image_path,
                brightness,
                contrast,
                gamma,
                sharpen,
                ..
            } => {
                if *width <= 0.0 || *height <= 0.0 {
                    return vec![];
                }

                // Retrieve or load RGBA pixels
                let (pixels, w_orig, h_orig) = if let (Some(px), w, h) = (raw_pixels, *raw_width, *raw_height) {
                    if w > 0 && h > 0 {
                        (px.clone(), w as usize, h as usize)
                    } else {
                        return vec![];
                    }
                } else if let Some(path) = image_path {
                    if let Ok(dyn_img) = image::open(path) {
                        let rgba = dyn_img.to_rgba8();
                        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                        (rgba.into_raw(), w, h)
                    } else {
                        return vec![];
                    }
                } else {
                    return vec![];
                };

                let interval = if cut_setting.line_interval > 0.005 {
                    cut_setting.line_interval
                } else {
                    0.08
                };

                if cut_setting.dither_mode == DitherMode::GrayscalePower {
                    let gray_raw = crate::dither::to_grayscale(&pixels, w_orig, h_orig, 4);
                    let gray = crate::dither::preprocess_grayscale(
                        &gray_raw,
                        w_orig,
                        h_orig,
                        *brightness,
                        *contrast,
                        *gamma,
                        *sharpen,
                    );
                    let target_h = ((*height / interval).round() as usize).clamp(1, 4000);
                    let target_w = ((*width / interval).round() as usize).clamp(1, 4000);
                    let resampled = crate::dither::resample_grayscale(&gray, w_orig, h_orig, target_w, target_h);

                    let dx = *width / target_w as f64;
                    let dy = *height / target_h as f64;
                    let overscan = cut_setting.overscan.max(0.0);

                    let mut result_paths = Vec::new();
                    let p_min = cut_setting.power_min.clamp(0.0, 100.0);
                    let p_max = cut_setting.power_max.clamp(0.0, 100.0);

                    for py in 0..target_h {
                        let cur_y = *y + (py as f64 + 0.5) * dy;
                        let reverse = (py % 2) == 1;
                        let row_start = py * target_w;

                        // 1. Calculate per-pixel power percentage
                        let mut row_powers = Vec::with_capacity(target_w);
                        for px in 0..target_w {
                            let mut val = resampled[row_start + px];
                            if cut_setting.invert_image {
                                val = 255.0 - val;
                            }
                            // Values >= 254.0 are pure white / background -> laser off (0.0% power)
                            let p = if val >= 254.0 {
                                0.0
                            } else {
                                let darkness = ((254.0 - val) / 254.0).clamp(0.0, 1.0) as f64;
                                p_min + (p_max - p_min) * darkness
                            };
                            row_powers.push(p);
                        }

                        // Check if row has any burning pixels
                        let first_active = row_powers.iter().position(|&p| p > 0.0);
                        let last_active = row_powers.iter().rposition(|&p| p > 0.0);
                        let (first_px, last_px) = match (first_active, last_active) {
                            (Some(f), Some(l)) => (f, l),
                            _ => continue, // completely white row, skip
                        };

                        // 2. Break row into strokes if there are large whitespace gaps (> 5.0 mm)
                        let max_gap_px = (5.0 / dx).max(1.0).round() as usize;
                        let mut strokes: Vec<(usize, usize)> = Vec::new();
                        let mut stroke_start = first_px;
                        let mut blank_count = 0;

                        for px in first_px..=last_px {
                            if row_powers[px] <= 0.0 {
                                blank_count += 1;
                            } else {
                                if blank_count > max_gap_px {
                                    let prev_stroke_end = px - blank_count - 1;
                                    strokes.push((stroke_start, prev_stroke_end));
                                    stroke_start = px;
                                }
                                blank_count = 0;
                            }
                        }
                        strokes.push((stroke_start, last_px));

                        // 3. For each stroke, compress adjacent pixels with similar power into segments
                        let mut row_paths = Vec::new();
                        for (s_start, s_end) in strokes {
                            let stroke_x0 = *x + s_start as f64 * dx;
                            let stroke_x1 = *x + (s_end + 1) as f64 * dx;

                            let mut pts = Vec::new();
                            let mut powers = Vec::new();

                            pts.push(Point2D::new(stroke_x0, cur_y));

                            let mut seg_power = row_powers[s_start];

                            for px in s_start..=s_end {
                                let p_curr = row_powers[px];
                                let px_end_x = *x + (px + 1) as f64 * dx;

                                // If power changed significantly (> 0.25% power diff)
                                if (p_curr - seg_power).abs() > 0.25 {
                                    pts.push(Point2D::new(px_end_x - dx, cur_y));
                                    powers.push(seg_power);
                                    seg_power = p_curr;
                                }
                            }
                            pts.push(Point2D::new(stroke_x1, cur_y));
                            powers.push(seg_power);

                            // Apply overscan margin for smooth acceleration/deceleration without tile edge burns
                            if overscan > 0.0 {
                                let lead_in_x = (stroke_x0 - overscan).max(0.0);
                                let lead_out_x = stroke_x1 + overscan;
                                if (lead_in_x - stroke_x0).abs() > 1e-4 {
                                    pts.insert(0, Point2D::new(lead_in_x, cur_y));
                                    powers.insert(0, 0.0); // 0% power during acceleration
                                }
                                pts.push(Point2D::new(lead_out_x, cur_y));
                                powers.push(0.0); // 0% power during deceleration
                            }

                            if reverse {
                                pts.reverse();
                                powers.reverse();
                            }

                            row_paths.push((Polyline::new(pts, false), Some(powers)));
                        }

                        if reverse {
                            row_paths.reverse();
                        }
                        result_paths.extend(row_paths);
                    }
                    result_paths
                } else {
                    // Binary dither mode
                    self.to_raster_polylines(cut_setting.dither_mode, interval)
                        .into_iter()
                        .map(|p| (p, None))
                        .collect()
                }
            }
            Shape::Group { shapes, .. } => {
                shapes.iter().flat_map(|s| s.to_raster_paths(cut_setting)).collect()
            }
            _ => {
                self.to_polylines().into_iter().map(|p| (p, None)).collect()
            }
        }
    }

    /// Creates an outward (+distance) or inward (-distance) offset contour of this shape.
    pub fn offset_shape(&self, distance: f64, target_layer: Option<usize>) -> Vec<Shape> {
        let layer_index = target_layer.unwrap_or_else(|| self.layer_index());
        match self {
            Shape::Rect {
                x,
                y,
                width,
                height,
                corner_radius,
                ..
            } => {
                let nw = *width + 2.0 * distance;
                let nh = *height + 2.0 * distance;
                if nw > 0.1 && nh > 0.1 {
                    vec![Shape::Rect {
                        layer_index,
                        x: *x - distance,
                        y: *y - distance,
                        width: nw,
                        height: nh,
                        corner_radius: (*corner_radius + distance).max(0.0),
                    }]
                } else {
                    vec![]
                }
            }
            Shape::Circle { center, radius, .. } => {
                let nr = *radius + distance;
                if nr > 0.1 {
                    vec![Shape::Circle {
                        layer_index,
                        center: *center,
                        radius: nr,
                    }]
                } else {
                    vec![]
                }
            }
            Shape::Path { polyline, .. } => {
                let off_polys = crate::geometry::offset_polylines_sdf(std::slice::from_ref(polyline), distance);
                if off_polys.len() == 1 {
                    vec![Shape::Path {
                        layer_index,
                        polyline: off_polys.into_iter().next().unwrap(),
                    }]
                } else if off_polys.len() > 1 {
                    let shapes = off_polys
                        .into_iter()
                        .map(|p| Shape::Path {
                            layer_index,
                            polyline: p,
                        })
                        .collect();
                    vec![Shape::Group {
                        layer_index,
                        shapes,
                        locked: false,
                    }]
                } else {
                    vec![]
                }
            }
            Shape::Group { locked, .. } => {
                let polys = self.to_polylines();
                let off_polys = crate::geometry::offset_polylines_sdf(&polys, distance);
                let child_offsets: Vec<Shape> = off_polys
                    .into_iter()
                    .map(|p| Shape::Path {
                        layer_index,
                        polyline: p,
                    })
                    .collect();
                if child_offsets.is_empty() {
                    vec![]
                } else if child_offsets.len() == 1 {
                    child_offsets
                } else {
                    vec![Shape::Group {
                        layer_index,
                        shapes: child_offsets,
                        locked: *locked,
                    }]
                }
            }
            Shape::Image {
                x,
                y,
                width,
                height,
                ..
            } => {
                let nw = *width + 2.0 * distance;
                let nh = *height + 2.0 * distance;
                if nw > 0.1 && nh > 0.1 {
                    vec![Shape::Rect {
                        layer_index,
                        x: *x - distance,
                        y: *y - distance,
                        width: nw,
                        height: nh,
                        corner_radius: 0.0,
                    }]
                } else {
                    vec![]
                }
            }
        }
    }

    pub fn bounds(&self) -> BoundingBox {
        match self {
            Shape::Path { polyline, .. } => polyline.bounds(),
            Shape::Group { shapes, .. } => {
                let mut bb = BoundingBox::empty();
                for s in shapes {
                    bb = bb.union(&s.bounds());
                }
                bb
            }
            Shape::Rect {
                x,
                y,
                width,
                height,
                ..
            } => BoundingBox::new(*x, *y, *x + *width, *y + *height),
            Shape::Circle { center, radius, .. } => BoundingBox::new(
                center.x - radius,
                center.y - radius,
                center.x + radius,
                center.y + radius,
            ),
            Shape::Image {
                x,
                y,
                width,
                height,
                ..
            } => BoundingBox::new(*x, *y, *x + *width, *y + *height),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub name: String,
    pub width: f64,  // workspace bed width in mm
    pub height: f64, // workspace bed height in mm
    pub layers: BTreeMap<usize, Layer>,
    pub shapes: Vec<Shape>,
}

impl Document {
    pub fn default_layers_path() -> Option<std::path::PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;
        let dir = std::path::PathBuf::from(home).join(".config").join("laser_burn");
        let _ = std::fs::create_dir_all(&dir);
        Some(dir.join("default_layers.json"))
    }

    pub fn load_default_layers() -> Option<BTreeMap<usize, Layer>> {
        if cfg!(test) {
            return None;
        }
        let path = Self::default_layers_path()?;
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(layers) = serde_json::from_str::<BTreeMap<usize, Layer>>(&content) {
                    if !layers.is_empty() {
                        return Some(layers);
                    }
                }
            }
        }
        None
    }

    pub fn save_default_layers(layers: &BTreeMap<usize, Layer>) -> Result<(), String> {
        if cfg!(test) {
            return Ok(());
        }
        if let Some(path) = Self::default_layers_path() {
            let json = serde_json::to_string_pretty(layers).map_err(|e| e.to_string())?;
            std::fs::write(&path, json).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Could not find configuration directory".to_string())
        }
    }

    pub fn reset_default_layers() -> Result<(), String> {
        if cfg!(test) {
            return Ok(());
        }
        if let Some(path) = Self::default_layers_path() {
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }
        Ok(())
    }

    pub fn new(name: &str, width: f64, height: f64) -> Self {
        let layers = if let Some(saved) = Self::load_default_layers() {
            saved
        } else {
            let mut layers = BTreeMap::new();
            // Default color layers (L00 to L07) and Tool layers (T01, T02)
            let default_palette = [
                (0, "L00", "#000000"),
                (1, "L01", "#0000FF"),
                (2, "L02", "#FF0000"),
                (3, "L03", "#00AA00"),
                (4, "L04", "#D4AA00"),
                (5, "L05", "#00CCCC"),
                (6, "L06", "#CC00CC"),
                (7, "L07", "#FF8800"),
                (30, "T01", "#FF6600"),
                (31, "T02", "#00CCFF"),
            ];

            for (idx, name, hex) in default_palette {
                layers.insert(idx, Layer::new(idx, name, hex));
            }
            layers
        };

        Self {
            name: name.to_string(),
            width,
            height,
            layers,
            shapes: Vec::new(),
        }
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    pub fn default_layer_info(index: usize) -> (String, &'static str) {
        let palette = [
            "#000000", // C00 Black
            "#0000FF", // C01 Blue
            "#FF0000", // C02 Red
            "#00AA00", // C03 Green
            "#D4AA00", // C04 Yellow
            "#00CCCC", // C05 Cyan
            "#CC00CC", // C06 Magenta
            "#FF8800", // C07 Orange
            "#808080", // C08 Gray
            "#7A4A00", // C09 Brown
            "#000080", // C10 Dark Blue
            "#800000", // C11 Dark Red
            "#008000", // C12 Dark Green
            "#808000", // C13 Olive
            "#008080", // C14 Teal
            "#800080", // C15 Purple
            "#FFA07A", // C16 Salmon
            "#98FB98", // C17 Pale Green
            "#AFEEEE", // C18 Pale Turquoise
            "#D8BFD8", // C19 Thistle
            "#FFD700", // C20 Gold
            "#FF69B4", // C21 Hot Pink
            "#CD5C5C", // C22 Indian Red
            "#4B0082", // C23 Indigo
            "#2E8B57", // C24 Sea Green
            "#DAA520", // C25 Goldenrod
            "#D2691E", // C26 Chocolate
            "#5F9EA0", // C27 Cadet Blue
            "#1E90FF", // C28 Dodger Blue
            "#FF1493", // C29 Deep Pink
        ];
        if index == 30 {
            ("T01".to_string(), "#FF6600")
        } else if index == 31 {
            ("T02".to_string(), "#00CCFF")
        } else if index == 32 {
            ("T03".to_string(), "#00FF00")
        } else if index < palette.len() {
            (format!("L{:02}", index), palette[index])
        } else {
            (format!("L{:02}", index), "#888888")
        }
    }

    pub fn get_or_create_layer(&mut self, index: usize) -> &mut Layer {
        self.layers.entry(index).or_insert_with(|| {
            let (name, hex) = Self::default_layer_info(index);
            Layer::new(index, &name, hex)
        })
    }

    pub fn bounds(&self) -> BoundingBox {
        let mut bb = BoundingBox::empty();
        for shape in &self.shapes {
            bb = bb.union(&shape.bounds());
        }
        bb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_hit_test() {
        let circle = Shape::Circle {
            layer_index: 0,
            center: Point2D::new(50.0, 50.0),
            radius: 10.0,
        };
        // Inside circle
        assert!(circle.hit_test(&Point2D::new(50.0, 50.0), 1.0));
        assert!(circle.hit_test(&Point2D::new(55.0, 50.0), 1.0));
        assert!(circle.hit_test(&Point2D::new(60.0, 50.0), 1.0)); // on edge
        // Outside circle
        assert!(!circle.hit_test(&Point2D::new(65.0, 50.0), 1.0));

        let rect = Shape::Rect {
            layer_index: 0,
            x: 20.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
            corner_radius: 0.0,
        };
        // Inside rect
        assert!(rect.hit_test(&Point2D::new(35.0, 40.0), 1.0));
        // Outside rect
        assert!(!rect.hit_test(&Point2D::new(60.0, 40.0), 1.0));
    }

    #[test]
    fn test_gear_hit_test() {
        let cx = 180.0;
        let cy = 180.0;
        let teeth = 12;
        let r_inner = 40.0;
        let r_outer = 50.0;
        let mut gear_pts = Vec::new();
        for i in 0..teeth {
            let a0 = (i as f64 / teeth as f64) * std::f64::consts::TAU;
            let a1 = a0 + (0.28 / teeth as f64) * std::f64::consts::TAU;
            let a2 = a0 + (0.50 / teeth as f64) * std::f64::consts::TAU;
            let a3 = a0 + (0.78 / teeth as f64) * std::f64::consts::TAU;

            gear_pts.push(Point2D::new(cx + r_inner * a0.cos(), cy + r_inner * a0.sin()));
            gear_pts.push(Point2D::new(cx + r_outer * a1.cos(), cy + r_outer * a1.sin()));
            gear_pts.push(Point2D::new(cx + r_outer * a2.cos(), cy + r_outer * a2.sin()));
            gear_pts.push(Point2D::new(cx + r_inner * a3.cos(), cy + r_inner * a3.sin()));
        }
        let gear = Shape::Path {
            layer_index: 0,
            polyline: Polyline::new(gear_pts, true),
        };
        // Inside gear (e.g. at 180, 180)
        assert!(gear.hit_test(&Point2D::new(180.0, 180.0), 1.0));
        // Outside gear (radius 70 at angle 0 -> 250, 180)
        assert!(!gear.hit_test(&Point2D::new(250.0, 180.0), 1.0));
    }

    #[test]
    fn test_shape_flip_scale() {
        let mut rect = Shape::Rect {
            layer_index: 0,
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
            corner_radius: 0.0,
        };
        let b = rect.bounds();
        let c = b.center();
        // Flip horizontal around center
        rect.scale(c, -1.0, 1.0);
        let b_flipped = rect.bounds();
        assert!((b_flipped.min_x - 10.0).abs() < 1e-4);
        assert!((b_flipped.max_x - 40.0).abs() < 1e-4);
        assert!((b_flipped.min_y - 20.0).abs() < 1e-4);
        assert!((b_flipped.max_y - 60.0).abs() < 1e-4);

        // Flip vertical around center
        rect.scale(c, 1.0, -1.0);
        let b_v_flipped = rect.bounds();
        assert!((b_v_flipped.min_x - 10.0).abs() < 1e-4);
        assert!((b_v_flipped.max_x - 40.0).abs() < 1e-4);
        assert!((b_v_flipped.min_y - 20.0).abs() < 1e-4);
        assert!((b_v_flipped.max_y - 60.0).abs() < 1e-4);
    }
}
