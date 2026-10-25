use crate::geometry::{Point2D, Polyline};

/// Computes the intersection of an infinite line defined by point P and direction D
/// with a line segment [A, B]. Returns t along segment [0.0, 1.0] if intersecting.
fn line_segment_intersection(
    origin: Point2D,
    dir: Point2D,
    a: Point2D,
    b: Point2D,
) -> Option<Point2D> {
    let v1 = Point2D::new(origin.x - a.x, origin.y - a.y);
    let v2 = Point2D::new(b.x - a.x, b.y - a.y);
    let v3 = Point2D::new(-dir.y, dir.x);

    let dot = v2.x * v3.x + v2.y * v3.y;
    if dot.abs() < 1e-9 {
        return None;
    }

    let t1 = (v2.x * v1.y - v2.y * v1.x) / dot;
    let t2 = (v1.x * v3.x + v1.y * v3.y) / dot;

    if (0.0..=1.0).contains(&t2) {
        Some(Point2D::new(origin.x + t1 * dir.x, origin.y + t1 * dir.y))
    } else {
        None
    }
}

/// Generates raster hatch toolpaths (parallel scan lines) inside closed polygons
pub fn generate_hatch(
    polylines: &[Polyline],
    angle_degrees: f64,
    line_interval: f64,
    cross_hatch: bool,
) -> Vec<Polyline> {
    let mut result = generate_single_hatch(polylines, angle_degrees, line_interval);
    if cross_hatch {
        let mut second = generate_single_hatch(polylines, angle_degrees + 90.0, line_interval);
        result.append(&mut second);
    }
    result
}

fn generate_single_hatch(
    polylines: &[Polyline],
    angle_degrees: f64,
    line_interval: f64,
) -> Vec<Polyline> {
    if polylines.is_empty() || line_interval <= 1e-4 {
        return Vec::new();
    }

    // Determine overall bounding box
    let mut bounds = polylines[0].bounds();
    for p in &polylines[1..] {
        bounds = bounds.union(&p.bounds());
    }

    let center = bounds.center();
    let diag = (bounds.width().powi(2) + bounds.height().powi(2)).sqrt() * 0.6;

    let rad = angle_degrees.to_radians();
    let dir = Point2D::new(rad.cos(), rad.sin());
    let normal = Point2D::new(-rad.sin(), rad.cos());

    let steps = (diag * 2.0 / line_interval).ceil() as i32;
    let mut scan_lines: Vec<Polyline> = Vec::new();

    for i in -steps..=steps {
        let offset = i as f64 * line_interval;
        let line_origin = Point2D::new(
            center.x + normal.x * offset,
            center.y + normal.y * offset,
        );

        let mut intersections: Vec<f64> = Vec::new();

        // Intersect with all polygon segments
        for poly in polylines {
            if poly.points.len() < 2 {
                continue;
            }
            let n = poly.points.len();
            let count = if poly.is_closed { n } else { n - 1 };

            for j in 0..count {
                let a = poly.points[j];
                let b = poly.points[(j + 1) % n];

                if let Some(pt) = line_segment_intersection(line_origin, dir, a, b) {
                    // Project pt onto dir to get scalar coordinate along scanline
                    let t = (pt.x - line_origin.x) * dir.x + (pt.y - line_origin.y) * dir.y;
                    intersections.push(t);
                }
            }
        }

        if intersections.len() >= 2 {
            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            // Filter near duplicates (corner intersections)
            intersections.dedup_by(|a, b| (*a - *b).abs() < 1e-5);

            // Pair intersections into solid fill segments (even-odd rule)
            for chunk in intersections.chunks_exact(2) {
                let p_start = Point2D::new(
                    line_origin.x + dir.x * chunk[0],
                    line_origin.y + dir.y * chunk[0],
                );
                let p_end = Point2D::new(
                    line_origin.x + dir.x * chunk[1],
                    line_origin.y + dir.y * chunk[1],
                );

                scan_lines.push(Polyline::new(vec![p_start, p_end], false));
            }
        }
    }

    // Connect adjacent scan lines in serpentine (zigzag) order to minimize travel
    for (i, line) in scan_lines.iter_mut().enumerate() {
        if i % 2 == 1 {
            line.reverse();
        }
    }

    scan_lines
}
