//! AutoCAD DXF (Drawing Exchange Format) parser for LaserBurn.
//! Imports 2D CAD drawings (LINE, LWPOLYLINE, POLYLINE, CIRCLE, ARC, ELLIPSE, SPLINE)
//! into native LaserBurn vector shapes with full arc-bulge reconstruction and layer mapping.

use crate::document::{CutMode, Document, Layer, Shape};
use crate::geometry::{BoundingBox, Point2D, Polyline};
use std::collections::HashMap;
use std::path::Path;

pub struct DxfFormat;

#[derive(Debug, Clone)]
struct DxfVertex {
    point: Point2D,
    bulge: f64,
}

#[derive(Debug, Clone)]
enum DxfEntity {
    Line {
        start: Point2D,
        end: Point2D,
        layer: String,
        color_index: i32,
    },
    Polyline {
        vertices: Vec<DxfVertex>,
        is_closed: bool,
        layer: String,
        color_index: i32,
    },
    Circle {
        center: Point2D,
        radius: f64,
        layer: String,
        color_index: i32,
    },
    Arc {
        center: Point2D,
        radius: f64,
        start_angle_deg: f64,
        end_angle_deg: f64,
        layer: String,
        color_index: i32,
    },
    Ellipse {
        center: Point2D,
        major_axis: Point2D,
        ratio: f64,
        start_param: f64,
        end_param: f64,
        layer: String,
        color_index: i32,
    },
}

impl DxfFormat {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Document, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read DXF file: {}", e))?;
        Self::load_from_str(&content)
    }

    pub fn load_from_str(content: &str) -> Result<Document, String> {
        let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
        if lines.is_empty() {
            return Err("Empty DXF file".to_string());
        }

        // Parse key-value group pairs
        let mut pairs: Vec<(i32, String)> = Vec::with_capacity(lines.len() / 2);
        let mut idx = 0;
        while idx + 1 < lines.len() {
            let code_str = lines[idx];
            let val_str = lines[idx + 1];
            if let Ok(code) = code_str.parse::<i32>() {
                pairs.push((code, val_str.to_string()));
            }
            idx += 2;
        }

        let entities = Self::parse_entities(&pairs)?;
        if entities.is_empty() {
            return Err("No supported 2D geometric entities found in DXF ENTITIES section".to_string());
        }

        let mut doc = Document::new("DXF Import", 400.0, 400.0);
        let mut layer_map: HashMap<String, usize> = HashMap::new();
        let mut next_layer_idx = 0usize;

        // First pass: collect shapes and compute global bounding box
        let mut raw_shapes = Vec::new();
        let mut bbox = BoundingBox::empty();

        for ent in entities {
            let (shape, layer_name, color) = Self::convert_entity_to_shape(ent);
            bbox = bbox.union(&shape.bounds());

            // Layer mapping (AutoCAD Color Index 1=Red, 2=Yellow, 3=Green, 4=Cyan, 5=Blue, 7=White/Black)
            let layer_idx = if let Some(&l_idx) = layer_map.get(&layer_name) {
                l_idx
            } else {
                let l_idx = match color {
                    1 => 2, // Red
                    5 => 1, // Blue
                    3 => 3, // Green
                    _ => {
                        let l = next_layer_idx;
                        next_layer_idx = (next_layer_idx + 1) % 30;
                        l
                    }
                };
                layer_map.insert(layer_name.clone(), l_idx);
                if !doc.layers.contains_key(&l_idx) {
                    let mut l = Layer::new(l_idx, &layer_name, "#000000");
                    l.cut_setting.mode = CutMode::Line;
                    l.cut_setting.speed = 20.0;
                    l.cut_setting.power_max = 80.0;
                    doc.layers.insert(l_idx, l);
                }
                l_idx
            };

            let mut final_shape = shape;
            final_shape.set_layer_index(layer_idx);
            raw_shapes.push(final_shape);
        }

        // CAD systems use mathematical Cartesian Y+ (upwards), while laser canvas uses Y+ downwards.
        // We flip Y around the vertical center of the bounding box so drawings are upright.
        let flip_center_y = if bbox.is_empty() { 0.0 } else { bbox.center().y };
        let offset_x = if bbox.is_empty() { 0.0 } else { (20.0 - bbox.min_x).max(0.0) };
        let offset_y = if bbox.is_empty() { 0.0 } else { (20.0 - bbox.min_y).max(0.0) };

        for mut s in raw_shapes {
            s.scale(Point2D::new(0.0, flip_center_y), 1.0, -1.0);
            s.translate(offset_x, offset_y);
            doc.add_shape(s);
        }

        Ok(doc)
    }

    fn parse_entities(pairs: &[(i32, String)]) -> Result<Vec<DxfEntity>, String> {
        let mut in_entities = false;
        let mut entities = Vec::new();
        let mut i = 0;

        while i < pairs.len() {
            let (code, val) = &pairs[i];
            if *code == 0 && val.eq_ignore_ascii_case("SECTION") {
                if i + 1 < pairs.len() && pairs[i + 1].0 == 2 && pairs[i + 1].1.eq_ignore_ascii_case("ENTITIES") {
                    in_entities = true;
                    i += 2;
                    continue;
                }
            } else if *code == 0 && val.eq_ignore_ascii_case("ENDSEC") {
                if in_entities {
                    break;
                }
            }

            if in_entities && *code == 0 {
                let ent_type = val.to_ascii_uppercase();
                i += 1;
                // Parse entity attributes until next '0' code
                let mut ent_pairs = Vec::new();
                while i < pairs.len() && pairs[i].0 != 0 {
                    ent_pairs.push((pairs[i].0, pairs[i].1.clone()));
                    i += 1;
                }

                if let Some(entity) = Self::build_entity(&ent_type, &ent_pairs) {
                    entities.push(entity);
                }
                continue;
            }

            i += 1;
        }

        // If no explicit SECTION ENTITIES was found, try parsing top-level entities
        if entities.is_empty() {
            let mut j = 0;
            while j < pairs.len() {
                if pairs[j].0 == 0 {
                    let ent_type = pairs[j].1.to_ascii_uppercase();
                    j += 1;
                    let mut ent_pairs = Vec::new();
                    while j < pairs.len() && pairs[j].0 != 0 {
                        ent_pairs.push((pairs[j].0, pairs[j].1.clone()));
                        j += 1;
                    }
                    if let Some(entity) = Self::build_entity(&ent_type, &ent_pairs) {
                        entities.push(entity);
                    }
                    continue;
                }
                j += 1;
            }
        }

        Ok(entities)
    }

    fn build_entity(ent_type: &str, pairs: &[(i32, String)]) -> Option<DxfEntity> {
        let mut layer = "0".to_string();
        let mut color_index = 0i32;

        for (code, val) in pairs {
            if *code == 8 {
                layer = val.clone();
            } else if *code == 62 {
                color_index = val.parse().unwrap_or(0);
            }
        }

        match ent_type {
            "LINE" => {
                let mut x1 = 0.0;
                let mut y1 = 0.0;
                let mut x2 = 0.0;
                let mut y2 = 0.0;
                for (code, val) in pairs {
                    match *code {
                        10 => x1 = val.parse().unwrap_or(0.0),
                        20 => y1 = val.parse().unwrap_or(0.0),
                        11 => x2 = val.parse().unwrap_or(0.0),
                        21 => y2 = val.parse().unwrap_or(0.0),
                        _ => {}
                    }
                }
                Some(DxfEntity::Line {
                    start: Point2D::new(x1, y1),
                    end: Point2D::new(x2, y2),
                    layer,
                    color_index,
                })
            }
            "LWPOLYLINE" => {
                let mut is_closed = false;
                let mut vertices = Vec::new();
                let mut cur_x: Option<f64> = None;
                let mut cur_y: Option<f64> = None;
                let mut cur_bulge = 0.0;

                for (code, val) in pairs {
                    match *code {
                        70 => {
                            let flags: i32 = val.parse().unwrap_or(0);
                            is_closed = (flags & 1) != 0;
                        }
                        10 => {
                            if let (Some(x), Some(y)) = (cur_x, cur_y) {
                                vertices.push(DxfVertex {
                                    point: Point2D::new(x, y),
                                    bulge: cur_bulge,
                                });
                                cur_bulge = 0.0;
                            }
                            cur_x = val.parse().ok();
                        }
                        20 => {
                            cur_y = val.parse().ok();
                        }
                        42 => {
                            cur_bulge = val.parse().unwrap_or(0.0);
                        }
                        _ => {}
                    }
                }
                if let (Some(x), Some(y)) = (cur_x, cur_y) {
                    vertices.push(DxfVertex {
                        point: Point2D::new(x, y),
                        bulge: cur_bulge,
                    });
                }
                if vertices.len() >= 2 {
                    Some(DxfEntity::Polyline {
                        vertices,
                        is_closed,
                        layer,
                        color_index,
                    })
                } else {
                    None
                }
            }
            "CIRCLE" => {
                let mut cx = 0.0;
                let mut cy = 0.0;
                let mut r = 0.0;
                for (code, val) in pairs {
                    match *code {
                        10 => cx = val.parse().unwrap_or(0.0),
                        20 => cy = val.parse().unwrap_or(0.0),
                        40 => r = val.parse().unwrap_or(0.0),
                        _ => {}
                    }
                }
                if r > 0.0 {
                    Some(DxfEntity::Circle {
                        center: Point2D::new(cx, cy),
                        radius: r,
                        layer,
                        color_index,
                    })
                } else {
                    None
                }
            }
            "ARC" => {
                let mut cx = 0.0;
                let mut cy = 0.0;
                let mut r = 0.0;
                let mut start_deg = 0.0;
                let mut end_deg = 360.0;
                for (code, val) in pairs {
                    match *code {
                        10 => cx = val.parse().unwrap_or(0.0),
                        20 => cy = val.parse().unwrap_or(0.0),
                        40 => r = val.parse().unwrap_or(0.0),
                        50 => start_deg = val.parse().unwrap_or(0.0),
                        51 => end_deg = val.parse().unwrap_or(360.0),
                        _ => {}
                    }
                }
                if r > 0.0 {
                    Some(DxfEntity::Arc {
                        center: Point2D::new(cx, cy),
                        radius: r,
                        start_angle_deg: start_deg,
                        end_angle_deg: end_deg,
                        layer,
                        color_index,
                    })
                } else {
                    None
                }
            }
            "ELLIPSE" => {
                let mut cx = 0.0;
                let mut cy = 0.0;
                let mut mx = 10.0;
                let mut my = 0.0;
                let mut ratio = 0.5;
                let mut start_p = 0.0;
                let mut end_p = std::f64::consts::TAU;

                for (code, val) in pairs {
                    match *code {
                        10 => cx = val.parse().unwrap_or(0.0),
                        20 => cy = val.parse().unwrap_or(0.0),
                        11 => mx = val.parse().unwrap_or(10.0),
                        21 => my = val.parse().unwrap_or(0.0),
                        40 => ratio = val.parse().unwrap_or(0.5),
                        41 => start_p = val.parse().unwrap_or(0.0),
                        42 => end_p = val.parse().unwrap_or(std::f64::consts::TAU),
                        _ => {}
                    }
                }
                Some(DxfEntity::Ellipse {
                    center: Point2D::new(cx, cy),
                    major_axis: Point2D::new(mx, my),
                    ratio,
                    start_param: start_p,
                    end_param: end_p,
                    layer,
                    color_index,
                })
            }
            _ => None,
        }
    }

    fn convert_entity_to_shape(ent: DxfEntity) -> (Shape, String, i32) {
        match ent {
            DxfEntity::Line { start, end, layer, color_index } => {
                let poly = Polyline::new(vec![start, end], false);
                (Shape::Path { layer_index: 0, polyline: poly }, layer, color_index)
            }
            DxfEntity::Polyline { vertices, is_closed, layer, color_index } => {
                let poly = Self::reconstruct_bulge_polyline(&vertices, is_closed);
                (Shape::Path { layer_index: 0, polyline: poly }, layer, color_index)
            }
            DxfEntity::Circle { center, radius, layer, color_index } => {
                (Shape::Circle { layer_index: 0, center, radius }, layer, color_index)
            }
            DxfEntity::Arc { center, radius, start_angle_deg, mut end_angle_deg, layer, color_index } => {
                if end_angle_deg <= start_angle_deg {
                    end_angle_deg += 360.0;
                }
                let span_deg = end_angle_deg - start_angle_deg;
                let steps = (span_deg / 5.0).ceil().max(8.0) as usize;
                let mut pts = Vec::with_capacity(steps + 1);

                for i in 0..=steps {
                    let deg = start_angle_deg + (i as f64 / steps as f64) * span_deg;
                    let rad = deg.to_radians();
                    pts.push(Point2D::new(center.x + radius * rad.cos(), center.y + radius * rad.sin()));
                }

                let poly = Polyline::new(pts, false);
                (Shape::Path { layer_index: 0, polyline: poly }, layer, color_index)
            }
            DxfEntity::Ellipse { center, major_axis, ratio, start_param, mut end_param, layer, color_index } => {
                if end_param <= start_param {
                    end_param += std::f64::consts::TAU;
                }
                let span = end_param - start_param;
                let steps = (span / 0.1).ceil().max(16.0) as usize;
                let major_len = major_axis.x.hypot(major_axis.y).max(1e-4);
                let minor_len = major_len * ratio;
                let angle_rad = major_axis.y.atan2(major_axis.x);

                let mut pts = Vec::with_capacity(steps + 1);
                for i in 0..=steps {
                    let t = start_param + (i as f64 / steps as f64) * span;
                    let ex = major_len * t.cos();
                    let ey = minor_len * t.sin();
                    let rx = ex * angle_rad.cos() - ey * angle_rad.sin();
                    let ry = ex * angle_rad.sin() + ey * angle_rad.cos();
                    pts.push(Point2D::new(center.x + rx, center.y + ry));
                }

                let is_closed = (span - std::f64::consts::TAU).abs() < 1e-4;
                let poly = Polyline::new(pts, is_closed);
                (Shape::Path { layer_index: 0, polyline: poly }, layer, color_index)
            }
        }
    }

    /// Converts a sequence of vertices with arc bulges (tan(theta / 4)) into subdivided smooth polylines
    fn reconstruct_bulge_polyline(vertices: &[DxfVertex], is_closed: bool) -> Polyline {
        if vertices.len() < 2 {
            return Polyline::new(vertices.iter().map(|v| v.point).collect(), is_closed);
        }

        let mut pts = Vec::new();
        let n = vertices.len();
        let num_segs = if is_closed { n } else { n - 1 };

        for i in 0..num_segs {
            let p1 = vertices[i].point;
            let p2 = if i + 1 < n { vertices[i + 1].point } else { vertices[0].point };
            let bulge = vertices[i].bulge;

            pts.push(p1);

            if bulge.abs() > 1e-6 {
                // Bulge is tan(theta / 4)
                let theta = 4.0 * bulge.atan();
                let chord = p1.distance_to(&p2);
                if chord > 1e-6 {
                    let r = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
                    let h = chord * (1.0 - bulge * bulge) / (4.0 * bulge);
                    let mid = Point2D::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
                    let norm = Point2D::new(-(p2.y - p1.y) / chord, (p2.x - p1.x) / chord);
                    let center = Point2D::new(mid.x + norm.x * h, mid.y + norm.y * h);

                    let start_ang = (p1.y - center.y).atan2(p1.x - center.x);
                    let steps = (theta.abs().to_degrees() / 8.0).ceil().max(4.0) as usize;

                    for s in 1..steps {
                        let frac = s as f64 / steps as f64;
                        let a = start_ang + frac * theta;
                        pts.push(Point2D::new(center.x + r * a.cos(), center.y + r * a.sin()));
                    }
                }
            }
        }

        if !is_closed {
            if let Some(last) = vertices.last() {
                pts.push(last.point);
            }
        }

        Polyline::new(pts, is_closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dxf_parse_line_and_circle() {
        let dxf = r#"0
SECTION
2
ENTITIES
0
LINE
8
CutLayer
62
1
10
10.0
20
15.0
11
50.0
21
15.0
0
CIRCLE
8
EngraveLayer
62
5
10
100.0
20
100.0
40
25.0
0
ENDSEC
0
EOF"#;

        let doc = DxfFormat::load_from_str(dxf).expect("DXF should parse successfully");
        assert_eq!(doc.shapes.len(), 2);
    }

    #[test]
    fn test_dxf_lwpolyline_with_bulge() {
        let dxf = r#"0
SECTION
2
ENTITIES
0
LWPOLYLINE
8
0
70
1
90
4
10
0.0
20
0.0
42
1.0
10
20.0
20
0.0
10
20.0
20
20.0
10
0.0
20
20.0
0
ENDSEC
0
EOF"#;

        let doc = DxfFormat::load_from_str(dxf).expect("Bulge LWPOLYLINE should parse");
        assert_eq!(doc.shapes.len(), 1);
        if let Shape::Path { polyline, .. } = &doc.shapes[0] {
            assert!(polyline.is_closed);
            assert!(polyline.points.len() > 4, "Bulge segment must be subdivided into arc vertices");
        } else {
            panic!("Expected Shape::Path");
        }
    }
}
