use crate::geometry::{Point2D, Polyline};
use crate::Shape;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HingePattern {
    /// Classic straight alternating lattice lines (strongest, most common for wood)
    StraightLattice,
    /// Subtle wavy / serpentine lattice lines (more flexible, organic feel)
    WavyLattice,
    /// Diagonal cross / diamond cut pattern (multi-directional flex)
    CrossLattice,
}

impl Default for HingePattern {
    fn default() -> Self {
        HingePattern::StraightLattice
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingHingeParams {
    /// Bounding rectangle position X (mm)
    pub x: f64,
    /// Bounding rectangle position Y (mm)
    pub y: f64,
    /// Width of the bend area along bend direction (mm)
    pub width: f64,
    /// Height / length of the hinge strip (mm)
    pub height: f64,
    /// Distance between parallel cut lines along X (mm, typically 2.0 - 4.0 mm)
    pub pitch: f64,
    /// Length of each individual cut segment along Y (mm, typically 15.0 - 25.0 mm)
    pub cut_length: f64,
    /// Gap / bridge length between cuts along Y (mm, typically 1.5 - 3.0 mm)
    pub gap: f64,
    /// Solid uncut border margin at top and bottom (mm)
    pub border: f64,
    /// Hinge cut pattern style
    pub pattern: HingePattern,
}

impl Default for LivingHingeParams {
    fn default() -> Self {
        Self {
            x: 50.0,
            y: 50.0,
            width: 60.0,
            height: 100.0,
            pitch: 3.0,
            cut_length: 20.0,
            gap: 2.0,
            border: 4.0,
            pattern: HingePattern::StraightLattice,
        }
    }
}

/// Generates living hinge cut vectors
pub fn generate_living_hinge(params: &LivingHingeParams) -> Vec<Polyline> {
    let mut polylines = Vec::new();

    let w = params.width.max(5.0);
    let h = params.height.max(5.0);
    let pitch = params.pitch.clamp(1.0, 20.0);
    let cut_len = params.cut_length.clamp(2.0, 100.0);
    let gap = params.gap.clamp(0.5, 20.0);
    let border = params.border.clamp(0.0, h * 0.4);

    let effective_h = h - border * 2.0;
    if effective_h <= 1.0 {
        return polylines;
    }

    let cycle = cut_len + gap;
    let num_columns = (w / pitch).floor() as usize;

    for col in 0..num_columns {
        let cur_x = params.x + (col as f64) * pitch;

        // Alternating phase offset for odd/even columns
        let is_odd = col % 2 == 1;
        let phase_offset = if is_odd { cycle * 0.5 } else { 0.0 };

        match params.pattern {
            HingePattern::StraightLattice => {
                let mut cur_y = params.y + border - phase_offset;

                while cur_y < params.y + h - border {
                    let seg_start_y = cur_y.max(params.y + border);
                    let seg_end_y = (cur_y + cut_len).min(params.y + h - border);

                    if seg_end_y > seg_start_y + 0.5 {
                        polylines.push(Polyline::new(
                            vec![
                                Point2D::new(cur_x, seg_start_y),
                                Point2D::new(cur_x, seg_end_y),
                            ],
                            false,
                        ));
                    }

                    cur_y += cycle;
                }
            }

            HingePattern::WavyLattice => {
                let mut cur_y = params.y + border - phase_offset;
                let wave_amp = pitch * 0.35;

                while cur_y < params.y + h - border {
                    let seg_start_y = cur_y.max(params.y + border);
                    let seg_end_y = (cur_y + cut_len).min(params.y + h - border);

                    if seg_end_y > seg_start_y + 1.0 {
                        let steps = 6;
                        let mut pts = Vec::with_capacity(steps + 1);
                        for s in 0..=steps {
                            let t = s as f64 / steps as f64;
                            let y_val = seg_start_y + (seg_end_y - seg_start_y) * t;
                            let x_offset = (t * std::f64::consts::PI).sin() * wave_amp * if is_odd { -1.0 } else { 1.0 };
                            pts.push(Point2D::new(cur_x + x_offset, y_val));
                        }
                        polylines.push(Polyline::new(pts, false));
                    }

                    cur_y += cycle;
                }
            }

            HingePattern::CrossLattice => {
                let mut cur_y = params.y + border - phase_offset;
                let slant = pitch * 0.5;

                while cur_y < params.y + h - border {
                    let seg_start_y = cur_y.max(params.y + border);
                    let seg_end_y = (cur_y + cut_len).min(params.y + h - border);

                    if seg_end_y > seg_start_y + 0.5 {
                        let x1 = if is_odd { cur_x - slant } else { cur_x + slant };
                        let x2 = if is_odd { cur_x + slant } else { cur_x - slant };
                        polylines.push(Polyline::new(
                            vec![
                                Point2D::new(x1, seg_start_y),
                                Point2D::new(x2, seg_end_y),
                            ],
                            false,
                        ));
                    }

                    cur_y += cycle;
                }
            }
        }
    }

    polylines
}

/// Converts generated hinge polylines into shapes ready for the canvas document
pub fn living_hinge_to_shapes(polylines: Vec<Polyline>, layer_index: usize) -> Vec<Shape> {
    polylines
        .into_iter()
        .map(|polyline| Shape::Path {
            layer_index,
            polyline,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_living_hinge_straight_cuts() {
        let params = LivingHingeParams {
            x: 10.0,
            y: 10.0,
            width: 30.0,
            height: 50.0,
            pitch: 3.0,
            cut_length: 15.0,
            gap: 2.0,
            border: 3.0,
            pattern: HingePattern::StraightLattice,
        };

        let polylines = generate_living_hinge(&params);
        assert!(!polylines.is_empty(), "Expected cuts to be generated");
        for p in &polylines {
            assert!(!p.is_closed, "Hinge cut lines should be open polylines");
            assert!(p.points.len() >= 2);
        }
    }

    #[test]
    fn test_living_hinge_wavy_pattern() {
        let params = LivingHingeParams {
            pattern: HingePattern::WavyLattice,
            ..Default::default()
        };
        let polylines = generate_living_hinge(&params);
        assert!(!polylines.is_empty());
        assert!(polylines[0].points.len() > 2, "Wavy cuts should have multiple interpolation steps");
    }
}
