use crate::document::Document;
use crate::optimizer::OptimizedJob;

pub struct RuidaGenerator {
    pub step_per_um: f64, // Typically 1000 steps per mm (1 step = 1 um)
}

impl Default for RuidaGenerator {
    fn default() -> Self {
        Self { step_per_um: 1000.0 }
    }
}

impl RuidaGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates Ruida .rd binary byte stream
    pub fn generate_rd(&self, doc: &Document, job: &OptimizedJob) -> Vec<u8> {
        let mut stream = Vec::new();

        // Magic / Header for Ruida RD stream
        // Standard Ruida job files begin with setup records
        stream.extend_from_slice(b"RD6442");
        stream.push(0x00);

        for path in &job.paths {
            let layer = doc.layers.get(&path.layer_index);
            let speed = layer.map(|l| l.cut_setting.speed).unwrap_or(50.0);
            let power = layer.map(|l| l.cut_setting.power_max).unwrap_or(80.0);

            let pts = &path.polyline.points;
            if pts.len() < 2 {
                continue;
            }

            // Command: Set Speed (0x02)
            stream.push(0x02);
            let speed_val = (speed * 100.0) as u32;
            stream.extend_from_slice(&speed_val.to_le_bytes());

            // Command: Set Laser Power (0x03)
            stream.push(0x03);
            let power_val = (power * 10.0) as u16;
            stream.extend_from_slice(&power_val.to_le_bytes());

            // Command: Rapid Move (0x04)
            stream.push(0x04);
            let start_x = (pts[0].x * self.step_per_um) as i32;
            let start_y = (pts[0].y * self.step_per_um) as i32;
            stream.extend_from_slice(&start_x.to_le_bytes());
            stream.extend_from_slice(&start_y.to_le_bytes());

            // Command: Laser On & Cut Lines (0x05)
            for pt in &pts[1..] {
                stream.push(0x05);
                let x = (pt.x * self.step_per_um) as i32;
                let y = (pt.y * self.step_per_um) as i32;
                stream.extend_from_slice(&x.to_le_bytes());
                stream.extend_from_slice(&y.to_le_bytes());
            }

            // Close loop if closed
            if path.polyline.is_closed && pts.len() > 2 {
                stream.push(0x05);
                stream.extend_from_slice(&start_x.to_le_bytes());
                stream.extend_from_slice(&start_y.to_le_bytes());
            }

            // Command: Laser Off (0x06)
            stream.push(0x06);
        }

        // End of Job (0x00)
        stream.push(0x00);

        stream
    }
}
