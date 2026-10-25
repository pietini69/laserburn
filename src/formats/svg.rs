use crate::document::{Document, Layer, Shape};
use crate::geometry::{Point2D, Polyline};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform2D {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    pub fn apply(&self, p: Point2D) -> Point2D {
        Point2D::new(
            self.a * p.x + self.c * p.y + self.e,
            self.b * p.x + self.d * p.y + self.f,
        )
    }

    pub fn parse(s: &str) -> Self {
        let mut cur = Self::identity();
        let bytes = s.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b',') {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }

            let op_start = i;
            while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            let op = &s[op_start..i];

            while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'(') {
                i += 1;
            }
            let args_start = i;
            while i < bytes.len() && bytes[i] != b')' {
                i += 1;
            }
            let args_str = if args_start <= i && i <= bytes.len() {
                &s[args_start..i]
            } else {
                ""
            };
            if i < bytes.len() && bytes[i] == b')' {
                i += 1;
            }

            let args: Vec<f64> = args_str
                .split(|c: char| c.is_whitespace() || c == ',')
                .filter(|part| !part.is_empty())
                .filter_map(|part| part.parse::<f64>().ok())
                .collect();

            let op_mat = match op {
                "translate" => {
                    let tx = args.first().copied().unwrap_or(0.0);
                    let ty = args.get(1).copied().unwrap_or(0.0);
                    Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: tx, f: ty }
                }
                "scale" => {
                    let sx = args.first().copied().unwrap_or(1.0);
                    let sy = args.get(1).copied().unwrap_or(sx);
                    Self { a: sx, b: 0.0, c: 0.0, d: sy, e: 0.0, f: 0.0 }
                }
                "rotate" => {
                    let deg = args.first().copied().unwrap_or(0.0);
                    let rad = deg.to_radians();
                    let cos = rad.cos();
                    let sin = rad.sin();
                    if args.len() >= 3 {
                        let cx = args[1];
                        let cy = args[2];
                        let t1 = Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: cx, f: cy };
                        let r = Self { a: cos, b: sin, c: -sin, d: cos, e: 0.0, f: 0.0 };
                        let t2 = Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: -cx, f: -cy };
                        t1.multiply(&r).multiply(&t2)
                    } else {
                        Self { a: cos, b: sin, c: -sin, d: cos, e: 0.0, f: 0.0 }
                    }
                }
                "matrix" => {
                    if args.len() >= 6 {
                        Self {
                            a: args[0],
                            b: args[1],
                            c: args[2],
                            d: args[3],
                            e: args[4],
                            f: args[5],
                        }
                    } else {
                        Self::identity()
                    }
                }
                _ => Self::identity(),
            };

            cur = cur.multiply(&op_mat);
        }

        cur
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathToken {
    Cmd(char),
    Num(f64),
}

/// Tokenizes an SVG path `d` string according to SVG 1.1 / 2.0 specifications
pub fn tokenize_path_d(d: &str) -> Vec<PathToken> {
    let mut tokens = Vec::new();
    let bytes = d.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() || b == b',' {
            i += 1;
            continue;
        }

        match b {
            b'M' | b'm' | b'Z' | b'z' | b'L' | b'l' | b'H' | b'h' | b'V' | b'v' | b'C' | b'c'
            | b'S' | b's' | b'Q' | b'q' | b'T' | b't' | b'A' | b'a' => {
                tokens.push(PathToken::Cmd(b as char));
                i += 1;
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => {
                let start = i;
                let mut has_dot = false;
                let mut has_exp = false;

                if bytes[i] == b'+' || bytes[i] == b'-' {
                    i += 1;
                }

                while i < bytes.len() {
                    let c = bytes[i];
                    if c.is_ascii_digit() {
                        i += 1;
                    } else if c == b'.' {
                        if has_dot {
                            // A second dot starts a new number token (e.g. "12.3.4" -> 12.3 and .4)
                            break;
                        }
                        has_dot = true;
                        i += 1;
                    } else if (c == b'e' || c == b'E') && !has_exp {
                        has_exp = true;
                        i += 1;
                        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                            i += 1;
                        }
                    } else {
                        break;
                    }
                }

                if let Ok(val) = d[start..i].parse::<f64>() {
                    tokens.push(PathToken::Num(val));
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    tokens
}

/// Evaluates cubic bezier curve at parameter t (0.0 ..= 1.0)
fn eval_cubic_bezier(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D, t: f64) -> Point2D {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;

    Point2D::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

/// Evaluates quadratic bezier curve at parameter t (0.0 ..= 1.0)
fn eval_quad_bezier(p0: Point2D, p1: Point2D, p2: Point2D, t: f64) -> Point2D {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    Point2D::new(
        mt2 * p0.x + 2.0 * mt * t * p1.x + t2 * p2.x,
        mt2 * p0.y + 2.0 * mt * t * p1.y + t2 * p2.y,
    )
}

/// Parses an SVG path `d` string into vector Polyline paths
pub fn parse_path_d(d: &str) -> Vec<Polyline> {
    let tokens = tokenize_path_d(d);
    let mut polylines = Vec::new();
    let mut current_pts = Vec::new();
    let mut current_pt = Point2D::ZERO;
    let mut subpath_start = Point2D::ZERO;
    let mut last_cubic_ctrl: Option<Point2D> = None;
    let mut last_quad_ctrl: Option<Point2D> = None;
    let mut active_cmd: Option<char> = None;

    let finish_subpath = |pts: &mut Vec<Point2D>, is_closed: bool, polys: &mut Vec<Polyline>| {
        if pts.len() >= 2 {
            polys.push(Polyline::new(std::mem::take(pts), is_closed));
        } else {
            pts.clear();
        }
    };

    let mut idx = 0;
    while idx < tokens.len() {
        match &tokens[idx] {
            PathToken::Cmd(c) => {
                let cmd = *c;
                idx += 1;
                if cmd == 'Z' || cmd == 'z' {
                    if !current_pts.is_empty() {
                        current_pts.push(subpath_start);
                    }
                    finish_subpath(&mut current_pts, true, &mut polylines);
                    current_pt = subpath_start;
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                    active_cmd = None;
                } else {
                    active_cmd = Some(cmd);
                }
            }
            PathToken::Num(_) => {
                let cmd = match active_cmd {
                    Some(c) => c,
                    None => {
                        idx += 1;
                        continue;
                    }
                };

                match cmd {
                    'M' | 'm' => {
                        if idx + 1 < tokens.len() {
                            if let (PathToken::Num(x), PathToken::Num(y)) = (&tokens[idx], &tokens[idx + 1]) {
                                idx += 2;
                                finish_subpath(&mut current_pts, false, &mut polylines);
                                let pt = if cmd == 'M' {
                                    Point2D::new(*x, *y)
                                } else {
                                    Point2D::new(current_pt.x + *x, current_pt.y + *y)
                                };
                                current_pt = pt;
                                subpath_start = pt;
                                current_pts.push(pt);
                                last_cubic_ctrl = None;
                                last_quad_ctrl = None;
                                // Subsequent coordinate pairs for M/m are treated as L/l
                                active_cmd = Some(if cmd == 'M' { 'L' } else { 'l' });
                            } else {
                                idx += 1;
                            }
                        } else {
                            idx += 1;
                        }
                    }
                    'L' | 'l' => {
                        if idx + 1 < tokens.len() {
                            if let (PathToken::Num(x), PathToken::Num(y)) = (&tokens[idx], &tokens[idx + 1]) {
                                idx += 2;
                                let pt = if cmd == 'L' {
                                    Point2D::new(*x, *y)
                                } else {
                                    Point2D::new(current_pt.x + *x, current_pt.y + *y)
                                };
                                current_pt = pt;
                                current_pts.push(pt);
                                last_cubic_ctrl = None;
                                last_quad_ctrl = None;
                            } else {
                                idx += 1;
                            }
                        } else {
                            idx += 1;
                        }
                    }
                    'H' | 'h' => {
                        if let PathToken::Num(x) = tokens[idx] {
                            idx += 1;
                            let pt = if cmd == 'H' {
                                Point2D::new(x, current_pt.y)
                            } else {
                                Point2D::new(current_pt.x + x, current_pt.y)
                            };
                            current_pt = pt;
                            current_pts.push(pt);
                            last_cubic_ctrl = None;
                            last_quad_ctrl = None;
                        } else {
                            idx += 1;
                        }
                    }
                    'V' | 'v' => {
                        if let PathToken::Num(y) = tokens[idx] {
                            idx += 1;
                            let pt = if cmd == 'V' {
                                Point2D::new(current_pt.x, y)
                            } else {
                                Point2D::new(current_pt.x, current_pt.y + y)
                            };
                            current_pt = pt;
                            current_pts.push(pt);
                            last_cubic_ctrl = None;
                            last_quad_ctrl = None;
                        } else {
                            idx += 1;
                        }
                    }
                    'C' | 'c' => {
                        if idx + 5 < tokens.len() {
                            let mut coords = [0.0; 6];
                            let mut ok = true;
                            for k in 0..6 {
                                if let PathToken::Num(v) = tokens[idx + k] {
                                    coords[k] = v;
                                } else {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                idx += 6;
                                let p0 = current_pt;
                                let (p1, p2, p3) = if cmd == 'C' {
                                    (
                                        Point2D::new(coords[0], coords[1]),
                                        Point2D::new(coords[2], coords[3]),
                                        Point2D::new(coords[4], coords[5]),
                                    )
                                } else {
                                    (
                                        Point2D::new(p0.x + coords[0], p0.y + coords[1]),
                                        Point2D::new(p0.x + coords[2], p0.y + coords[3]),
                                        Point2D::new(p0.x + coords[4], p0.y + coords[5]),
                                    )
                                };

                                let steps = 12;
                                for step in 1..=steps {
                                    let t = step as f64 / steps as f64;
                                    current_pts.push(eval_cubic_bezier(p0, p1, p2, p3, t));
                                }

                                current_pt = p3;
                                last_cubic_ctrl = Some(p2);
                                last_quad_ctrl = None;
                            } else {
                                idx += 1;
                            }
                        } else {
                            idx += 1;
                        }
                    }
                    'S' | 's' => {
                        if idx + 3 < tokens.len() {
                            let mut coords = [0.0; 4];
                            let mut ok = true;
                            for k in 0..4 {
                                if let PathToken::Num(v) = tokens[idx + k] {
                                    coords[k] = v;
                                } else {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                idx += 4;
                                let p0 = current_pt;
                                let p1 = if let Some(prev) = last_cubic_ctrl {
                                    Point2D::new(2.0 * p0.x - prev.x, 2.0 * p0.y - prev.y)
                                } else {
                                    p0
                                };
                                let (p2, p3) = if cmd == 'S' {
                                    (
                                        Point2D::new(coords[0], coords[1]),
                                        Point2D::new(coords[2], coords[3]),
                                    )
                                } else {
                                    (
                                        Point2D::new(p0.x + coords[0], p0.y + coords[1]),
                                        Point2D::new(p0.x + coords[2], p0.y + coords[3]),
                                    )
                                };

                                let steps = 12;
                                for step in 1..=steps {
                                    let t = step as f64 / steps as f64;
                                    current_pts.push(eval_cubic_bezier(p0, p1, p2, p3, t));
                                }

                                current_pt = p3;
                                last_cubic_ctrl = Some(p2);
                                last_quad_ctrl = None;
                            } else {
                                idx += 1;
                            }
                        } else {
                            idx += 1;
                        }
                    }
                    'Q' | 'q' => {
                        if idx + 3 < tokens.len() {
                            let mut coords = [0.0; 4];
                            let mut ok = true;
                            for k in 0..4 {
                                if let PathToken::Num(v) = tokens[idx + k] {
                                    coords[k] = v;
                                } else {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                idx += 4;
                                let p0 = current_pt;
                                let (p1, p2) = if cmd == 'Q' {
                                    (
                                        Point2D::new(coords[0], coords[1]),
                                        Point2D::new(coords[2], coords[3]),
                                    )
                                } else {
                                    (
                                        Point2D::new(p0.x + coords[0], p0.y + coords[1]),
                                        Point2D::new(p0.x + coords[2], p0.y + coords[3]),
                                    )
                                };

                                let steps = 10;
                                for step in 1..=steps {
                                    let t = step as f64 / steps as f64;
                                    current_pts.push(eval_quad_bezier(p0, p1, p2, t));
                                }

                                current_pt = p2;
                                last_quad_ctrl = Some(p1);
                                last_cubic_ctrl = None;
                            } else {
                                idx += 1;
                            }
                        } else {
                            idx += 1;
                        }
                    }
                    'T' | 't' => {
                        if idx + 1 < tokens.len() {
                            if let (PathToken::Num(x), PathToken::Num(y)) = (&tokens[idx], &tokens[idx + 1]) {
                                idx += 2;
                                let p0 = current_pt;
                                let p1 = if let Some(prev) = last_quad_ctrl {
                                    Point2D::new(2.0 * p0.x - prev.x, 2.0 * p0.y - prev.y)
                                } else {
                                    p0
                                };
                                let p2 = if cmd == 'T' {
                                    Point2D::new(*x, *y)
                                } else {
                                    Point2D::new(p0.x + *x, p0.y + *y)
                                };

                                let steps = 10;
                                for step in 1..=steps {
                                    let t = step as f64 / steps as f64;
                                    current_pts.push(eval_quad_bezier(p0, p1, p2, t));
                                }

                                current_pt = p2;
                                last_quad_ctrl = Some(p1);
                                last_cubic_ctrl = None;
                            } else {
                                idx += 1;
                            }
                        } else {
                            idx += 1;
                        }
                    }
                    'A' | 'a' => {
                        // Elliptical Arc: rx ry x-axis-rotation large-arc-flag sweep-flag x y
                        if idx + 6 < tokens.len() {
                            let mut coords = [0.0; 7];
                            let mut ok = true;
                            for k in 0..7 {
                                if let PathToken::Num(v) = tokens[idx + k] {
                                    coords[k] = v;
                                } else {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                idx += 7;
                                let p0 = current_pt;
                                let p_end = if cmd == 'A' {
                                    Point2D::new(coords[5], coords[6])
                                } else {
                                    Point2D::new(p0.x + coords[5], p0.y + coords[6])
                                };

                                // Approximate arc linear segments
                                let steps = 12;
                                for step in 1..=steps {
                                    let t = step as f64 / steps as f64;
                                    let mid_x = p0.x + (p_end.x - p0.x) * t;
                                    let mid_y = p0.y + (p_end.y - p0.y) * t;
                                    current_pts.push(Point2D::new(mid_x, mid_y));
                                }

                                current_pt = p_end;
                                last_cubic_ctrl = None;
                                last_quad_ctrl = None;
                            } else {
                                idx += 1;
                            }
                        } else {
                            idx += 1;
                        }
                    }
                    _ => {
                        idx += 1;
                    }
                }
            }
        }
    }

    finish_subpath(&mut current_pts, false, &mut polylines);
    polylines
}

/// Parses CSS/SVG color string into standardized hex "#RRGGBB"
pub fn parse_color(val: &str) -> Option<String> {
    let s = val.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("none") || s.eq_ignore_ascii_case("transparent") {
        return None;
    }

    if s.starts_with('#') {
        let hex = &s[1..];
        if hex.len() == 3 {
            let r = &hex[0..1];
            let g = &hex[1..2];
            let b = &hex[2..3];
            return Some(format!("#{0}{0}{1}{1}{2}{2}", r, g, b).to_uppercase());
        } else if hex.len() >= 6 {
            return Some(format!("#{}", &hex[0..6]).to_uppercase());
        }
    }

    match s.to_lowercase().as_str() {
        "black" => Some("#000000".to_string()),
        "white" => Some("#FFFFFF".to_string()),
        "red" => Some("#FF0000".to_string()),
        "green" => Some("#00FF00".to_string()),
        "blue" => Some("#0000FF".to_string()),
        "cyan" => Some("#00FFFF".to_string()),
        "magenta" => Some("#FF00FF".to_string()),
        "yellow" => Some("#FFFF00".to_string()),
        "orange" => Some("#FFA500".to_string()),
        "purple" => Some("#800080".to_string()),
        "gray" | "grey" => Some("#808080".to_string()),
        _ => None,
    }
}

/// Extracts stroke or fill color from XML node attributes or style string
fn extract_node_color(node: &roxmltree::Node) -> Option<String> {
    if let Some(stroke) = node.attribute("stroke") {
        if let Some(c) = parse_color(stroke) {
            return Some(c);
        }
    }

    if let Some(style) = node.attribute("style") {
        for prop in style.split(';') {
            let parts: Vec<&str> = prop.split(':').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                if parts[0] == "stroke" {
                    if let Some(c) = parse_color(parts[1]) {
                        return Some(c);
                    }
                } else if parts[0] == "fill" {
                    if let Some(c) = parse_color(parts[1]) {
                        return Some(c);
                    }
                }
            }
        }
    }

    if let Some(fill) = node.attribute("fill") {
        if let Some(c) = parse_color(fill) {
            return Some(c);
        }
    }

    None
}

/// Parses an SVG length attribute (e.g. "100mm", "10in", "50px", "50") to millimeters
fn parse_length_to_mm(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.ends_with("mm") {
        trimmed[..trimmed.len() - 2].trim().parse::<f64>().ok()
    } else if trimmed.ends_with("cm") {
        trimmed[..trimmed.len() - 2].trim().parse::<f64>().ok().map(|v| v * 10.0)
    } else if trimmed.ends_with("in") {
        trimmed[..trimmed.len() - 2].trim().parse::<f64>().ok().map(|v| v * 25.4)
    } else if trimmed.ends_with("pt") {
        trimmed[..trimmed.len() - 2].trim().parse::<f64>().ok().map(|v| v * (25.4 / 72.0))
    } else if trimmed.ends_with("px") {
        trimmed[..trimmed.len() - 2].trim().parse::<f64>().ok().map(|v| v * (25.4 / 96.0))
    } else {
        trimmed.parse::<f64>().ok().map(|v| v * (25.4 / 96.0))
    }
}

pub struct SvgFormat;

impl SvgFormat {
    /// Loads SVG file and extracts all vector paths, shapes, and group transforms
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Document, String> {
        let mut file = File::open(path.as_ref())
            .map_err(|e| format!("Failed to open SVG: {}", e))?;
        let mut xml_data = String::new();
        file.read_to_string(&mut xml_data)
            .map_err(|e| format!("Failed to read SVG: {}", e))?;

        let doc_name = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("SVG_Import");

        Self::load_from_str(&xml_data, doc_name)
    }

    /// Loads SVG from XML string content
    pub fn load_from_str(xml_data: &str, doc_name: &str) -> Result<Document, String> {
        let doc_xml = roxmltree::Document::parse(xml_data)
            .map_err(|e| format!("XML parse error: {}", e))?;

        let mut doc = Document::new(doc_name, 400.0, 400.0);
        let root = doc_xml.root_element();

        // Calculate global scale factor from SVG viewBox or physical width/height
        let mut global_scale = 1.0;
        if let Some(vb) = root.attribute("viewBox") {
            let parts: Vec<f64> = vb
                .split_whitespace()
                .filter_map(|s| s.trim_matches(',').parse::<f64>().ok())
                .collect();
            if parts.len() >= 4 && parts[2] > 0.0 {
                if let Some(w_attr) = root.attribute("width") {
                    if let Some(w_mm) = parse_length_to_mm(w_attr) {
                        global_scale = w_mm / parts[2];
                    }
                }
            }
        }

        let mut layer_map: HashMap<String, usize> = HashMap::new();
        // Layer 0 is default black
        layer_map.insert("#000000".to_string(), 0);

        let mut get_or_create_layer = |doc: &mut Document, hex: &str| -> usize {
            if let Some(&idx) = layer_map.get(hex) {
                return idx;
            }
            let new_idx = doc.layers.len();
            let layer = Layer::new(new_idx, &format!("Layer {}", new_idx), hex);
            doc.layers.insert(new_idx, layer);
            layer_map.insert(hex.to_string(), new_idx);
            new_idx
        };

        // Cache defined symbols for <use> element resolution
        let mut defined_shapes: HashMap<String, Vec<Polyline>> = HashMap::new();

        // Recursive traversal tracking combined affine transforms
        fn traverse_node<'a>(
            node: roxmltree::Node<'a, 'a>,
            parent_tf: Transform2D,
            inherited_color: Option<String>,
            doc: &mut Document,
            global_scale: f64,
            defined_shapes: &mut HashMap<String, Vec<Polyline>>,
            get_or_create_layer: &mut dyn FnMut(&mut Document, &str) -> usize,
        ) {
            let node_tf = node
                .attribute("transform")
                .map(Transform2D::parse)
                .unwrap_or_else(Transform2D::identity);
            let combined_tf = parent_tf.multiply(&node_tf);

            let node_color = extract_node_color(&node).or(inherited_color.clone());

            let mut node_polylines: Vec<Polyline> = Vec::new();

            match node.tag_name().name() {
                "path" => {
                    if let Some(d) = node.attribute("d") {
                        let polys = parse_path_d(d);
                        for poly in polys {
                            let transformed_pts: Vec<Point2D> = poly
                                .points
                                .iter()
                                .map(|p| {
                                    let scaled = Point2D::new(p.x * global_scale, p.y * global_scale);
                                    combined_tf.apply(scaled)
                                })
                                .collect();
                            if transformed_pts.len() >= 2 {
                                node_polylines.push(Polyline::new(transformed_pts, poly.is_closed));
                            }
                        }
                    }
                }
                "rect" => {
                    let x = node.attribute("x").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let y = node.attribute("y").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let w = node.attribute("width").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let h = node.attribute("height").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;

                    if w > 0.0 && h > 0.0 {
                        let corners = vec![
                            combined_tf.apply(Point2D::new(x, y)),
                            combined_tf.apply(Point2D::new(x + w, y)),
                            combined_tf.apply(Point2D::new(x + w, y + h)),
                            combined_tf.apply(Point2D::new(x, y + h)),
                        ];
                        node_polylines.push(Polyline::new(corners, true));
                    }
                }
                "circle" => {
                    let cx = node.attribute("cx").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let cy = node.attribute("cy").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let r = node.attribute("r").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;

                    if r > 0.0 {
                        let segments = 36;
                        let mut pts = Vec::with_capacity(segments);
                        for i in 0..segments {
                            let a = (i as f64 / segments as f64) * std::f64::consts::TAU;
                            let p = Point2D::new(cx + r * a.cos(), cy + r * a.sin());
                            pts.push(combined_tf.apply(p));
                        }
                        node_polylines.push(Polyline::new(pts, true));
                    }
                }
                "ellipse" => {
                    let cx = node.attribute("cx").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let cy = node.attribute("cy").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let rx = node.attribute("rx").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let ry = node.attribute("ry").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;

                    if rx > 0.0 && ry > 0.0 {
                        let segments = 36;
                        let mut pts = Vec::with_capacity(segments);
                        for i in 0..segments {
                            let a = (i as f64 / segments as f64) * std::f64::consts::TAU;
                            let p = Point2D::new(cx + rx * a.cos(), cy + ry * a.sin());
                            pts.push(combined_tf.apply(p));
                        }
                        node_polylines.push(Polyline::new(pts, true));
                    }
                }
                "line" => {
                    let x1 = node.attribute("x1").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let y1 = node.attribute("y1").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let x2 = node.attribute("x2").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;
                    let y2 = node.attribute("y2").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * global_scale;

                    let p1 = combined_tf.apply(Point2D::new(x1, y1));
                    let p2 = combined_tf.apply(Point2D::new(x2, y2));
                    node_polylines.push(Polyline::new(vec![p1, p2], false));
                }
                "polyline" | "polygon" => {
                    let is_closed = node.tag_name().name() == "polygon";
                    if let Some(pts_str) = node.attribute("points") {
                        let mut pts = Vec::new();
                        for pair in pts_str.split_whitespace() {
                            let coords: Vec<&str> = pair.split(',').collect();
                            if coords.len() == 2 {
                                if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                                    let p = Point2D::new(x * global_scale, y * global_scale);
                                    pts.push(combined_tf.apply(p));
                                }
                            }
                        }
                        if pts.len() >= 2 {
                            node_polylines.push(Polyline::new(pts, is_closed));
                        }
                    }
                }
                "use" => {
                    let href = node.attribute("href").or_else(|| node.attribute("xlink:href"));
                    if let Some(target_id) = href.and_then(|h| h.strip_prefix('#')) {
                        if let Some(cached) = defined_shapes.get(target_id) {
                            for poly in cached {
                                let transformed_pts: Vec<Point2D> = poly
                                    .points
                                    .iter()
                                    .map(|p| combined_tf.apply(*p))
                                    .collect();
                                if transformed_pts.len() >= 2 {
                                    node_polylines.push(Polyline::new(transformed_pts, poly.is_closed));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            if let Some(id) = node.attribute("id") {
                if !node_polylines.is_empty() {
                    defined_shapes.insert(id.to_string(), node_polylines.clone());
                }
            }

            // Assign polylines to shapes with layer color
            let layer_idx = if let Some(ref c) = node_color {
                get_or_create_layer(doc, c)
            } else {
                0
            };

            for poly in node_polylines {
                doc.add_shape(Shape::Path {
                    layer_index: layer_idx,
                    polyline: poly,
                });
            }

            // Recurse into children
            for child in node.children() {
                if child.is_element() {
                    traverse_node(
                        child,
                        combined_tf,
                        node_color.clone(),
                        doc,
                        global_scale,
                        defined_shapes,
                        get_or_create_layer,
                    );
                }
            }
        }

        // Run traversal from root element
        for child in root.children() {
            if child.is_element() {
                traverse_node(
                    child,
                    Transform2D::identity(),
                    None,
                    &mut doc,
                    global_scale,
                    &mut defined_shapes,
                    &mut get_or_create_layer,
                );
            }
        }

        // Bed bounds normalization: ensure shapes are positioned within positive visible bed area
        if !doc.shapes.is_empty() {
            let bounds = doc.bounds();
            if bounds.min_x < 0.0 || bounds.min_y < 0.0 {
                let dx = if bounds.min_x < 0.0 { -bounds.min_x + 10.0 } else { 0.0 };
                let dy = if bounds.min_y < 0.0 { -bounds.min_y + 10.0 } else { 0.0 };
                for s in &mut doc.shapes {
                    s.translate(dx, dy);
                }
            }
            let updated_bounds = doc.bounds();
            doc.width = doc.width.max(updated_bounds.max_x + 20.0).max(400.0);
            doc.height = doc.height.max(updated_bounds.max_y + 20.0).max(400.0);
        }

        Ok(doc)
    }

    /// Exports Document to standard SVG format
    pub fn save_to_file<P: AsRef<Path>>(doc: &Document, path: P) -> Result<(), String> {
        let mut out = String::new();
        out.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{:.1}mm" height="{:.1}mm" viewBox="0 0 {:.1} {:.1}">"#,
            doc.width, doc.height, doc.width, doc.height
        ));
        out.push('\n');

        for shape in &doc.shapes {
            let layer = doc.layers.get(&shape.layer_index());
            let color = layer.map(|l| l.color_hex.as_str()).unwrap_or("#000000");

            for poly in shape.to_polylines() {
                if poly.points.is_empty() {
                    continue;
                }
                out.push_str(&format!(
                    r#"  <path d="M {:.3} {:.3}"#,
                    poly.points[0].x, poly.points[0].y
                ));
                for pt in &poly.points[1..] {
                    out.push_str(&format!(r#" L {:.3} {:.3}"#, pt.x, pt.y));
                }
                if poly.is_closed {
                    out.push_str(r#" Z""#);
                } else {
                    out.push('"');
                }
                out.push_str(&format!(
                    r#" fill="none" stroke="{}" stroke-width="0.2"/>"#,
                    color
                ));
                out.push('\n');
            }
        }

        out.push_str("</svg>\n");

        let mut file = File::create(path.as_ref())
            .map_err(|e| format!("Failed to create SVG: {}", e))?;
        file.write_all(out.as_bytes())
            .map_err(|e| format!("Failed to write SVG: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_path_d_tokenization() {
        let d = "M10,20L30.5,40.5C50 60 70 80 90 100Z";
        let tokens = tokenize_path_d(d);
        assert_eq!(tokens[0], PathToken::Cmd('M'));
        assert_eq!(tokens[1], PathToken::Num(10.0));
        assert_eq!(tokens[2], PathToken::Num(20.0));
        assert_eq!(tokens[3], PathToken::Cmd('L'));
        assert_eq!(tokens[4], PathToken::Num(30.5));
        assert_eq!(tokens[5], PathToken::Num(40.5));
        assert_eq!(tokens[6], PathToken::Cmd('C'));
        assert_eq!(tokens.last(), Some(&PathToken::Cmd('Z')));
    }

    #[test]
    fn test_svg_path_d_parsing() {
        let d = "M 0 0 L 10 0 L 10 10 L 0 10 Z";
        let polys = parse_path_d(d);
        assert_eq!(polys.len(), 1);
        assert!(polys[0].is_closed);
        assert_eq!(polys[0].points.len(), 5);
    }

    #[test]
    fn test_svg_transform_parsing() {
        let tf = Transform2D::parse("translate(10, 20) scale(2, 2)");
        let pt = tf.apply(Point2D::new(5.0, 5.0));
        assert!((pt.x - 20.0).abs() < 1e-4);
        assert!((pt.y - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_load_rust_logo_svg() {
        let path = "/Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/target/doc/static.files/rust-logo-9a9549ea.svg";
        if Path::new(path).exists() {
            let doc = SvgFormat::load_from_file(path).expect("Failed to load rust-logo.svg");
            assert!(!doc.shapes.is_empty(), "Expected shapes parsed from rust-logo.svg");
            println!("Successfully parsed {} shapes from Rust logo SVG!", doc.shapes.len());
        }
    }
}
