use crate::document::Document;
use crate::optimizer::OptimizedJob;

#[derive(Debug, Clone)]
pub struct SimulationStats {
    pub total_cut_distance_mm: f64,
    pub total_rapid_distance_mm: f64,
    pub estimated_cut_time_sec: f64,
    pub estimated_rapid_time_sec: f64,
    pub total_estimated_time_sec: f64,
    pub cut_count: usize,
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

        SimulationStats {
            total_cut_distance_mm: total_cut_dist,
            total_rapid_distance_mm: total_rapid_dist,
            estimated_cut_time_sec: cut_time,
            estimated_rapid_time_sec: rapid_time,
            total_estimated_time_sec: cut_time + rapid_time,
            cut_count: job.paths.len(),
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
