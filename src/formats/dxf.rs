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

impl DxfEntity {
    fn transformed(&self, ins_x: f64, ins_y: f64, sx: f64, sy: f64, rot_deg: f64) -> Self {
        let rad = rot_deg.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();
        let transform_pt = |p: Point2D| -> Point2D {
            let px = p.x * sx;
            let py = p.y * sy;
            let rx = px * cos_a - py * sin_a + ins_x;
            let ry = px * sin_a + py * cos_a + ins_y;
            Point2D::new(rx, ry)
        };

        match self {
            DxfEntity::Line { start, end, layer, color_index } => DxfEntity::Line {
                start: transform_pt(*start),
                end: transform_pt(*end),
                layer: layer.clone(),
                color_index: *color_index,
            },
            DxfEntity::Polyline { vertices, is_closed, layer, color_index } => DxfEntity::Polyline {
                vertices: vertices
                    .iter()
                    .map(|v| DxfVertex {
                        point: transform_pt(v.point),
                        bulge: v.bulge,
                    })
                    .collect(),
                is_closed: *is_closed,
                layer: layer.clone(),
                color_index: *color_index,
            },
            DxfEntity::Circle { center, radius, layer, color_index } => DxfEntity::Circle {
                center: transform_pt(*center),
                radius: radius * ((sx.abs() + sy.abs()) * 0.5),
                layer: layer.clone(),
                color_index: *color_index,
            },
            DxfEntity::Arc { center, radius, start_angle_deg, end_angle_deg, layer, color_index } => DxfEntity::Arc {
                center: transform_pt(*center),
                radius: radius * ((sx.abs() + sy.abs()) * 0.5),
                start_angle_deg: start_angle_deg + rot_deg,
                end_angle_deg: end_angle_deg + rot_deg,
                layer: layer.clone(),
                color_index: *color_index,
            },
            DxfEntity::Ellipse { center, major_axis, ratio, start_param, end_param, layer, color_index } => DxfEntity::Ellipse {
                center: transform_pt(*center),
                major_axis: Point2D::new(
                    major_axis.x * sx * cos_a - major_axis.y * sy * sin_a,
                    major_axis.x * sx * sin_a + major_axis.y * sy * cos_a,
                ),
                ratio: *ratio,
                start_param: *start_param,
                end_param: *end_param,
                layer: layer.clone(),
                color_index: *color_index,
            },
        }
    }
}

impl DxfFormat {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Document, String> {
        let bytes = std::fs::read(path.as_ref())
            .map_err(|e| format!("Failed to read DXF file {}: {}", path.as_ref().display(), e))?;
        let content = String::from_utf8_lossy(&bytes);
        Self::load_from_str(&content)
    }

    pub fn load_from_str(content: &str) -> Result<Document, String> {
        let clean = content.strip_prefix('\u{feff}').unwrap_or(content);
        let lines: Vec<&str> = clean.lines().map(|l| l.trim()).collect();
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

        let layer_colors = Self::parse_layer_table(&pairs);
        let entities = Self::parse_entities(&pairs)?;
        if entities.is_empty() {
            return Err("No supported 2D geometric entities found in DXF file (ensure file contains LINE, POLYLINE, CIRCLE, ARC, or SPLINE entities)".to_string());
        }

        let mut doc = Document::new("DXF Import", 400.0, 400.0);
        let mut layer_map: HashMap<String, usize> = HashMap::new();
        let mut next_layer_idx = 0usize;

        // First pass: collect shapes and compute global bounding box
        let mut raw_shapes = Vec::new();
        let mut bbox = BoundingBox::empty();

        for ent in entities {
            let (shape, layer_name, mut color) = Self::convert_entity_to_shape(ent);
            if color == 0 || color == 256 {
                if let Some(&tbl_color) = layer_colors.get(&layer_name) {
                    color = tbl_color;
                }
            }
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

    fn parse_layer_table(pairs: &[(i32, String)]) -> HashMap<String, i32> {
        let mut map = HashMap::new();
        let mut in_layer_table = false;
        let mut i = 0;
        while i < pairs.len() {
            if pairs[i].0 == 0 && pairs[i].1.eq_ignore_ascii_case("TABLE") {
                if i + 1 < pairs.len() && pairs[i + 1].0 == 2 && pairs[i + 1].1.eq_ignore_ascii_case("LAYER") {
                    in_layer_table = true;
                    i += 2;
                    continue;
                }
            } else if in_layer_table && pairs[i].0 == 0 && pairs[i].1.eq_ignore_ascii_case("ENDTAB") {
                break;
            }

            if in_layer_table && pairs[i].0 == 0 && pairs[i].1.eq_ignore_ascii_case("LAYER") {
                i += 1;
                let mut layer_name = String::new();
                let mut color_idx = 7;
                while i < pairs.len() && pairs[i].0 != 0 {
                    if pairs[i].0 == 2 {
                        layer_name = pairs[i].1.clone();
                    } else if pairs[i].0 == 62 {
                        color_idx = pairs[i].1.trim().parse::<i32>().unwrap_or(7).abs();
                    }
                    i += 1;
                }
                if !layer_name.is_empty() {
                    map.insert(layer_name, color_idx);
                }
                continue;
            }
            i += 1;
        }
        map
    }

    fn parse_entities(pairs: &[(i32, String)]) -> Result<Vec<DxfEntity>, String> {
        let mut blocks: HashMap<String, Vec<DxfEntity>> = HashMap::new();
        let mut entities = Vec::new();

        // 1. Parse BLOCKS section
        let mut in_blocks = false;
        let mut current_block_name: Option<String> = None;
        let mut current_block_ents = Vec::new();
        let mut i = 0;

        while i < pairs.len() {
            let (code, val) = &pairs[i];
            if *code == 0 && val.eq_ignore_ascii_case("SECTION") {
                if i + 1 < pairs.len() && pairs[i + 1].0 == 2 && pairs[i + 1].1.eq_ignore_ascii_case("BLOCKS") {
                    in_blocks = true;
                    i += 2;
                    continue;
                }
            } else if *code == 0 && val.eq_ignore_ascii_case("ENDSEC") {
                if in_blocks {
                    in_blocks = false;
                }
            }

            if in_blocks && *code == 0 {
                let tag = val.to_ascii_uppercase();
                if tag == "BLOCK" {
                    if let Some(name) = current_block_name.take() {
                        blocks.insert(name, std::mem::take(&mut current_block_ents));
                    }
                    i += 1;
                    while i < pairs.len() && pairs[i].0 != 0 {
                        if pairs[i].0 == 2 {
                            current_block_name = Some(pairs[i].1.clone());
                        }
                        i += 1;
                    }
                    continue;
                } else if tag == "ENDBLK" {
                    if let Some(name) = current_block_name.take() {
                        blocks.insert(name, std::mem::take(&mut current_block_ents));
                    }
                    i += 1;
                    continue;
                } else {
                    let (ent, next_i) = Self::parse_single_entity(&tag, pairs, i + 1);
                    i = next_i;
                    if let Some(e) = ent {
                        current_block_ents.push(e);
                    }
                    continue;
                }
            }
            i += 1;
        }
        if let Some(name) = current_block_name.take() {
            blocks.insert(name, current_block_ents);
        }

        // 2. Parse ENTITIES section
        let mut in_entities = false;
        i = 0;
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
                if ent_type == "INSERT" {
                    i += 1;
                    let mut bname = String::new();
                    let mut ins_x = 0.0;
                    let mut ins_y = 0.0;
                    let mut sx = 1.0;
                    let mut sy = 1.0;
                    let mut rot_deg = 0.0;
                    while i < pairs.len() && pairs[i].0 != 0 {
                        match pairs[i].0 {
                            2 => bname = pairs[i].1.clone(),
                            10 => ins_x = pairs[i].1.trim().parse().unwrap_or(0.0),
                            20 => ins_y = pairs[i].1.trim().parse().unwrap_or(0.0),
                            41 => sx = pairs[i].1.trim().parse().unwrap_or(1.0),
                            42 => sy = pairs[i].1.trim().parse().unwrap_or(1.0),
                            50 => rot_deg = pairs[i].1.trim().parse().unwrap_or(0.0),
                            _ => {}
                        }
                        i += 1;
                    }
                    if let Some(block_ents) = blocks.get(&bname) {
                        for bent in block_ents {
                            entities.push(bent.transformed(ins_x, ins_y, sx, sy, rot_deg));
                        }
                    }
                    continue;
                } else {
                    let (ent, next_i) = Self::parse_single_entity(&ent_type, pairs, i + 1);
                    i = next_i;
                    if let Some(e) = ent {
                        entities.push(e);
                    }
                    continue;
                }
            }
            i += 1;
        }

        // 3. Fallbacks: if no entities found, check *Model_Space or all non-empty blocks
        if entities.is_empty() {
            for (bname, bents) in &blocks {
                if bname.eq_ignore_ascii_case("*Model_Space") || bname.eq_ignore_ascii_case("Model") {
                    entities.extend(bents.clone());
                }
            }
        }
        if entities.is_empty() && !blocks.is_empty() {
            for (bname, bents) in &blocks {
                if !bname.starts_with('*') {
                    entities.extend(bents.clone());
                }
            }
        }
        if entities.is_empty() {
            // Try parsing top-level pairs without SECTION
            i = 0;
            while i < pairs.len() {
                if pairs[i].0 == 0 {
                    let ent_type = pairs[i].1.to_ascii_uppercase();
                    if ent_type != "SECTION" && ent_type != "ENDSEC" && ent_type != "EOF" {
                        let (ent, next_i) = Self::parse_single_entity(&ent_type, pairs, i + 1);
                        i = next_i;
                        if let Some(e) = ent {
                            entities.push(e);
                        }
                        continue;
                    }
                }
                i += 1;
            }
        }

        Ok(entities)
    }

    fn parse_single_entity(ent_type: &str, pairs: &[(i32, String)], start_idx: usize) -> (Option<DxfEntity>, usize) {
        let mut i = start_idx;

        if ent_type == "POLYLINE" {
            let mut poly_pairs = Vec::new();
            while i < pairs.len() && pairs[i].0 != 0 {
                poly_pairs.push((pairs[i].0, pairs[i].1.clone()));
                i += 1;
            }
            let mut layer = "0".to_string();
            let mut color_index = 0i32;
            let mut is_closed = false;
            for (c, v) in &poly_pairs {
                if *c == 8 {
                    layer = v.clone();
                } else if *c == 62 {
                    color_index = v.trim().parse().unwrap_or(0);
                } else if *c == 70 {
                    let f: i32 = v.trim().parse().unwrap_or(0);
                    is_closed = (f & 1) != 0;
                }
            }

            let mut vertices = Vec::new();
            while i < pairs.len() {
                if pairs[i].0 == 0 {
                    let sub_tag = pairs[i].1.to_ascii_uppercase();
                    if sub_tag == "SEQEND" || sub_tag == "ENDSEC" {
                        i += 1;
                        break;
                    } else if sub_tag == "VERTEX" {
                        i += 1;
                        let mut vx = 0.0;
                        let mut vy = 0.0;
                        let mut bulge = 0.0;
                        while i < pairs.len() && pairs[i].0 != 0 {
                            match pairs[i].0 {
                                10 => vx = pairs[i].1.trim().parse().unwrap_or(0.0),
                                20 => vy = pairs[i].1.trim().parse().unwrap_or(0.0),
                                42 => bulge = pairs[i].1.trim().parse().unwrap_or(0.0),
                                _ => {}
                            }
                            i += 1;
                        }
                        vertices.push(DxfVertex { point: Point2D::new(vx, vy), bulge });
                        continue;
                    } else {
                        break;
                    }
                }
                i += 1;
            }

            let entity = if vertices.len() >= 2 {
                Some(DxfEntity::Polyline { vertices, is_closed, layer, color_index })
            } else {
                None
            };
            return (entity, i);
        }

        let mut ent_pairs = Vec::new();
        while i < pairs.len() && pairs[i].0 != 0 {
            ent_pairs.push((pairs[i].0, pairs[i].1.clone()));
            i += 1;
        }

        let entity = Self::build_entity(ent_type, &ent_pairs);
        (entity, i)
    }

    fn build_entity(ent_type: &str, pairs: &[(i32, String)]) -> Option<DxfEntity> {
        let mut layer = "0".to_string();
        let mut color_index = 0i32;

        for (code, val) in pairs {
            if *code == 8 {
                layer = val.clone();
            } else if *code == 62 {
                color_index = val.trim().parse().unwrap_or(0);
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
                        10 => x1 = val.trim().parse().unwrap_or(0.0),
                        20 => y1 = val.trim().parse().unwrap_or(0.0),
                        11 => x2 = val.trim().parse().unwrap_or(0.0),
                        21 => y2 = val.trim().parse().unwrap_or(0.0),
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
                            let flags: i32 = val.trim().parse().unwrap_or(0);
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
                            cur_x = val.trim().parse().ok();
                        }
                        20 => {
                            cur_y = val.trim().parse().ok();
                        }
                        42 => {
                            cur_bulge = val.trim().parse().unwrap_or(0.0);
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
                        10 => cx = val.trim().parse().unwrap_or(0.0),
                        20 => cy = val.trim().parse().unwrap_or(0.0),
                        40 => r = val.trim().parse().unwrap_or(0.0),
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
                        10 => cx = val.trim().parse().unwrap_or(0.0),
                        20 => cy = val.trim().parse().unwrap_or(0.0),
                        40 => r = val.trim().parse().unwrap_or(0.0),
                        50 => start_deg = val.trim().parse().unwrap_or(0.0),
                        51 => end_deg = val.trim().parse().unwrap_or(360.0),
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
                        10 => cx = val.trim().parse().unwrap_or(0.0),
                        20 => cy = val.trim().parse().unwrap_or(0.0),
                        11 => mx = val.trim().parse().unwrap_or(10.0),
                        21 => my = val.trim().parse().unwrap_or(0.0),
                        40 => ratio = val.trim().parse().unwrap_or(0.5),
                        41 => start_p = val.trim().parse().unwrap_or(0.0),
                        42 => end_p = val.trim().parse().unwrap_or(std::f64::consts::TAU),
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
            "SPLINE" => {
                let mut is_closed = false;
                let mut fit_pts = Vec::new();
                let mut ctrl_pts = Vec::new();
                let mut cur_fx: Option<f64> = None;
                let mut cur_cx: Option<f64> = None;

                for (code, val) in pairs {
                    match *code {
                        70 => {
                            let f: i32 = val.trim().parse().unwrap_or(0);
                            is_closed = (f & 1) != 0;
                        }
                        11 => cur_fx = val.trim().parse().ok(),
                        21 => {
                            if let Some(x) = cur_fx.take() {
                                if let Ok(y) = val.trim().parse::<f64>() {
                                    fit_pts.push(Point2D::new(x, y));
                                }
                            }
                        }
                        10 => cur_cx = val.trim().parse().ok(),
                        20 => {
                            if let Some(x) = cur_cx.take() {
                                if let Ok(y) = val.trim().parse::<f64>() {
                                    ctrl_pts.push(Point2D::new(x, y));
                                }
                            }
                        }
                        _ => {}
                    }
                }

                let pts = if fit_pts.len() >= 2 {
                    fit_pts
                } else if ctrl_pts.len() >= 2 {
                    ctrl_pts
                } else {
                    Vec::new()
                };

                if pts.len() >= 2 {
                    Some(DxfEntity::Polyline {
                        vertices: pts.into_iter().map(|p| DxfVertex { point: p, bulge: 0.0 }).collect(),
                        is_closed,
                        layer,
                        color_index,
                    })
                } else {
                    None
                }
            }
            "SOLID" | "3DFACE" | "TRACE" => {
                let mut x0 = 0.0; let mut y0 = 0.0;
                let mut x1 = 0.0; let mut y1 = 0.0;
                let mut x2 = 0.0; let mut y2 = 0.0;
                let mut x3 = 0.0; let mut y3 = 0.0;
                let mut has3 = false;
                for (code, val) in pairs {
                    match *code {
                        10 => x0 = val.trim().parse().unwrap_or(0.0),
                        20 => y0 = val.trim().parse().unwrap_or(0.0),
                        11 => x1 = val.trim().parse().unwrap_or(0.0),
                        21 => y1 = val.trim().parse().unwrap_or(0.0),
                        12 => x2 = val.trim().parse().unwrap_or(0.0),
                        22 => y2 = val.trim().parse().unwrap_or(0.0),
                        13 => { x3 = val.trim().parse().unwrap_or(0.0); has3 = true; }
                        23 => { y3 = val.trim().parse().unwrap_or(0.0); has3 = true; }
                        _ => {}
                    }
                }
                let mut pts = vec![Point2D::new(x0, y0), Point2D::new(x1, y1), Point2D::new(x2, y2)];
                if has3 && (Point2D::new(x3, y3).distance_to(&Point2D::new(x2, y2)) > 1e-4) {
                    pts.push(Point2D::new(x3, y3));
                }
                Some(DxfEntity::Polyline {
                    vertices: pts.into_iter().map(|p| DxfVertex { point: p, bulge: 0.0 }).collect(),
                    is_closed: true,
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

    #[test]
    fn test_sample_bracket_dxf_loading() {
        let sample_str = include_str!("../../samples/sample_bracket.dxf");
        let doc = DxfFormat::load_from_str(sample_str).expect("Sample bracket DXF must load");
        assert!(doc.shapes.len() >= 10, "Bracket DXF has perimeter, circles, and lines");
        assert!(doc.layers.len() >= 3, "Has CUT, HOLES, ENGRAVE layers");
    }

    #[test]
    fn test_dxf_polyline_with_vertex_seqend() {
        let dxf = r#"0
SECTION
2
ENTITIES
0
POLYLINE
8
CUT
70
1
0
VERTEX
10
10.0
20
10.0
42
0.0
0
VERTEX
10
50.0
20
10.0
42
0.5
0
VERTEX
10
50.0
20
50.0
42
0.0
0
VERTEX
10
10.0
20
50.0
42
0.0
0
SEQEND
0
ENDSEC
0
EOF"#;
        let doc = DxfFormat::load_from_str(dxf).expect("Classic POLYLINE with VERTEX/SEQEND should parse");
        assert_eq!(doc.shapes.len(), 1);
        if let Shape::Path { polyline, .. } = &doc.shapes[0] {
            assert!(polyline.is_closed);
            assert!(polyline.points.len() > 4);
        } else {
            panic!("Expected Shape::Path");
        }
    }

    #[test]
    fn test_dxf_blocks_and_insert() {
        let dxf = r#"0
SECTION
2
BLOCKS
0
BLOCK
2
HOLE_BLOCK
0
CIRCLE
8
HOLES
10
0.0
20
0.0
40
5.0
0
ENDBLK
0
ENDSEC
0
SECTION
2
ENTITIES
0
INSERT
2
HOLE_BLOCK
10
30.0
20
40.0
41
1.0
42
1.0
50
0.0
0
ENDSEC
0
EOF"#;
        let doc = DxfFormat::load_from_str(dxf).expect("DXF with BLOCKS and INSERT should instantiate");
        assert_eq!(doc.shapes.len(), 1);
        if let Shape::Circle { radius, .. } = &doc.shapes[0] {
            assert_eq!(*radius, 5.0);
        } else {
            panic!("Expected Shape::Circle");
        }
    }
}
