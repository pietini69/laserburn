use crate::document::Document;
use crate::optimizer::OptimizedJob;

#[derive(Debug, Clone, Default)]
pub struct SimulationStats {
    pub total_cut_distance_mm: f64,
    pub total_rapid_distance_mm: f64,
    pub estimated_cut_time_sec: f64,
    pub estimated_rapid_time_sec: f64,
    pub total_estimated_time_sec: f64,
    pub cut_count: usize,
    pub formatted_time: String,
}

pub struct Simulator {
    pub acceleration_mm_s2: f64, // e.g. 3000 mm/s^2 for diode, 10000 mm/s^2 for CO2/galvo
    pub rapid_speed_mm_s: f64,   // e.g. 100 mm/s (6000 mm/min)
}

impl Default for Simulator {
    fn default() -> Self {
        Self {
            acceleration_mm_s2: 3000.0,
            rapid_speed_mm_s: 100.0,
        }
    }
}

impl Simulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Simulates the job execution and estimates cycle time with trapezoidal speed profiles
    pub fn simulate(&self, doc: &Document, job: &OptimizedJob) -> SimulationStats {
        let mut total_cut_dist = 0.0;
        let mut total_rapid_dist = 0.0;
        let mut cut_time = 0.0;
        let mut rapid_time = 0.0;

        let mut current_pos = crate::geometry::Point2D::ZERO;

        for path in &job.paths {
            let layer = doc.layers.get(&path.layer_index);
            let cut_speed = layer.map(|l| l.cut_setting.speed).unwrap_or(50.0);
            let passes = layer.map(|l| l.cut_setting.num_passes).unwrap_or(1);

            let pts = &path.polyline.points;
            if pts.len() < 2 {
                continue;
            }

            // Rapid move to start of path
            let rapid_d = current_pos.distance_to(&pts[0]);
            total_rapid_dist += rapid_d;
            rapid_time += self.trapezoid_time(rapid_d, self.rapid_speed_mm_s);

            let mut path_dist = path.polyline.length();
            if path.polyline.is_closed && pts.len() > 2 {
                path_dist += pts.last().unwrap().distance_to(&pts[0]);
            }

            for _ in 0..passes {
                total_cut_dist += path_dist;
                cut_time += self.trapezoid_time(path_dist, cut_speed);
            }

            current_pos = if path.polyline.is_closed {
                pts[0]
            } else {
                *pts.last().unwrap()
            };
        }

        // Return to home
        let return_d = current_pos.distance_to(&crate::geometry::Point2D::ZERO);
        total_rapid_dist += return_d;
        rapid_time += self.trapezoid_time(return_d, self.rapid_speed_mm_s);

        let total_time = cut_time + rapid_time;
        SimulationStats {
            total_cut_distance_mm: total_cut_dist,
            total_rapid_distance_mm: total_rapid_dist,
            estimated_cut_time_sec: cut_time,
            estimated_rapid_time_sec: rapid_time,
            total_estimated_time_sec: total_time,
            cut_count: job.paths.len(),
            formatted_time: Self::format_duration(total_time),
        }
    }

    /// Formats elapsed or estimated duration into human-readable representation:
    /// e.g. "45 s", "4 min 12 s", or "1 h 24 min 05 s"
    pub fn format_duration(seconds: f64) -> String {
        let s = seconds.max(0.0).round() as u64;
        let h = s / 3600;
        let m = (s % 3600) / 60;
        let sec = s % 60;
        if h > 0 {
            format!("{} h {:02} min {:02} s", h, m, sec)
        } else if m > 0 {
            format!("{} min {:02} s", m, sec)
        } else {
            format!("{} s", sec)
        }
    }

    /// Rapid and non-blocking job duration estimation for the whole document.
    /// Perfectly handles vector paths as well as massive raster images without
    /// incurring heavy dithering CPU overhead.
    pub fn estimate_document(&self, doc: &Document, rapid_speed_mm_min: f64) -> SimulationStats {
        let mut total_cut_dist = 0.0;
        let mut total_rapid_dist = 0.0;
        let mut cut_time = 0.0;
        let mut rapid_time = 0.0;
        let mut cut_count = 0;

        let rapid_speed_s = if rapid_speed_mm_min > 60.0 {
            rapid_speed_mm_min / 60.0
        } else {
            self.rapid_speed_mm_s
        };
        let mut current_pos = crate::geometry::Point2D::ZERO;

        for shape in &doc.shapes {
            let layer_idx = shape.layer_index();
            let layer = doc.layers.get(&layer_idx);
            let cs = layer.map(|l| &l.cut_setting);
            let passes = cs.map(|c| c.num_passes).unwrap_or(1);
            let cut_speed = cs.map(|c| c.speed).unwrap_or(50.0).max(1.0);

            match shape {
                crate::document::Shape::Image { width, height, .. } => {
                    let interval = cs.map(|c| c.line_interval).unwrap_or(0.08).max(0.01);
                    let num_lines = (*height / interval).ceil().max(1.0);
                    // Line engraving time: cruise + acceleration/deceleration trapezoid
                    let t_line = (*width / cut_speed) + 2.0 * (cut_speed / self.acceleration_mm_s2);
                    let t_step = self.trapezoid_time(interval, rapid_speed_s);
                    let img_time = (t_line + t_step) * num_lines * (passes as f64);
                    let img_cut_d = *width * num_lines * (passes as f64);
                    let img_rapid_d = interval * num_lines * (passes as f64);

                    total_cut_dist += img_cut_d;
                    total_rapid_dist += img_rapid_d;
                    cut_time += img_time;
                    cut_count += num_lines as usize;

                    let b = shape.bounds();
                    let rapid_d = current_pos.distance_to(&crate::geometry::Point2D::new(b.min_x, b.min_y));
                    total_rapid_dist += rapid_d;
                    rapid_time += self.trapezoid_time(rapid_d, rapid_speed_s);
                    current_pos = crate::geometry::Point2D::new(b.max_x, b.max_y);
                }
                _ => {
                    let polys = shape.to_polylines();
                    for poly in polys {
                        let pts = &poly.points;
                        if pts.len() < 2 {
                            continue;
                        }

                        let rapid_d = current_pos.distance_to(&pts[0]);
                        total_rapid_dist += rapid_d;
                        rapid_time += self.trapezoid_time(rapid_d, rapid_speed_s);

                        let mut path_d = poly.length();
                        if poly.is_closed && pts.len() > 2 {
                            path_d += pts.last().unwrap().distance_to(&pts[0]);
                        }

                        for _ in 0..passes {
                            total_cut_dist += path_d;
                            cut_time += self.trapezoid_time(path_d, cut_speed);
                        }

                        cut_count += 1;
                        current_pos = if poly.is_closed {
                            pts[0]
                        } else {
                            *pts.last().unwrap()
                        };
                    }
                }
            }
        }

        // Return to home
        let return_d = current_pos.distance_to(&crate::geometry::Point2D::ZERO);
        total_rapid_dist += return_d;
        rapid_time += self.trapezoid_time(return_d, rapid_speed_s);

        let total_time = cut_time + rapid_time;
        SimulationStats {
            total_cut_distance_mm: total_cut_dist,
            total_rapid_distance_mm: total_rapid_dist,
            estimated_cut_time_sec: cut_time,
            estimated_rapid_time_sec: rapid_time,
            total_estimated_time_sec: total_time,
            cut_count,
            formatted_time: Self::format_duration(total_time),
        }
    }

    /// Calculates time to travel a distance given target speed and machine acceleration
    fn trapezoid_time(&self, distance: f64, max_speed: f64) -> f64 {
        if distance <= 1e-4 {
            return 0.0;
        }
        // Distance needed to accelerate to max_speed and decelerate back to 0:
        // d_accel = v^2 / (2 * a)
        let d_accel_decel = (max_speed * max_speed) / self.acceleration_mm_s2;

        if distance >= d_accel_decel {
            // Full trapezoid (accelerate, cruise at max_speed, decelerate)
            let t_accel = max_speed / self.acceleration_mm_s2;
            let d_cruise = distance - d_accel_decel;
            let t_cruise = d_cruise / max_speed;
            2.0 * t_accel + t_cruise
        } else {
            // Triangular profile (does not reach full max_speed)
            // v_peak = sqrt(a * d)
            // t = 2 * sqrt(d / a)
            2.0 * (distance / self.acceleration_mm_s2).sqrt()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Document, Layer, Shape};

    #[test]
    fn test_format_duration() {
        assert_eq!(Simulator::format_duration(35.0), "35 s");
        assert_eq!(Simulator::format_duration(125.0), "2 min 05 s");
        assert_eq!(Simulator::format_duration(3665.0), "1 h 01 min 05 s");
    }

    #[test]
    fn test_estimate_document_with_image_and_rect() {
        let mut doc = Document::new("Test Doc", 400.0, 400.0);
        let mut layer0 = Layer::new(0, "Cuts", "#000000");
        layer0.cut_setting.speed = 10.0; // 10 mm/s
        doc.layers.insert(0, layer0);

        let mut layer1 = Layer::new(1, "Engrave", "#00A0FF");
        layer1.cut_setting.speed = 100.0; // 100 mm/s
        layer1.cut_setting.line_interval = 0.1; // 10 lines per mm
        doc.layers.insert(1, layer1);

        // Add 50x50 mm rect
        doc.add_shape(Shape::Rect {
            layer_index: 0,
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 50.0,
            corner_radius: 0.0,
        });

        // Add 50x50 mm image
        doc.add_shape(Shape::Image {
            layer_index: 1,
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
            raw_pixels: None,
            raw_width: 500,
            raw_height: 500,
            image_path: None,
        });

        let sim = Simulator::new();
        let stats = sim.estimate_document(&doc, 6000.0);
        assert!(stats.total_cut_distance_mm > 25000.0); // 500 lines * 50mm + 200mm rect
        assert!(stats.total_estimated_time_sec > 100.0);
        assert!(!stats.formatted_time.is_empty());
        println!("Estimated job time: {}", stats.formatted_time);
    }
}
