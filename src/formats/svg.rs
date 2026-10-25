use crate::document::{Document, Shape};
use crate::geometry::{Point2D, Polyline};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub struct SvgFormat;

impl SvgFormat {
    /// Loads SVG file and extracts basic vector paths
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Document, String> {
        let mut file = File::open(path.as_ref())
            .map_err(|e| format!("Failed to open SVG: {}", e))?;
        let mut xml_data = String::new();
        file.read_to_string(&mut xml_data)
            .map_err(|e| format!("Failed to read SVG: {}", e))?;

        let doc_xml = roxmltree::Document::parse(&xml_data)
            .map_err(|e| format!("XML parse error: {}", e))?;

        let mut doc = Document::new(
            path.as_ref()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("SVG_Import"),
            400.0,
            400.0,
        );

        let root = doc_xml.root_element();

        for node in root.descendants() {
            match node.tag_name().name() {
                "rect" => {
                    let x = node.attribute("x").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let y = node.attribute("y").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let w = node.attribute("width").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let h = node.attribute("height").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let rx = node.attribute("rx").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

                    if w > 0.0 && h > 0.0 {
                        doc.add_shape(Shape::Rect {
                            layer_index: 0,
                            x,
                            y,
                            width: w,
                            height: h,
                            corner_radius: rx,
                        });
                    }
                }
                "circle" => {
                    let cx = node.attribute("cx").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let cy = node.attribute("cy").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let r = node.attribute("r").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

                    if r > 0.0 {
                        doc.add_shape(Shape::Circle {
                            layer_index: 0,
                            center: Point2D::new(cx, cy),
                            radius: r,
                        });
                    }
                }
                "polyline" | "polygon" => {
                    let is_closed = node.tag_name().name() == "polygon";
                    if let Some(pts_str) = node.attribute("points") {
                        let mut pts = Vec::new();
                        for pair in pts_str.split_whitespace() {
                            let coords: Vec<&str> = pair.split(',').collect();
                            if coords.len() == 2 {
                                if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                                    pts.push(Point2D::new(x, y));
                                }
                            }
                        }
                        if pts.len() >= 2 {
                            doc.add_shape(Shape::Path {
                                layer_index: 0,
                                polyline: Polyline::new(pts, is_closed),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(doc)
    }

    /// Exports Document to standard SVG
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
