//! Smart 2D Auto-Nesting and Material Packing Engine for LaserBurn.
//! Automatically packs multiple irregular and geometric vector shapes onto a laser bed or sheet
//! using 2D bottom-left packing with multi-angle rotation and collision detection.

use crate::document::{Document, Shape};
use crate::geometry::{BoundingBox, Point2D, Polyline};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AllowedRotations {
    None,     // 0° only
    TwoWay,   // 0°, 90°
    FourWay,  // 0°, 90°, 180°, 270°
    EightWay, // 0°, 45°, 90°, 135°, 180°, 225°, 270°, 315°
}

impl AllowedRotations {
    pub const ALL: &'static [AllowedRotations] = &[
        AllowedRotations::None,
        AllowedRotations::TwoWay,
        AllowedRotations::FourWay,
        AllowedRotations::EightWay,
    ];

    pub fn angles_deg(&self) -> &'static [f64] {
        match self {
            AllowedRotations::None => &[0.0],
            AllowedRotations::TwoWay => &[0.0, 90.0],
            AllowedRotations::FourWay => &[0.0, 90.0, 180.0, 270.0],
            AllowedRotations::EightWay => &[0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0],
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AllowedRotations::None => "0° (No Rotation)",
            AllowedRotations::TwoWay => "0°, 90° (2-Way)",
            AllowedRotations::FourWay => "90° Steps (4-Way)",
            AllowedRotations::EightWay => "45° Steps (8-Way)",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NestingOptions {
    pub sheet_width: f64,
    pub sheet_height: f64,
    pub part_spacing: f64,
    pub sheet_margin: f64,
    pub allowed_rotations: AllowedRotations,
    pub only_selected: bool,
}

impl Default for NestingOptions {
    fn default() -> Self {
        Self {
            sheet_width: 400.0,
            sheet_height: 400.0,
            part_spacing: 2.0,
            sheet_margin: 3.0,
            allowed_rotations: AllowedRotations::FourWay,
            only_selected: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartPlacement {
    pub original_index: usize,
    pub placed_shape: Shape,
    pub placed: bool,
}

#[derive(Debug, Clone)]
pub struct NestingResult {
    pub placed_count: usize,
    pub unplaced_count: usize,
    pub total_parts: usize,
    pub material_utilization_pct: f64,
    pub used_bounds: BoundingBox,
    pub placements: Vec<PartPlacement>,
}

#[derive(Debug, Clone)]
struct PlacedPart {
    polylines: Vec<Polyline>,
    bounds: BoundingBox,
}

/// Checks if two line segments intersect
fn segments_intersect(a0: &Point2D, a1: &Point2D, b0: &Point2D, b1: &Point2D) -> bool {
    let dax = a1.x - a0.x;
    let day = a1.y - a0.y;
    let dbx = b1.x - b0.x;
    let dby = b1.y - b0.y;

    let denom = dax * dby - day * dbx;
    if denom.abs() < 1e-12 {
        return false;
    }

    let dx = b0.x - a0.x;
    let dy = b0.y - a0.y;
    let ta = (dx * dby - dy * dbx) / denom;
    let tb = (dx * day - dy * dax) / denom;

    ta >= 0.0 && ta <= 1.0 && tb >= 0.0 && tb <= 1.0
}

/// Calculates squared distance from point p to segment [a, b]
fn point_to_seg_dist_sq(p: &Point2D, a: &Point2D, b: &Point2D) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-12 {
        return (p.x - a.x) * (p.x - a.x) + (p.y - a.y) * (p.y - a.y);
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len_sq).clamp(0.0, 1.0);
    let proj_x = a.x + t * dx;
    let proj_y = a.y + t * dy;
    (p.x - proj_x) * (p.x - proj_x) + (p.y - proj_y) * (p.y - proj_y)
}

/// Checks if two polylines collide given a minimum clearance spacing
fn polylines_collide(poly_a: &Polyline, poly_b: &Polyline, spacing: f64) -> bool {
    let bb_a = poly_a.bounds();
    let bb_b = poly_b.bounds();

    if bb_a.max_x + spacing < bb_b.min_x
        || bb_a.min_x - spacing > bb_b.max_x
        || bb_a.max_y + spacing < bb_b.min_y
        || bb_a.min_y - spacing > bb_b.max_y
    {
        return false;
    }

    // Interior inclusion test for closed contours
    if poly_a.is_closed && poly_a.points.len() >= 3 {
        for p in &poly_b.points {
            if poly_a.contains_point(p) {
                return true;
            }
        }
    }
    if poly_b.is_closed && poly_b.points.len() >= 3 {
        for p in &poly_a.points {
            if poly_b.contains_point(p) {
                return true;
            }
        }
    }

    let na = poly_a.points.len();
    let nb = poly_b.points.len();
    if na < 2 || nb < 2 {
        return false;
    }

    let segs_a = if poly_a.is_closed { na } else { na - 1 };
    let segs_b = if poly_b.is_closed { nb } else { nb - 1 };
    let sp_sq = spacing * spacing;

    for i in 0..segs_a {
        let a0 = poly_a.points[i];
        let a1 = poly_a.points[(i + 1) % na];

        let min_ax = a0.x.min(a1.x) - spacing;
        let max_ax = a0.x.max(a1.x) + spacing;
        let min_ay = a0.y.min(a1.y) - spacing;
        let max_ay = a0.y.max(a1.y) + spacing;

        for j in 0..segs_b {
            let b0 = poly_b.points[j];
            let b1 = poly_b.points[(j + 1) % nb];

            if b0.x.max(b1.x) < min_ax
                || b0.x.min(b1.x) > max_ax
                || b0.y.max(b1.y) < min_ay
                || b0.y.min(b1.y) > max_ay
            {
                continue;
            }

            if segments_intersect(&a0, &a1, &b0, &b1) {
                return true;
            }

            if point_to_seg_dist_sq(&a0, &b0, &b1) < sp_sq
                || point_to_seg_dist_sq(&a1, &b0, &b1) < sp_sq
                || point_to_seg_dist_sq(&b0, &a0, &a1) < sp_sq
                || point_to_seg_dist_sq(&b1, &a0, &a1) < sp_sq
            {
                return true;
            }
        }
    }

    false
}

/// Checks if candidate shape at candidate position collides with any already placed part
fn collides_with_placed(
    cand_bounds: &BoundingBox,
    cand_polylines: &[Polyline],
    placed: &[PlacedPart],
    spacing: f64,
) -> bool {
    for p in placed {
        // Broadphase AABB rejection
        if cand_bounds.max_x + spacing < p.bounds.min_x
            || cand_bounds.min_x - spacing > p.bounds.max_x
            || cand_bounds.max_y + spacing < p.bounds.min_y
            || cand_bounds.min_y - spacing > p.bounds.max_y
        {
            continue;
        }

        // Narrowphase polyline collision check
        for poly_a in cand_polylines {
            for poly_b in &p.polylines {
                if polylines_collide(poly_a, poly_b, spacing) {
                    return true;
                }
            }
        }
    }
    false
}

/// Computes the approximate area of a shape
fn approximate_shape_area(shape: &Shape) -> f64 {
    let b = shape.bounds();
    let box_area = b.width() * b.height();
    match shape {
        Shape::Circle { radius, .. } => std::f64::consts::PI * radius * radius,
        Shape::Rect { width, height, .. } => width * height,
        Shape::Path { polyline, .. } => {
            let poly_area = polyline.signed_area().abs();
            if poly_area > 1e-4 {
                poly_area
            } else {
                box_area * 0.7
            }
        }
        _ => box_area * 0.75,
    }
}

/// Runs the 2D auto-nesting material packing algorithm
pub fn run_nesting(
    shapes: &[Shape],
    indices_to_nest: &[usize],
    options: &NestingOptions,
) -> NestingResult {
    if indices_to_nest.is_empty() {
        return NestingResult {
            placed_count: 0,
            unplaced_count: 0,
            total_parts: 0,
            material_utilization_pct: 0.0,
            used_bounds: BoundingBox::empty(),
            placements: Vec::new(),
        };
    }

    let sheet_w = options.sheet_width.max(10.0);
    let sheet_h = options.sheet_height.max(10.0);
    let margin = options.sheet_margin.max(0.0);
    let spacing = options.part_spacing.max(0.0);
    let angles = options.allowed_rotations.angles_deg();

    // Prepare items to nest
    struct Item {
        original_idx: usize,
        shape: Shape,
        area: f64,
        max_dim: f64,
    }

    let mut items: Vec<Item> = indices_to_nest
        .iter()
        .copied()
        .filter(|&idx| idx < shapes.len())
        .map(|idx| {
            let s = shapes[idx].clone();
            let b = s.bounds();
            let area = approximate_shape_area(&s);
            let max_dim = b.width().max(b.height());
            Item {
                original_idx: idx,
                shape: s,
                area,
                max_dim,
            }
        })
        .collect();

    // First-Fit Decreasing: Sort by max dimension and area descending (large parts placed first)
    items.sort_by(|a, b| {
        b.max_dim
            .partial_cmp(&a.max_dim)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.area.partial_cmp(&a.area).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut placed_parts: Vec<PlacedPart> = Vec::new();
    let mut placements: Vec<PartPlacement> = Vec::new();

    let mut total_part_area = 0.0;
    let mut overall_used_bounds = BoundingBox::empty();

    for item in items {
        total_part_area += item.area;

        let mut best_placement: Option<(Shape, Vec<Polyline>, BoundingBox, f64)> = None;

        for &angle_deg in angles {
            let mut rotated_shape = item.shape.clone();
            if angle_deg.abs() > 1e-4 {
                let rot_center = rotated_shape.bounds().center();
                rotated_shape.rotate(rot_center, angle_deg.to_radians());
            }

            // Normalize so min_x = 0, min_y = 0
            let r_bounds = rotated_shape.bounds();
            rotated_shape.translate(-r_bounds.min_x, -r_bounds.min_y);
            let part_w = r_bounds.width();
            let part_h = r_bounds.height();

            if part_w > sheet_w - margin * 2.0 || part_h > sheet_h - margin * 2.0 {
                continue;
            }

            // Generate candidate X and Y anchor coordinates
            let mut x_anchors = vec![margin];
            let mut y_anchors = vec![margin];

            for p in &placed_parts {
                let cand_x1 = p.bounds.max_x + spacing;
                let cand_x2 = p.bounds.min_x;
                if cand_x1 + part_w <= sheet_w - margin {
                    x_anchors.push(cand_x1);
                }
                if cand_x2 + part_w <= sheet_w - margin {
                    x_anchors.push(cand_x2);
                }

                let cand_y1 = p.bounds.max_y + spacing;
                let cand_y2 = p.bounds.min_y;
                if cand_y1 + part_h <= sheet_h - margin {
                    y_anchors.push(cand_y1);
                }
                if cand_y2 + part_h <= sheet_h - margin {
                    y_anchors.push(cand_y2);
                }
            }

            x_anchors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            x_anchors.dedup_by(|a, b| (*a - *b).abs() < 0.1);

            y_anchors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            y_anchors.dedup_by(|a, b| (*a - *b).abs() < 0.1);

            // Test anchor points
            for &test_y in &y_anchors {
                for &test_x in &x_anchors {
                    let mut candidate_shape = rotated_shape.clone();
                    candidate_shape.translate(test_x, test_y);
                    let mut candidate_bounds = BoundingBox::new(test_x, test_y, test_x + part_w, test_y + part_h);
                    let mut candidate_polys = candidate_shape.to_polylines();

                    if !collides_with_placed(&candidate_bounds, &candidate_polys, &placed_parts, spacing) {
                        // Apply Bottom-Left gravity push: slide downwards as far as possible
                        let mut final_y = test_y;
                        if final_y > margin {
                            let mut step_y = (final_y - margin).min(10.0);
                            while step_y >= 0.5 {
                                let try_y = (final_y - step_y).max(margin);
                                if (try_y - final_y).abs() < 1e-4 {
                                    break;
                                }
                                let shift_y = try_y - final_y;
                                let mut test_b = candidate_bounds.clone();
                                test_b.min_y += shift_y;
                                test_b.max_y += shift_y;
                                let mut test_p = candidate_polys.clone();
                                for poly in &mut test_p {
                                    poly.translate(0.0, shift_y);
                                }
                                if !collides_with_placed(&test_b, &test_p, &placed_parts, spacing) {
                                    final_y = try_y;
                                    candidate_bounds = test_b;
                                    candidate_polys = test_p;
                                    candidate_shape.translate(0.0, shift_y);
                                    if (final_y - margin).abs() < 1e-4 {
                                        break;
                                    }
                                } else {
                                    step_y *= 0.5;
                                }
                            }
                        }

                        // Apply Leftward gravity push: slide leftwards as far as possible
                        let mut final_x = test_x;
                        if final_x > margin {
                            let mut step_x = (final_x - margin).min(10.0);
                            while step_x >= 0.5 {
                                let try_x = (final_x - step_x).max(margin);
                                if (try_x - final_x).abs() < 1e-4 {
                                    break;
                                }
                                let shift_x = try_x - final_x;
                                let mut test_b = candidate_bounds.clone();
                                test_b.min_x += shift_x;
                                test_b.max_x += shift_x;
                                let mut test_p = candidate_polys.clone();
                                for poly in &mut test_p {
                                    poly.translate(shift_x, 0.0);
                                }
                                if !collides_with_placed(&test_b, &test_p, &placed_parts, spacing) {
                                    final_x = try_x;
                                    candidate_bounds = test_b;
                                    candidate_polys = test_p;
                                    candidate_shape.translate(shift_x, 0.0);
                                    if (final_x - margin).abs() < 1e-4 {
                                        break;
                                    }
                                } else {
                                    step_x *= 0.5;
                                }
                            }
                        }

                        // Bottom-Left score: prioritize minimal Y first, then minimal X
                        let score = final_y * 1000.0 + final_x;

                        if best_placement.as_ref().map_or(true, |bp| score < bp.3) {
                            best_placement = Some((candidate_shape, candidate_polys, candidate_bounds, score));
                        }
                    }
                }
            }
        }

        if let Some((shape, polys, bounds, _)) = best_placement {
            overall_used_bounds = overall_used_bounds.union(&bounds);
            placed_parts.push(PlacedPart {
                polylines: polys,
                bounds,
            });
            placements.push(PartPlacement {
                original_index: item.original_idx,
                placed_shape: shape,
                placed: true,
            });
        } else {
            // Cannot fit on sheet
            placements.push(PartPlacement {
                original_index: item.original_idx,
                placed_shape: item.shape,
                placed: false,
            });
        }
    }

    let placed_count = placements.iter().filter(|p| p.placed).count();
    let unplaced_count = placements.len() - placed_count;

    let used_area = if !overall_used_bounds.is_empty() {
        overall_used_bounds.width() * overall_used_bounds.height()
    } else {
        0.0
    };

    let material_utilization_pct = if used_area > 1e-4 {
        ((total_part_area / used_area) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    NestingResult {
        placed_count,
        unplaced_count,
        total_parts: placements.len(),
        material_utilization_pct,
        used_bounds: overall_used_bounds,
        placements,
    }
}

/// Applies nesting placements directly to the document shapes
pub fn apply_nesting_result(doc: &mut Document, result: &NestingResult) {
    for placement in &result.placements {
        if placement.placed && placement.original_index < doc.shapes.len() {
            doc.shapes[placement.original_index] = placement.placed_shape.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nesting_simple_rectangles() {
        let shapes = vec![
            Shape::Rect { layer_index: 0, x: 100.0, y: 100.0, width: 40.0, height: 30.0, corner_radius: 0.0 },
            Shape::Rect { layer_index: 0, x: 200.0, y: 200.0, width: 50.0, height: 40.0, corner_radius: 0.0 },
            Shape::Rect { layer_index: 0, x: 0.0, y: 0.0, width: 30.0, height: 20.0, corner_radius: 0.0 },
        ];
        let indices = vec![0, 1, 2];
        let options = NestingOptions {
            sheet_width: 300.0,
            sheet_height: 300.0,
            part_spacing: 2.0,
            sheet_margin: 5.0,
            allowed_rotations: AllowedRotations::FourWay,
            only_selected: false,
        };

        let result = run_nesting(&shapes, &indices, &options);
        assert_eq!(result.placed_count, 3, "All 3 rectangles must be placed");
        assert_eq!(result.unplaced_count, 0);
        assert!(result.material_utilization_pct > 0.0);

        // Verify that no two placed shapes collide
        for i in 0..result.placements.len() {
            for j in (i + 1)..result.placements.len() {
                let p1 = &result.placements[i];
                let p2 = &result.placements[j];
                assert!(p1.placed && p2.placed);
                let b1 = p1.placed_shape.bounds();
                let b2 = p2.placed_shape.bounds();

                let no_overlap = b1.max_x + options.part_spacing <= b2.min_x + 1e-4
                    || b2.max_x + options.part_spacing <= b1.min_x + 1e-4
                    || b1.max_y + options.part_spacing <= b2.min_y + 1e-4
                    || b2.max_y + options.part_spacing <= b1.min_y + 1e-4;

                assert!(no_overlap, "Placed shapes must respect spacing: b1={:?}, b2={:?}", b1, b2);
            }
        }
    }

    #[test]
    fn test_nesting_sheet_too_small() {
        let shapes = vec![
            Shape::Rect { layer_index: 0, x: 0.0, y: 0.0, width: 100.0, height: 100.0, corner_radius: 0.0 },
            Shape::Rect { layer_index: 0, x: 0.0, y: 0.0, width: 100.0, height: 100.0, corner_radius: 0.0 },
        ];
        let indices = vec![0, 1];
        let options = NestingOptions {
            sheet_width: 120.0,
            sheet_height: 120.0,
            part_spacing: 5.0,
            sheet_margin: 5.0,
            allowed_rotations: AllowedRotations::None,
            only_selected: false,
        };

        let result = run_nesting(&shapes, &indices, &options);
        assert_eq!(result.placed_count, 1, "Only 1 part can fit on small sheet");
        assert_eq!(result.unplaced_count, 1, "1 part should not fit");
    }
}
