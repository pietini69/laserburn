//! PDF vector graphics parser and importer for LaserBurn.
//! Extracts vector cutting and engraving contours (m, l, c, v, y, re, h, S, f)
//! from compressed (/FlateDecode) and uncompressed PDF content streams, converts PDF points
//! to millimeters (1 pt = 25.4 / 72 mm), maps stroke/fill colors to layers, and flips Y upright.

use crate::document::{CutMode, Document, Layer, Shape};
use crate::geometry::{BoundingBox, Point2D, Polyline};
use std::path::Path;

pub struct PdfFormat;

#[derive(Debug, Clone, Copy)]
struct Affine2D {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Affine2D {
    fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    fn transform(&self, p: Point2D) -> Point2D {
        Point2D::new(
            self.a * p.x + self.c * p.y + self.e,
            self.b * p.x + self.d * p.y + self.f,
        )
    }

    fn concat(&self, other: &Affine2D) -> Self {
        Self {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            e: self.e * other.a + self.f * other.c + other.e,
            f: self.e * other.b + self.f * other.d + other.f,
        }
    }
}

impl PdfFormat {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Document, String> {
        let bytes = std::fs::read(path.as_ref())
            .map_err(|e| format!("Failed to read PDF file: {}", e))?;
        Self::load_from_bytes(&bytes)
    }

    pub fn load_from_bytes(bytes: &[u8]) -> Result<Document, String> {
        let max_hdr = bytes.len().min(1024);
        if bytes.len() < 8 || find_subsequence(&bytes[..max_hdr], b"%PDF-").is_none() {
            return Err("Not a valid PDF file (missing %PDF- header)".to_string());
        }

        // 1. Extract and decompress all content streams
        let content_streams = Self::extract_streams(bytes);
        if content_streams.is_empty() {
            return Err("No content streams found in PDF".to_string());
        }

        let mut doc = Document::new("PDF Import", 400.0, 400.0);
        let mut raw_shapes = Vec::new();
        let mut bbox = BoundingBox::empty();

        // Conversion factor from PDF points (1/72 inch) to millimeters
        let pt_to_mm = 25.4 / 72.0;

        for stream in &content_streams {
            let shapes = Self::parse_stream_vector_shapes(stream, pt_to_mm);
            for s in shapes {
                bbox = bbox.union(&s.bounds());
                raw_shapes.push(s);
            }
        }

        if raw_shapes.is_empty() {
            return Err("No vector paths found in PDF content streams (file might contain only scanned raster images or text fonts)".to_string());
        }

        // PDF coordinates have (0, 0) at bottom-left with Y+ going upwards.
        // We flip Y around the vertical center of the bounding box to make it upright in LaserBurn.
        let flip_center_y = bbox.center().y;
        let offset_x = (20.0 - bbox.min_x).max(0.0);
        let offset_y = (20.0 - bbox.min_y).max(0.0);

        for mut s in raw_shapes {
            s.scale(Point2D::new(0.0, flip_center_y), 1.0, -1.0);
            s.translate(offset_x, offset_y);
            let layer_idx = s.layer_index();
            if !doc.layers.contains_key(&layer_idx) {
                let mut layer = Layer::new(layer_idx, &format!("PDF Layer {:02}", layer_idx), "#000000");
                layer.cut_setting.mode = CutMode::Line;
                layer.cut_setting.speed = 20.0;
                layer.cut_setting.power_max = 80.0;
                doc.layers.insert(layer_idx, layer);
            }
            doc.add_shape(s);
        }

        Ok(doc)
    }

    /// Locates and inflates content streams from the PDF byte buffer
    fn extract_streams(bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut streams = Vec::new();
        let mut pos = 0;

        while let Some(stream_idx) = find_subsequence(&bytes[pos..], b"stream") {
            let abs_stream = pos + stream_idx;
            // The stream keyword must be followed by newline (\r\n or \n)
            let mut data_start = abs_stream + 6;
            if data_start < bytes.len() && bytes[data_start] == b'\r' {
                data_start += 1;
            }
            if data_start < bytes.len() && bytes[data_start] == b'\n' {
                data_start += 1;
            }

            // Find endstream
            if let Some(end_idx) = find_subsequence(&bytes[data_start..], b"endstream") {
                let data_end = data_start + end_idx;
                let raw_stream = &bytes[data_start..data_end];

                let mut clean_data = raw_stream;
                while !clean_data.is_empty() && (clean_data[clean_data.len() - 1] == b'\n' || clean_data[clean_data.len() - 1] == b'\r') {
                    clean_data = &clean_data[..clean_data.len() - 1];
                }

                // Check preceding dictionary for /FlateDecode
                let dict_window_start = abs_stream.saturating_sub(600);
                let dict_window = &bytes[dict_window_start..abs_stream];
                let is_flate = find_subsequence(dict_window, b"/FlateDecode").is_some()
                    || find_subsequence(dict_window, b"/Fl").is_some();

                let mut decompressed_opt = None;
                if is_flate
                    || clean_data.starts_with(&[0x78, 0x9c])
                    || clean_data.starts_with(&[0x78, 0x01])
                    || clean_data.starts_with(&[0x78, 0xda])
                {
                    if let Ok(decompressed) = miniz_oxide::inflate::decompress_to_vec_zlib(clean_data) {
                        decompressed_opt = Some(decompressed);
                    } else if let Ok(decompressed) = miniz_oxide::inflate::decompress_to_vec(clean_data) {
                        decompressed_opt = Some(decompressed);
                    } else if let Ok(decompressed) = miniz_oxide::inflate::decompress_to_vec_zlib(raw_stream) {
                        decompressed_opt = Some(decompressed);
                    } else if let Ok(decompressed) = miniz_oxide::inflate::decompress_to_vec(raw_stream) {
                        decompressed_opt = Some(decompressed);
                    }
                }

                if let Some(decompressed) = decompressed_opt {
                    streams.push(decompressed);
                } else if !is_flate {
                    streams.push(clean_data.to_vec());
                }

                pos = data_end + 9;
            } else {
                break;
            }
        }

        streams
    }

    /// Parses vector drawing operators from an uncompressed PDF content stream
    fn parse_stream_vector_shapes(stream_bytes: &[u8], scale: f64) -> Vec<Shape> {
        let text = String::from_utf8_lossy(stream_bytes);
        let tokens: Vec<&str> = text.split_whitespace().collect();

        let mut shapes = Vec::new();
        let mut stack: Vec<f64> = Vec::new();
        let mut matrix_stack: Vec<Affine2D> = vec![Affine2D::identity()];

        let mut current_pt = Point2D::new(0.0, 0.0);
        let mut current_subpath = Vec::new();
        let mut active_subpaths: Vec<(Vec<Point2D>, bool)> = Vec::new();
        let mut active_layer_idx = 0usize;

        let mut i = 0;
        while i < tokens.len() {
            let tok = tokens[i];

            // If it's a numeric operand, push to stack
            if let Ok(num) = tok.parse::<f64>() {
                stack.push(num);
                i += 1;
                continue;
            }

            let cur_matrix = *matrix_stack.last().unwrap();

            match tok {
                "q" => {
                    matrix_stack.push(cur_matrix);
                    stack.clear();
                }
                "Q" => {
                    if matrix_stack.len() > 1 {
                        matrix_stack.pop();
                    }
                    stack.clear();
                }
                "cm" => {
                    if stack.len() >= 6 {
                        let f = stack.pop().unwrap();
                        let e = stack.pop().unwrap();
                        let d = stack.pop().unwrap();
                        let c = stack.pop().unwrap();
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        let m = Affine2D { a, b, c, d, e, f };
                        let top = matrix_stack.last_mut().unwrap();
                        *top = top.concat(&m);
                    }
                    stack.clear();
                }
                // Color operators: rg / RG (RGB)
                "rg" | "RG" => {
                    if stack.len() >= 3 {
                        let b = stack.pop().unwrap();
                        let g = stack.pop().unwrap();
                        let r = stack.pop().unwrap();
                        active_layer_idx = map_rgb_to_layer(r, g, b);
                    }
                    stack.clear();
                }
                // Color operator: k / K (CMYK)
                "k" | "K" => {
                    if stack.len() >= 4 {
                        let _k = stack.pop().unwrap();
                        let y = stack.pop().unwrap();
                        let m = stack.pop().unwrap();
                        let c = stack.pop().unwrap();
                        active_layer_idx = map_rgb_to_layer(1.0 - c, 1.0 - m, 1.0 - y);
                    }
                    stack.clear();
                }
                // Color operator: g / G (Grayscale)
                "g" | "G" => {
                    if stack.len() >= 1 {
                        let _gray = stack.pop().unwrap();
                        active_layer_idx = 0;
                    }
                    stack.clear();
                }
                // Move To: x y m
                "m" => {
                    if stack.len() >= 2 {
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        if !current_subpath.is_empty() {
                            active_subpaths.push((std::mem::take(&mut current_subpath), false));
                        }
                        let transformed = cur_matrix.transform(Point2D::new(x * scale, y * scale));
                        current_pt = transformed;
                        current_subpath.push(transformed);
                    }
                    stack.clear();
                }
                // Line To: x y l
                "l" => {
                    if stack.len() >= 2 {
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();
                        let transformed = cur_matrix.transform(Point2D::new(x * scale, y * scale));
                        current_pt = transformed;
                        current_subpath.push(transformed);
                    }
                    stack.clear();
                }
                // Cubic Bézier Curve To: x1 y1 x2 y2 x3 y3 c
                "c" => {
                    if stack.len() >= 6 {
                        let y3 = stack.pop().unwrap();
                        let x3 = stack.pop().unwrap();
                        let y2 = stack.pop().unwrap();
                        let x2 = stack.pop().unwrap();
                        let y1 = stack.pop().unwrap();
                        let x1 = stack.pop().unwrap();

                        let p0 = current_pt;
                        let p1 = cur_matrix.transform(Point2D::new(x1 * scale, y1 * scale));
                        let p2 = cur_matrix.transform(Point2D::new(x2 * scale, y2 * scale));
                        let p3 = cur_matrix.transform(Point2D::new(x3 * scale, y3 * scale));

                        let steps = 8;
                        for s in 1..=steps {
                            let t = s as f64 / steps as f64;
                            let mt = 1.0 - t;
                            let px = mt * mt * mt * p0.x + 3.0 * mt * mt * t * p1.x + 3.0 * mt * t * t * p2.x + t * t * t * p3.x;
                            let py = mt * mt * mt * p0.y + 3.0 * mt * mt * t * p1.y + 3.0 * mt * t * t * p2.y + t * t * t * p3.y;
                            current_subpath.push(Point2D::new(px, py));
                        }
                        current_pt = p3;
                    }
                    stack.clear();
                }
                // Cubic Bézier Curve To (v): x2 y2 x3 y3 v
                "v" => {
                    if stack.len() >= 4 {
                        let y3 = stack.pop().unwrap();
                        let x3 = stack.pop().unwrap();
                        let y2 = stack.pop().unwrap();
                        let x2 = stack.pop().unwrap();

                        let p0 = current_pt;
                        let p1 = p0;
                        let p2 = cur_matrix.transform(Point2D::new(x2 * scale, y2 * scale));
                        let p3 = cur_matrix.transform(Point2D::new(x3 * scale, y3 * scale));

                        let steps = 8;
                        for s in 1..=steps {
                            let t = s as f64 / steps as f64;
                            let mt = 1.0 - t;
                            let px = mt * mt * mt * p0.x + 3.0 * mt * mt * t * p1.x + 3.0 * mt * t * t * p2.x + t * t * t * p3.x;
                            let py = mt * mt * mt * p0.y + 3.0 * mt * mt * t * p1.y + 3.0 * mt * t * t * p2.y + t * t * t * p3.y;
                            current_subpath.push(Point2D::new(px, py));
                        }
                        current_pt = p3;
                    }
                    stack.clear();
                }
                // Cubic Bézier Curve To (y): x1 y1 x3 y3 y
                "y" => {
                    if stack.len() >= 4 {
                        let y3 = stack.pop().unwrap();
                        let x3 = stack.pop().unwrap();
                        let y1 = stack.pop().unwrap();
                        let x1 = stack.pop().unwrap();

                        let p0 = current_pt;
                        let p1 = cur_matrix.transform(Point2D::new(x1 * scale, y1 * scale));
                        let p3 = cur_matrix.transform(Point2D::new(x3 * scale, y3 * scale));
                        let p2 = p3;

                        let steps = 8;
                        for s in 1..=steps {
                            let t = s as f64 / steps as f64;
                            let mt = 1.0 - t;
                            let px = mt * mt * mt * p0.x + 3.0 * mt * mt * t * p1.x + 3.0 * mt * t * t * p2.x + t * t * t * p3.x;
                            let py = mt * mt * mt * p0.y + 3.0 * mt * mt * t * p1.y + 3.0 * mt * t * t * p2.y + t * t * t * p3.y;
                            current_subpath.push(Point2D::new(px, py));
                        }
                        current_pt = p3;
                    }
                    stack.clear();
                }
                // Rectangle: x y w h re
                "re" => {
                    if stack.len() >= 4 {
                        let h = stack.pop().unwrap();
                        let w = stack.pop().unwrap();
                        let y = stack.pop().unwrap();
                        let x = stack.pop().unwrap();

                        let p0 = cur_matrix.transform(Point2D::new(x * scale, y * scale));
                        let p1 = cur_matrix.transform(Point2D::new((x + w) * scale, y * scale));
                        let p2 = cur_matrix.transform(Point2D::new((x + w) * scale, (y + h) * scale));
                        let p3 = cur_matrix.transform(Point2D::new(x * scale, (y + h) * scale));

                        active_subpaths.push((vec![p0, p1, p2, p3], true));
                    }
                    stack.clear();
                }
                // Close Subpath: h
                "h" => {
                    if !current_subpath.is_empty() {
                        active_subpaths.push((std::mem::take(&mut current_subpath), true));
                    }
                    stack.clear();
                }
                // Paint path: S (stroke), s (close & stroke), f / F / f* (fill), B / b / B* / b* (fill & stroke)
                "S" | "s" | "f" | "F" | "f*" | "b" | "B" | "b*" | "B*" => {
                    let should_close = tok == "s" || tok == "b" || tok == "b*";
                    if !current_subpath.is_empty() {
                        active_subpaths.push((std::mem::take(&mut current_subpath), should_close));
                    }

                    for (pts, is_closed) in std::mem::take(&mut active_subpaths) {
                        if pts.len() >= 2 {
                            let poly = Polyline::new(pts, is_closed || should_close);
                            shapes.push(Shape::Path {
                                layer_index: active_layer_idx,
                                polyline: poly,
                            });
                        }
                    }
                    stack.clear();
                }
                "W" | "W*" => {
                    // Clipping path operator in PDF
                    stack.clear();
                }
                "n" => {
                    // Discard path without painting
                    current_subpath.clear();
                    active_subpaths.clear();
                    stack.clear();
                }
                _ => {
                    // Unrecognized operator; clear operand stack to avoid pollution
                    stack.clear();
                }
            }

            i += 1;
        }

        shapes
    }
}

fn map_rgb_to_layer(r: f64, g: f64, b: f64) -> usize {
    // 0: Black, 1: Blue, 2: Red, 3: Green, 4: Yellow/Orange, 5: Cyan, 6: Magenta
    if r < 0.2 && g < 0.2 && b < 0.2 {
        0 // Black
    } else if r > 0.7 && g < 0.3 && b < 0.3 {
        2 // Red
    } else if r < 0.3 && g < 0.3 && b > 0.7 {
        1 // Blue
    } else if r < 0.3 && g > 0.7 && b < 0.3 {
        3 // Green
    } else if r > 0.7 && g > 0.7 && b < 0.3 {
        4 // Yellow
    } else if r < 0.3 && g > 0.7 && b > 0.7 {
        5 // Cyan
    } else if r > 0.7 && g < 0.3 && b > 0.7 {
        6 // Magenta
    } else {
        0
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_uncompressed_vector_stream() {
        let pdf_data = b"%PDF-1.4\n\
1 0 obj\n\
<< /Length 85 >>\n\
stream\n\
1 0 0 rg\n\
10 10 100 50 re\n\
B\n\
0 0 1 RG\n\
20 20 m 80 80 l S\n\
endstream\n\
endobj\n\
%%EOF";

        let doc = PdfFormat::load_from_bytes(pdf_data).expect("PDF should parse vector shapes");
        assert_eq!(doc.shapes.len(), 2);
    }

    #[test]
    fn test_sample_vector_pdf_loading() {
        let pdf_data = include_bytes!("../../samples/sample_vector.pdf");
        let doc = PdfFormat::load_from_bytes(pdf_data).expect("Sample vector PDF must load");
        assert!(doc.shapes.len() >= 6, "PDF has rectangles, lines, and curves");
        assert!(doc.layers.len() >= 2, "PDF has multiple color layers");
    }
}
