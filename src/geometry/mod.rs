use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn distance_squared_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2D {
    pub dx: f64,
    pub dy: f64,
}

impl Vector2D {
    pub fn new(dx: f64, dy: f64) -> Self {
        Self { dx, dy }
    }

    pub fn length(&self) -> f64 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len > 1e-12 {
            Self {
                dx: self.dx / len,
                dy: self.dy / len,
            }
        } else {
            Self { dx: 0.0, dy: 0.0 }
        }
    }

    pub fn perpendicular(&self) -> Self {
        Self {
            dx: -self.dy,
            dy: self.dx,
        }
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.dx * other.dx + self.dy * other.dy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn empty() -> Self {
        Self {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    pub fn from_points(points: &[Point2D]) -> Self {
        let mut bb = Self::empty();
        for p in points {
            bb.expand_to_include(p);
        }
        bb
    }

    pub fn expand_to_include(&mut self, p: &Point2D) {
        if p.x < self.min_x {
            self.min_x = p.x;
        }
        if p.x > self.max_x {
            self.max_x = p.x;
        }
        if p.y < self.min_y {
            self.min_y = p.y;
        }
        if p.y > self.max_y {
            self.max_y = p.y;
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    pub fn width(&self) -> f64 {
        if self.max_x >= self.min_x {
            self.max_x - self.min_x
        } else {
            0.0
        }
    }

    pub fn height(&self) -> f64 {
        if self.max_y >= self.min_y {
            self.max_y - self.min_y
        } else {
            0.0
        }
    }

    pub fn center(&self) -> Point2D {
        Point2D::new(
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
    }

    pub fn contains_point(&self, p: &Point2D) -> bool {
        p.x >= self.min_x && p.x <= self.max_x && p.y >= self.min_y && p.y <= self.max_y
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    pub points: Vec<Point2D>,
    pub is_closed: bool,
}

impl Polyline {
    pub fn new(points: Vec<Point2D>, is_closed: bool) -> Self {
        Self { points, is_closed }
    }

    pub fn bounds(&self) -> BoundingBox {
        BoundingBox::from_points(&self.points)
    }

    pub fn length(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 0..self.points.len() - 1 {
            total += self.points[i].distance_to(&self.points[i + 1]);
        }
        if self.is_closed && self.points.len() > 2 {
            total += self.points.last().unwrap().distance_to(&self.points[0]);
        }
        total
    }

    pub fn reverse(&mut self) {
        self.points.reverse();
    }

    pub fn contains_point(&self, p: &Point2D) -> bool {
        if !self.is_closed || self.points.len() < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = self.points.len() - 1;
        for i in 0..self.points.len() {
            let pi = self.points[i];
            let pj = self.points[j];
            if ((pi.y > p.y) != (pj.y > p.y))
                && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    pub fn signed_area(&self) -> f64 {
        if self.points.len() < 3 {
            return 0.0;
        }
        let mut area = 0.0;
        let n = self.points.len();
        for i in 0..n {
            let j = (i + 1) % n;
            area += self.points[i].x * self.points[j].y;
            area -= self.points[j].x * self.points[i].y;
        }
        area * 0.5
    }

    pub fn is_clockwise(&self) -> bool {
        self.signed_area() < 0.0
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        for p in &mut self.points {
            p.x += dx;
            p.y += dy;
        }
    }

    pub fn scale(&mut self, origin: Point2D, sx: f64, sy: f64) {
        for p in &mut self.points {
            p.x = origin.x + (p.x - origin.x) * sx;
            p.y = origin.y + (p.y - origin.y) * sy;
        }
    }

    pub fn rotate(&mut self, origin: Point2D, angle_rad: f64) {
        let (sin_a, cos_a) = angle_rad.sin_cos();
        for p in &mut self.points {
            let dx = p.x - origin.x;
            let dy = p.y - origin.y;
            p.x = origin.x + dx * cos_a - dy * sin_a;
            p.y = origin.y + dx * sin_a + dy * cos_a;
        }
    }

    pub fn distance_to_point(&self, p: &Point2D) -> f64 {
        if self.points.is_empty() {
            return f64::INFINITY;
        }
        if self.points.len() == 1 {
            return p.distance_to(&self.points[0]);
        }
        let mut min_d = f64::INFINITY;
        let n = self.points.len();
        let count = if self.is_closed { n } else { n - 1 };
        for i in 0..count {
            let a = self.points[i];
            let b = self.points[(i + 1) % n];
            let d = point_to_segment_distance(p, &a, &b);
            if d < min_d {
                min_d = d;
            }
        }
        min_d
    }
}

pub fn point_to_segment_distance(p: &Point2D, a: &Point2D, b: &Point2D) -> f64 {
    let l2 = a.distance_squared_to(b);
    if l2 < 1e-9 {
        return p.distance_to(a);
    }
    let t = (((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2).clamp(0.0, 1.0);
    let proj = Point2D::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y));
    p.distance_to(&proj)
}

/// Evaluates a Cubic Bezier curve at parameter t in [0.0, 1.0]
pub fn eval_cubic_bezier(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D, t: f64) -> Point2D {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    Point2D::new(
        uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
        uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
    )
}

/// Recursively flattens a cubic bezier curve into points within chordal deviation tolerance
pub fn tessellate_cubic_bezier(
    p0: Point2D,
    p1: Point2D,
    p2: Point2D,
    p3: Point2D,
    tolerance: f64,
    points: &mut Vec<Point2D>,
) {
    let chord = p3.distance_to(&p0);
    let d1 = p1.distance_to(&p0.lerp(&p3, 1.0 / 3.0));
    let d2 = p2.distance_to(&p0.lerp(&p3, 2.0 / 3.0));

    if (d1 + d2) <= tolerance || chord < 1e-4 {
        points.push(p3);
    } else {
        // De Casteljau subdivision
        let p01 = p0.lerp(&p1, 0.5);
        let p12 = p1.lerp(&p2, 0.5);
        let p23 = p2.lerp(&p3, 0.5);

        let p012 = p01.lerp(&p12, 0.5);
        let p123 = p12.lerp(&p23, 0.5);

        let p0123 = p012.lerp(&p123, 0.5);

        tessellate_cubic_bezier(p0, p01, p012, p0123, tolerance, points);
        tessellate_cubic_bezier(p0123, p123, p23, p3, tolerance, points);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polyline_contains_point() {
        let pts = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
        ];
        let poly = Polyline::new(pts, true);
        assert!(poly.contains_point(&Point2D::new(5.0, 5.0)));
        assert!(!poly.contains_point(&Point2D::new(15.0, 5.0)));
        assert!(!poly.contains_point(&Point2D::new(-5.0, 5.0)));
    }
}
