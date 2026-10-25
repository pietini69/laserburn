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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub index: usize,
    pub name: String,
    pub color_hex: String, // e.g. "#0000FF", "#FF0000"
    pub priority: i32,     // lower numbers execute first
    pub visible: bool,
    pub locked: bool,
    pub cut_setting: CutSetting,
}

impl Layer {
    pub fn new(index: usize, name: &str, color_hex: &str) -> Self {
        Self {
            index,
            name: name.to_string(),
            color_hex: color_hex.to_string(),
            priority: index as i32,
            visible: true,
            locked: false,
            cut_setting: CutSetting::default(),
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
            Shape::Circle { center, radius, .. } => {
                center.x = origin.x + (center.x - origin.x) * sx;
                center.y = origin.y + (center.y - origin.y) * sy;
                *radius *= (sx.abs() + sy.abs()) * 0.5;
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
            Shape::Image { .. } => vec![],
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
    pub fn new(name: &str, width: f64, height: f64) -> Self {
        let mut layers = BTreeMap::new();
        // Standard LightBurn default color layers (C00 to C07)
        let default_palette = [
            (0, "C00 (Black)", "#000000"),
            (1, "C01 (Blue)", "#0000FF"),
            (2, "C02 (Red)", "#FF0000"),
            (3, "C03 (Green)", "#00AA00"),
            (4, "C04 (Yellow)", "#D4AA00"),
            (5, "C05 (Cyan)", "#00CCCC"),
            (6, "C06 (Magenta)", "#CC00CC"),
            (7, "C07 (Orange)", "#FF8800"),
        ];

        for (idx, name, hex) in default_palette {
            layers.insert(idx, Layer::new(idx, name, hex));
        }

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
            ("T0 (Tool 0)".to_string(), "#00FF00")
        } else if index == 31 {
            ("T1 (Tool 1)".to_string(), "#FF6600")
        } else if index == 32 {
            ("T2 (Tool 2)".to_string(), "#00CCFF")
        } else if index < palette.len() {
            (format!("C{:02}", index), palette[index])
        } else {
            (format!("C{:02}", index), "#888888")
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
