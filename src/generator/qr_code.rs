use crate::geometry::Point2D;
use crate::Shape;
use qrcode::{Color, EcLevel, QrCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QrEcLevel {
    /// Low (~7% recovery)
    Low,
    /// Medium (~15% recovery)
    Medium,
    /// Quartile (~25% recovery)
    Quartile,
    /// High (~30% recovery)
    High,
}

impl Default for QrEcLevel {
    fn default() -> Self {
        QrEcLevel::Medium
    }
}

impl From<QrEcLevel> for EcLevel {
    fn from(l: QrEcLevel) -> Self {
        match l {
            QrEcLevel::Low => EcLevel::L,
            QrEcLevel::Medium => EcLevel::M,
            QrEcLevel::Quartile => EcLevel::Q,
            QrEcLevel::High => EcLevel::H,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QrStyle {
    /// Sharp vector squares (optimal, run-length merged horizontally)
    SharpSquares,
    /// Modern rounded vector squares
    RoundedSquares,
    /// Circular dots
    Dots,
}

impl Default for QrStyle {
    fn default() -> Self {
        QrStyle::SharpSquares
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeParams {
    /// Content text, URL, vCard, or Wi-Fi string
    pub content: String,
    /// Position X (mm)
    pub x: f64,
    /// Position Y (mm)
    pub y: f64,
    /// Total width and height (mm)
    pub size_mm: f64,
    /// Quiet zone (margin) in module units (typically 2 to 4)
    pub quiet_zone: usize,
    /// Error correction level
    pub ec_level: QrEcLevel,
    /// Visual style of modules
    pub style: QrStyle,
    /// Invert colors (light modules on dark background)
    pub invert: bool,
}

impl Default for QrCodeParams {
    fn default() -> Self {
        Self {
            content: "https://photoncam.app".to_string(),
            x: 50.0,
            y: 50.0,
            size_mm: 40.0,
            quiet_zone: 2,
            ec_level: QrEcLevel::Medium,
            style: QrStyle::SharpSquares,
            invert: false,
        }
    }
}

/// Generates laser-ready vector shapes from a text string or URL
pub fn generate_qr_shapes(params: &QrCodeParams, layer_index: usize) -> Result<Vec<Shape>, String> {
    if params.content.trim().is_empty() {
        return Err("Content string cannot be empty".to_string());
    }

    let code = QrCode::with_error_correction_level(params.content.as_bytes(), params.ec_level.into())
        .map_err(|e| format!("QR encoding failed: {}", e))?;

    let q_w = code.width(); // number of matrix modules
    let total_modules = q_w + params.quiet_zone * 2;
    let cell_size = params.size_mm / (total_modules as f64);

    let colors = code.to_colors();
    let mut shapes = Vec::new();

    // Inverted background plate if requested
    if params.invert {
        shapes.push(Shape::Rect {
            layer_index,
            x: params.x,
            y: params.y,
            width: params.size_mm,
            height: params.size_mm,
            corner_radius: 0.0,
        });
    }

    // Traverse grid row by row
    for r in 0..q_w {
        let cell_y = params.y + ((r + params.quiet_zone) as f64) * cell_size;

        match params.style {
            QrStyle::SharpSquares => {
                // Optimize: Merge consecutive dark modules on the same row into single rectangles (RLE)
                let mut c = 0;
                while c < q_w {
                    let is_dark = colors[r * q_w + c] == Color::Dark;
                    let target = if params.invert { !is_dark } else { is_dark };

                    if target {
                        let start_c = c;
                        while c < q_w && (if params.invert { colors[r * q_w + c] != Color::Dark } else { colors[r * q_w + c] == Color::Dark }) {
                            c += 1;
                        }
                        let run_len = c - start_c;
                        let cell_x = params.x + ((start_c + params.quiet_zone) as f64) * cell_size;
                        let run_width = (run_len as f64) * cell_size;

                        shapes.push(Shape::Rect {
                            layer_index,
                            x: cell_x,
                            y: cell_y,
                            width: run_width,
                            height: cell_size,
                            corner_radius: 0.0,
                        });
                    } else {
                        c += 1;
                    }
                }
            }

            QrStyle::RoundedSquares => {
                for c in 0..q_w {
                    let is_dark = colors[r * q_w + c] == Color::Dark;
                    let target = if params.invert { !is_dark } else { is_dark };

                    if target {
                        let cell_x = params.x + ((c + params.quiet_zone) as f64) * cell_size;
                        shapes.push(Shape::Rect {
                            layer_index,
                            x: cell_x,
                            y: cell_y,
                            width: cell_size,
                            height: cell_size,
                            corner_radius: cell_size * 0.28,
                        });
                    }
                }
            }

            QrStyle::Dots => {
                let radius = cell_size * 0.48;
                for c in 0..q_w {
                    let is_dark = colors[r * q_w + c] == Color::Dark;
                    let target = if params.invert { !is_dark } else { is_dark };

                    if target {
                        let center_x = params.x + ((c + params.quiet_zone) as f64 + 0.5) * cell_size;
                        let center_y = cell_y + cell_size * 0.5;
                        shapes.push(Shape::Circle {
                            layer_index,
                            center: Point2D::new(center_x, center_y),
                            radius,
                        });
                    }
                }
            }
        }
    }

    Ok(shapes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_code_generation_sharp() {
        let params = QrCodeParams {
            content: "https://photoncam.app".to_string(),
            x: 10.0,
            y: 10.0,
            size_mm: 50.0,
            quiet_zone: 2,
            ec_level: QrEcLevel::Medium,
            style: QrStyle::SharpSquares,
            invert: false,
        };

        let shapes = generate_qr_shapes(&params, 0).expect("QR code generation failed");
        assert!(!shapes.is_empty(), "Expected shapes to be generated");
    }

    #[test]
    fn test_qr_code_dots() {
        let params = QrCodeParams {
            content: "TEST-123".to_string(),
            style: QrStyle::Dots,
            ..Default::default()
        };

        let shapes = generate_qr_shapes(&params, 0).expect("QR code generation failed");
        assert!(!shapes.is_empty());
        assert!(matches!(shapes[0], Shape::Circle { .. }));
    }
}
