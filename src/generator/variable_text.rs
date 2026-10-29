use crate::geometry::Point2D;
use crate::text::{default_system_font, generate_text_polylines, FontType, TextAlign, TextOptions};
use crate::Shape;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableTextMode {
    /// Incremental numeric sequence (e.g. SN-001 ... SN-050)
    Sequential,
    /// Line-separated custom text list
    CustomList,
    /// Multi-column CSV with custom formatting template
    CsvTemplate,
}

impl Default for VariableTextMode {
    fn default() -> Self {
        VariableTextMode::Sequential
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTextParams {
    pub mode: VariableTextMode,

    // --- Sequential Options ---
    pub prefix: String,
    pub start_num: i32,
    pub end_num: i32,
    pub step: i32,
    pub padding: usize,
    pub suffix: String,

    // --- Custom List & CSV Options ---
    /// Raw lines of names or CSV rows
    pub raw_data: String,
    /// Template string for CSV (e.g. "{1} - {2}" or "{name} #{id}")
    pub csv_template: String,

    // --- Layout & Typography ---
    pub font_size_mm: f64,
    pub use_single_line_font: bool,
    pub font_path: String,
    pub start_x: f64,
    pub start_y: f64,
    pub columns: usize,
    pub spacing_x: f64,
    pub spacing_y: f64,

    // --- Optional Badge / Plaque Enclosure ---
    /// Generate an outer cutting boundary plaque around each text item
    pub add_badge: bool,
    pub badge_width: f64,
    pub badge_height: f64,
    pub badge_corner_radius: f64,
    pub add_mounting_hole: bool,
    pub hole_diameter: f64,
}

impl Default for VariableTextParams {
    fn default() -> Self {
        Self {
            mode: VariableTextMode::Sequential,
            prefix: "SN-".to_string(),
            start_num: 1,
            end_num: 10,
            step: 1,
            padding: 3,
            suffix: "".to_string(),

            raw_data: "Matti Meikäläinen\nMaija Mallikas\nTeppo Testaaja".to_string(),
            csv_template: "{1}".to_string(),

            font_size_mm: 7.0,
            use_single_line_font: true,
            font_path: default_system_font(),
            start_x: 20.0,
            start_y: 20.0,
            columns: 4,
            spacing_x: 10.0,
            spacing_y: 10.0,

            add_badge: true,
            badge_width: 60.0,
            badge_height: 25.0,
            badge_corner_radius: 3.0,
            add_mounting_hole: true,
            hole_diameter: 3.5,
        }
    }
}

/// Generates list of resolved strings according to the configuration
pub fn resolve_variable_strings(params: &VariableTextParams) -> Vec<String> {
    match params.mode {
        VariableTextMode::Sequential => {
            let mut list = Vec::new();
            let start = params.start_num;
            let end = params.end_num;
            let step = params.step.max(1);

            let mut curr = start;
            let count_limit = 500; // Safety cap
            let mut count = 0;

            while curr <= end && count < count_limit {
                let num_str = if params.padding > 1 {
                    format!("{:0width$}", curr, width = params.padding)
                } else {
                    format!("{}", curr)
                };
                list.push(format!("{}{}{}", params.prefix, num_str, params.suffix));
                curr += step;
                count += 1;
            }
            list
        }

        VariableTextMode::CustomList => {
            params
                .raw_data
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }

        VariableTextMode::CsvTemplate => {
            let mut list = Vec::new();
            for line in params.raw_data.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Split by comma or semicolon
                let fields: Vec<&str> = trimmed.split([',', ';']).map(|f| f.trim()).collect();
                let mut out = params.csv_template.clone();
                for (idx, field) in fields.iter().enumerate() {
                    let placeholder1 = format!("{{{}}}", idx + 1);
                    out = out.replace(&placeholder1, field);
                    let placeholder0 = format!("{{{}}}", idx);
                    out = out.replace(&placeholder0, field);
                }
                list.push(out);
            }
            list
        }
    }
}

/// Generates laser cut/engrave shapes for the variable text batch
pub fn generate_variable_text_shapes(
    params: &VariableTextParams,
    text_layer: usize,
    cut_layer: usize,
) -> Vec<Shape> {
    let strings = resolve_variable_strings(params);
    let mut all_shapes = Vec::new();

    let cols = params.columns.max(1);
    let cell_w = if params.add_badge {
        params.badge_width + params.spacing_x
    } else {
        60.0 + params.spacing_x
    };
    let cell_h = if params.add_badge {
        params.badge_height + params.spacing_y
    } else {
        params.font_size_mm * 2.0 + params.spacing_y
    };

    let font_type = if params.use_single_line_font {
        FontType::SingleLine
    } else {
        FontType::TrueType {
            path: if params.font_path.is_empty() {
                default_system_font()
            } else {
                params.font_path.clone()
            },
        }
    };

    for (idx, text_str) in strings.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;

        let origin_x = params.start_x + (col as f64) * cell_w;
        let origin_y = params.start_y + (row as f64) * cell_h;

        let mut item_shapes = Vec::new();

        // 1. Badge Outer Cut & Mounting Hole (if enabled)
        if params.add_badge {
            item_shapes.push(Shape::Rect {
                layer_index: cut_layer,
                x: origin_x,
                y: origin_y,
                width: params.badge_width,
                height: params.badge_height,
                corner_radius: params.badge_corner_radius,
            });

            if params.add_mounting_hole && params.hole_diameter > 0.5 {
                let hole_radius = params.hole_diameter * 0.5;
                // Place hole on left margin of plaque
                let hole_cx = origin_x + hole_radius + 3.0;
                let hole_cy = origin_y + params.badge_height * 0.5;
                item_shapes.push(Shape::Circle {
                    layer_index: cut_layer,
                    center: Point2D::new(hole_cx, hole_cy),
                    radius: hole_radius,
                });
            }
        }

        // 2. Text Vectorization
        let text_center_x = if params.add_badge {
            if params.add_mounting_hole {
                origin_x + params.badge_width * 0.55
            } else {
                origin_x + params.badge_width * 0.5
            }
        } else {
            origin_x
        };

        let text_center_y = if params.add_badge {
            origin_y + params.badge_height * 0.5 + params.font_size_mm * 0.35
        } else {
            origin_y + params.font_size_mm
        };

        let text_options = TextOptions {
            text: text_str.clone(),
            font_type: font_type.clone(),
            font_size_mm: params.font_size_mm,
            x: text_center_x,
            y: text_center_y,
            align: if params.add_badge { TextAlign::Center } else { TextAlign::Left },
            letter_spacing: 1.0,
            line_spacing: 1.2,
        };

        if let Ok(polylines) = generate_text_polylines(&text_options) {
            for polyline in polylines {
                item_shapes.push(Shape::Path {
                    layer_index: text_layer,
                    polyline,
                });
            }
        }

        // Group together
        if !item_shapes.is_empty() {
            all_shapes.push(Shape::Group {
                layer_index: text_layer,
                shapes: item_shapes,
                locked: false,
            });
        }
    }

    all_shapes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_variable_strings() {
        let params = VariableTextParams {
            mode: VariableTextMode::Sequential,
            prefix: "ID-".to_string(),
            start_num: 1,
            end_num: 5,
            step: 1,
            padding: 3,
            suffix: "".to_string(),
            ..Default::default()
        };

        let list = resolve_variable_strings(&params);
        assert_eq!(list, vec!["ID-001", "ID-002", "ID-003", "ID-004", "ID-005"]);
    }

    #[test]
    fn test_csv_template_strings() {
        let params = VariableTextParams {
            mode: VariableTextMode::CsvTemplate,
            raw_data: "Desk 1, Lab A\nDesk 2, Lab B".to_string(),
            csv_template: "{1} ({2})".to_string(),
            ..Default::default()
        };

        let list = resolve_variable_strings(&params);
        assert_eq!(list, vec!["Desk 1 (Lab A)", "Desk 2 (Lab B)"]);
    }

    #[test]
    fn test_variable_text_generates_shapes() {
        let params = VariableTextParams {
            mode: VariableTextMode::Sequential,
            start_num: 1,
            end_num: 3,
            use_single_line_font: true,
            add_badge: true,
            ..Default::default()
        };

        let shapes = generate_variable_text_shapes(&params, 0, 1);
        assert_eq!(shapes.len(), 3);
    }
}
