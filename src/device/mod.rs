//! Device communication, controller driver, and laser motion interface for LaserBurn.
//! Supports GRBL (USB/Serial), Smoothieware, and virtual/offline controller modes.

use crate::geometry::Point2D;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerType {
    Grbl,
    Smoothieware,
    RuidaUdp,
    VirtualDemo,
}

impl ControllerType {
    pub fn name(&self) -> &'static str {
        match self {
            ControllerType::Grbl => "GRBL 1.1f / 1.1h (USB/Serial)",
            ControllerType::Smoothieware => "Smoothieware (USB/Serial)",
            ControllerType::RuidaUdp => "Ruida DSP (Ethernet/UDP)",
            ControllerType::VirtualDemo => "Virtual Device (Simulator)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Busy,
    Alarm,
}

/// Non-blocking state machine for safely stopping a physical job while preserving coordinate origin
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbortSequencePhase {
    HoldIssued { stopped_x: f64, stopped_y: f64, started_at: Instant },
    ResetIssued { stopped_x: f64, stopped_y: f64, started_at: Instant },
    RestoreSent { stopped_x: f64, stopped_y: f64, started_at: Instant },
}

/// Non-blocking state machine for physical limit switch homing & zero-point calibration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwitchHomingPhase {
    /// Initializing sequence (clear alarm, ensure laser off, enable hard limits $21=1)
    Starting { started_at: Instant },
    /// Moving X axis towards the X limit switch
    MovingX { started_at: Instant },
    /// X limit switch hit: pulling off 5mm to release switch
    PullingOffX { started_at: Instant },
    /// Moving Y axis towards the Y limit switch
    MovingY { started_at: Instant },
    /// Y limit switch hit: pulling off 5mm to release switch
    PullingOffY { started_at: Instant },
    /// Finalizing: set G90, G92 X0 Y0, update LaserBurn zero point
    Finalizing { started_at: Instant },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomBoundaries {
    #[serde(default)]
    pub enabled: bool,
    pub min_x: Option<f64>,
    pub max_x: Option<f64>,
    pub min_y: Option<f64>,
    pub max_y: Option<f64>,
    // Saved corner positions (X, Y in work coordinates)
    pub corner_bottom_left: Option<(f64, f64)>,
    pub corner_bottom_right: Option<(f64, f64)>,
    pub corner_top_left: Option<(f64, f64)>,
    pub corner_top_right: Option<(f64, f64)>,
}

impl Default for CustomBoundaries {
    fn default() -> Self {
        Self {
            enabled: false,
            min_x: None,
            max_x: None,
            min_y: None,
            max_y: None,
            corner_bottom_left: None,
            corner_bottom_right: None,
            corner_top_left: None,
            corner_top_right: None,
        }
    }
}

impl CustomBoundaries {
    pub fn update_limits(&mut self) {
        let mut xs = Vec::new();
        let mut ys = Vec::new();

        if let Some((x, y)) = self.corner_bottom_left { xs.push(x); ys.push(y); }
        if let Some((x, y)) = self.corner_bottom_right { xs.push(x); ys.push(y); }
        if let Some((x, y)) = self.corner_top_left { xs.push(x); ys.push(y); }
        if let Some((x, y)) = self.corner_top_right { xs.push(x); ys.push(y); }

        if !xs.is_empty() && !ys.is_empty() {
            let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            self.min_x = Some(min_x);
            self.max_x = Some(max_x);
            self.min_y = Some(min_y);
            self.max_y = Some(max_y);
        }
    }

    pub fn is_within(&self, x: f64, y: f64) -> bool {
        if !self.enabled {
            return true;
        }
        if let Some(min_x) = self.min_x {
            if x < min_x - 1e-3 { return false; }
        }
        if let Some(max_x) = self.max_x {
            if x > max_x + 1e-3 { return false; }
        }
        if let Some(min_y) = self.min_y {
            if y < min_y - 1e-3 { return false; }
        }
        if let Some(max_y) = self.max_y {
            if y > max_y + 1e-3 { return false; }
        }
        true
    }

    pub fn clamp_target(&self, x: f64, y: f64) -> (f64, f64) {
        if !self.enabled {
            return (x, y);
        }
        let cx = match (self.min_x, self.max_x) {
            (Some(min), Some(max)) => x.clamp(min, max),
            (Some(min), None) => x.max(min),
            (None, Some(max)) => x.min(max),
            (None, None) => x,
        };
        let cy = match (self.min_y, self.max_y) {
            (Some(min), Some(max)) => y.clamp(min, max),
            (Some(min), None) => y.max(min),
            (None, Some(max)) => y.min(max),
            (None, None) => y,
        };
        (cx, cy)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSettings {
    // Bed dimensions
    pub bed_width: f64,
    pub bed_height: f64,
    pub origin_corner: OriginCorner,

    // Physical Limit Switches
    #[serde(default)]
    pub enable_limit_switches: bool, // When false, all switch-based homing and buttons are disabled for machine protection

    // Learned Custom Boundaries (Soft Limits per Corner)
    #[serde(default)]
    pub custom_boundaries: CustomBoundaries,

    // Laser & Optics
    pub max_s_value: f64,          // e.g. 1000 for GRBL $30
    pub weak_light_percent: f64,   // e.g. 1.0% for aiming / framing without burning
    pub weak_light_enabled: bool,
    pub laser_type: LaserType,

    // Speeds
    pub rapid_speed: f64,          // G0 travel speed mm/min
    pub jog_speed: f64,            // Jog move speed mm/min
    pub frame_speed: f64,          // Test framing speed mm/min

    // Connection
    pub default_port: String,
    pub default_baud: u32,
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OriginCorner {
    BottomLeft,
    TopLeft,
    BottomRight,
    TopRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaserType {
    Diode,
    Co2,
    FiberGalvo,
}

impl DeviceSettings {
    pub fn config_path() -> Option<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;
        let dir = PathBuf::from(home).join(".config").join("laser_burn");
        let _ = std::fs::create_dir_all(&dir);
        Some(dir.join("settings.json"))
    }

    pub fn load_saved() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(settings) = serde_json::from_str::<DeviceSettings>(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        if cfg!(test) {
            return Ok(());
        }
        if let Some(path) = Self::config_path() {
            let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
            std::fs::write(&path, json).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Could not locate home directory to save settings".to_string())
        }
    }
}

impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            bed_width: 465.0,
            bed_height: 440.0,
            origin_corner: OriginCorner::TopRight, // Top-right corner (LightBurn & hardware home)
            enable_limit_switches: false, // Disabled by default for machine safety!
            custom_boundaries: CustomBoundaries::default(),

            max_s_value: 1000.0,
            weak_light_percent: 1.0, // 1% = S10, gentle diode glow
            weak_light_enabled: false,
            laser_type: LaserType::Diode,

            rapid_speed: 6000.0,
            jog_speed: 3000.0,
            frame_speed: 4000.0,

            default_port: "".to_string(),
            default_baud: 115200,
            auto_connect: false,
        }
    }
}

pub struct DeviceManager {
    pub settings: DeviceSettings,
    pub controller_type: ControllerType,
    pub selected_port: String,
    pub selected_baud: u32,
    pub status: ConnectionStatus,

    pub machine_x: f64,
    pub machine_y: f64,
    pub work_x: f64,
    pub work_y: f64,
    pub target_x: f64,
    pub target_y: f64,
    pub reported_x: f64,
    pub reported_y: f64,
    pub grbl_state: String,
    pub pending_ok_count: usize,
    pub framing_target_sent: bool,
    pub rx_buffer: String,
    #[cfg(feature = "gui")]
    last_status_poll: Option<Instant>,

    // Real-time limit switch & input pin state (GRBL Pn: reporting via $10=19)
    pub pin_state: String,
    pub pin_limit_x: bool,
    pub pin_limit_y: bool,
    pub pin_limit_z: bool,
    pub pin_probe: bool,

    pub is_weak_light_on: bool,
    pub is_framing_active: bool,
    pub framing_points: Vec<Point2D>,
    pub framing_idx: usize,

    // Work Coordinate Offset (WCO) & Stop Coordinate Retention
    pub last_wco: (f64, f64),
    pub abort_phase: Option<AbortSequencePhase>,
    pub homing_phase: Option<SwitchHomingPhase>,

    // Real-time Job Streaming
    pub is_job_running: bool,
    pub is_job_paused: bool,
    pub is_laser_firing: bool,
    pub job_lines: Vec<String>,
    pub job_idx: usize,
    pub job_total_lines: usize,
    pub job_progress: f32,
    pub current_command: String,

    pub console_history: VecDeque<String>,
    pub max_history: usize,

    #[cfg(feature = "gui")]
    active_port: Option<Box<dyn serialport::SerialPort>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let settings = DeviceSettings::load_saved();
        let default_port = settings.default_port.clone();
        let default_baud = settings.default_baud;

        let mut dm = Self {
            settings,
            controller_type: ControllerType::Grbl,
            selected_port: default_port.clone(),
            selected_baud: default_baud,
            status: ConnectionStatus::Disconnected, // Realistically starts disconnected

            machine_x: 0.0,
            machine_y: 0.0,
            work_x: 0.0,
            work_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            reported_x: 0.0,
            reported_y: 0.0,
            grbl_state: "Idle".to_string(),
            pending_ok_count: 0,
            framing_target_sent: false,
            rx_buffer: String::new(),
            #[cfg(feature = "gui")]
            last_status_poll: None,

            pin_state: String::new(),
            pin_limit_x: false,
            pin_limit_y: false,
            pin_limit_z: false,
            pin_probe: false,

            is_weak_light_on: false,
            is_framing_active: false,
            framing_points: Vec::new(),
            framing_idx: 0,

            last_wco: (0.0, 0.0),
            abort_phase: None,
            homing_phase: None,

            is_job_running: false,
            is_job_paused: false,
            is_laser_firing: false,
            job_lines: Vec::new(),
            job_idx: 0,
            job_total_lines: 0,
            job_progress: 0.0,
            current_command: "".to_string(),

            console_history: VecDeque::new(),
            max_history: 100,

            #[cfg(feature = "gui")]
            active_port: None,
        };

        dm.log("[LaserBurn] Controller subsystem initialized.");

        // Auto-connect to previous device if enabled and port exists
        if dm.settings.auto_connect && !default_port.trim().is_empty() {
            if std::path::Path::new(&default_port).exists() {
                dm.log(&format!("[AUTO-CONNECT] ⚡ Connecting to previous device: {} ({} baud)...", default_port, default_baud));
                let _ = dm.connect(&default_port, default_baud);
            } else {
                dm.log(&format!("[AUTO-CONNECT] ○ Previous port {} is not connected. Connect laser USB and power.", default_port));
            }
        } else {
            dm.log("[CONNECTION] ○ No device connected. Connect laser power and USB cable, then click Connect.");
        }

        dm
    }

    pub fn log(&mut self, line: &str) {
        if self.console_history.len() >= self.max_history {
            self.console_history.pop_front();
        }
        self.console_history.push_back(line.to_string());

        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/laserburn.log") {
            let _ = writeln!(f, "{}", line);
        }
    }

    /// Checks if a physical hardware serial port is currently open
    pub fn is_physically_connected(&self) -> bool {
        #[cfg(feature = "gui")]
        {
            self.active_port.is_some()
        }
        #[cfg(not(feature = "gui"))]
        false
    }

    /// Enumerate all currently available system serial / USB ports
    pub fn scan_available_ports() -> Vec<String> {
        let mut ports = Vec::new();

        #[cfg(feature = "gui")]
        {
            if let Ok(available) = serialport::available_ports() {
                for p in available {
                    ports.push(p.port_name);
                }
            }
        }

        // Add macOS / Linux standard laser port candidates ONLY if the device file actually exists on filesystem
        let standard_candidates = [
            "/dev/tty.usbserial-10",
            "/dev/tty.usbserial-11",
            "/dev/tty.usbserial-0001",
            "/dev/tty.usbmodem101",
            "/dev/tty.wchusbserial1410",
            "/dev/cu.usbserial",
            "/dev/ttyUSB0",
            "/dev/ttyUSB1",
            "/dev/ttyACM0",
            "/dev/ttyACM1",
        ];

        for &c in &standard_candidates {
            if std::path::Path::new(c).exists() && !ports.contains(&c.to_string()) {
                ports.push(c.to_string());
            }
        }

        // Always provide a Virtual demo port for testing without laser hardware
        if !ports.contains(&"Virtual Port (Demo / Offline)".to_string()) {
            ports.push("Virtual Port (Demo / Offline)".to_string());
        }

        ports
    }

    /// Connect to selected serial port
    pub fn connect(&mut self, port_name: &str, baud: u32) -> Result<(), String> {
        self.selected_port = port_name.to_string();
        self.selected_baud = baud;

        if port_name == "Virtual Port (Demo / Offline)" || port_name == "Virtuaaliportti (Demo / Offline)" || self.controller_type == ControllerType::VirtualDemo {
            self.status = ConnectionStatus::Connected;
            self.log(&format!("[SIMULATOR] Connected to virtual controller ({} baud, offline mode).", baud));
            self.log("[GRBL] Grbl 1.1h ['$' for help]");
            return Ok(());
        }

        #[cfg(feature = "gui")]
        {
            if !port_name.is_empty() {
                self.log(&format!("[CONNECTION] Connecting to port {} ({} baud)...", port_name, baud));
                match serialport::new(port_name, baud)
                    .timeout(Duration::from_millis(100))
                    .open()
                {
                    Ok(port) => {
                        self.active_port = Some(port);
                        self.status = ConnectionStatus::Connected;
                        self.rx_buffer.clear();
                        self.pending_ok_count = 0;
                        self.framing_target_sent = false;
                        self.grbl_state = "Idle".to_string();
                        self.abort_phase = None;
                        self.homing_phase = None;

                        // Remember successful physical connection and auto-save settings
                        if port_name != "Virtual Port (Demo / Offline)" && port_name != "Virtuaaliportti (Demo / Offline)" {
                            self.settings.default_port = port_name.to_string();
                            self.settings.default_baud = baud;
                            self.settings.auto_connect = true;
                            let _ = self.settings.save();
                        }

                        self.log(&format!("[CONNECTION] ✅ Successfully connected to device: {} ({} baud)", port_name, baud));
                        // Send wake-up reset to GRBL, enable pin reporting ($10=19: MPos+Buffers+PinState), hard limits ($21=1) and immediate status poll
                        self.send_raw("\r\n\r\n");
                        self.send_raw("$10=19");
                        self.send_raw("$21=1");
                        self.send_raw("$$");
                        self.send_realtime(b'?');
                        return Ok(());
                    }
                    Err(e) => {
                        self.active_port = None;
                        self.status = ConnectionStatus::Disconnected;
                        let err_msg = format!("Cannot connect to port {}: {}. Ensure laser is powered on and USB cable is connected!", port_name, e);
                        self.log(&format!("[ERROR] ❌ {}", err_msg));
                        return Err(err_msg);
                    }
                }
            }
        }

        self.status = ConnectionStatus::Disconnected;
        let err_msg = "Select a port before connecting".to_string();
        self.log(&format!("[WARNING] {}", err_msg));
        Err(err_msg)
    }

    /// Disconnect from port
    pub fn disconnect(&mut self) {
        #[cfg(feature = "gui")]
        {
            self.active_port = None;
        }
        self.status = ConnectionStatus::Disconnected;
        self.is_weak_light_on = false;
        self.is_framing_active = false;
        self.abort_phase = None;
        self.homing_phase = None;
        self.log("[CONNECTION] Disconnected.");
    }

    /// Send raw string command to controller
    pub fn send_raw(&mut self, cmd: &str) {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() {
            self.log(&format!("> {}", trimmed));
        }

        #[cfg(feature = "gui")]
        {
            if let Some(ref mut port) = self.active_port {
                use std::io::Write;
                let _ = writeln!(port, "{}", trimmed);
                let _ = port.flush();
            }
        }

        // Mock echo response for virtual mode
        if !self.is_physically_connected() {
            if trimmed == "$$" {
                self.log("$0=10 (Step pulse time, microseconds)");
                self.log("$1=25 (Step idle delay, milliseconds)");
                self.log("$30=1000 (Max spindle speed, RPM/S-value)");
                self.log("$31=0 (Min spindle speed, RPM)");
                self.log("$32=1 (Laser-mode enable)");
                self.log("$100=80.000 (X-axis travel resolution, step/mm)");
                self.log("$101=80.000 (Y-axis travel resolution, step/mm)");
                self.log("$110=6000.000 (X-axis maximum rate, mm/min)");
                self.log("$111=6000.000 (Y-axis maximum rate, mm/min)");
                self.log("$130=400.000 (X-axis maximum travel, mm)");
                self.log("$131=400.000 (Y-axis maximum travel, mm)");
                self.log("ok");
            } else if trimmed == "?" {
                self.log(&format!("<Idle|MPos:{:.3},{:.3},0.000|FS:0,0|WCO:0.000,0.000,0.000>", self.machine_x, self.machine_y));
                self.log("ok");
            } else if trimmed.starts_with("$J=") || trimmed.starts_with("G0") || trimmed.starts_with("G1") || trimmed == "$H" || trimmed.starts_with("G92") {
                self.log("ok");
            }
        }
    }

    /// Send a single real-time byte directly without newline (e.g. '?' for status, '!' for feed hold, '~' for cycle start)
    pub fn send_realtime(&mut self, byte: u8) {
        #[cfg(feature = "gui")]
        {
            if let Some(ref mut port) = self.active_port {
                use std::io::Write;
                let _ = port.write_all(&[byte]);
                let _ = port.flush();
            }
        }
    }

    /// Direction vector (dir_x, dir_y) towards the limit switches of the configured OriginCorner
    pub fn origin_corner_direction(&self) -> (f64, f64) {
        match self.settings.origin_corner {
            // For TopRight with $3=3 inverted axes:
            // -X moves Right towards switch, -Y moves Up/Rear towards switch
            OriginCorner::TopRight => (-1.0, -1.0),
            OriginCorner::TopLeft => (1.0, -1.0),
            OriginCorner::BottomRight => (-1.0, 1.0),
            OriginCorner::BottomLeft => (1.0, 1.0),
        }
    }

    /// UI arrow jog helper: translates screen arrow directions (left/right, up/down)
    /// to machine coordinate delta based on origin corner so arrows always move head in visual direction.
    pub fn jog_arrow(&mut self, arrow_x: f64, arrow_y: f64, step: f64, feed: f64) {
        let (dir_x, dir_y) = match self.settings.origin_corner {
            OriginCorner::TopRight => (-arrow_x, -arrow_y),
            OriginCorner::TopLeft => (arrow_x, -arrow_y),
            OriginCorner::BottomRight => (-arrow_x, arrow_y),
            OriginCorner::BottomLeft => (arrow_x, arrow_y),
        };
        self.jog(dir_x * step, dir_y * step, feed);
    }

    /// Real-time JOG move (GRBL $J=G91 G21 X... Y... F...)
    pub fn jog(&mut self, dx: f64, dy: f64, feed: f64) {
        if dx.abs() < 1e-4 && dy.abs() < 1e-4 {
            return;
        }

        // Enforce custom learned boundaries (soft limits)
        let (eff_dx, eff_dy) = if self.settings.custom_boundaries.enabled {
            let cur_x = if self.is_physically_connected() { self.work_x } else { self.target_x };
            let cur_y = if self.is_physically_connected() { self.work_y } else { self.target_y };

            let target_x = cur_x + dx;
            let target_y = cur_y + dy;

            if !self.settings.custom_boundaries.is_within(target_x, target_y) {
                let (clamped_x, clamped_y) = self.settings.custom_boundaries.clamp_target(target_x, target_y);
                let allowed_dx = clamped_x - cur_x;
                let allowed_dy = clamped_y - cur_y;

                if allowed_dx.abs() < 1e-4 && allowed_dy.abs() < 1e-4 {
                    self.log(&format!(
                        "⚠️ Motion blocked: Laser head reached soft limit (X: {:.1}, Y: {:.1} mm)!",
                        cur_x, cur_y
                    ));
                    return;
                }
                (allowed_dx, allowed_dy)
            } else {
                (dx, dy)
            }
        } else {
            (dx, dy)
        };

        if !self.is_physically_connected() {
            let mut new_x = (self.target_x + eff_dx).clamp(0.0, self.settings.bed_width);
            let mut new_y = (self.target_y + eff_dy).clamp(0.0, self.settings.bed_height);
            if self.settings.custom_boundaries.enabled {
                let (cx, cy) = self.settings.custom_boundaries.clamp_target(new_x, new_y);
                new_x = cx;
                new_y = cy;
            }
            self.target_x = new_x;
            self.target_y = new_y;
            self.machine_x = new_x;
            self.machine_y = new_y;
            self.work_x = new_x;
            self.work_y = new_y;
            return;
        }

        // If controller is currently in Alarm lock, unlock it automatically with $X so jog can proceed!
        if self.grbl_state.starts_with("Alarm") {
            self.send_raw("$X");
        }

        // Clean command formatting: only include axes that actually move (prevents Y-0.000 / X-0.000)
        let cmd = if eff_dx.abs() >= 1e-4 && eff_dy.abs() >= 1e-4 {
            format!("$J=G91 X{:.3} Y{:.3} F{:.0}", eff_dx, eff_dy, feed)
        } else if eff_dx.abs() >= 1e-4 {
            format!("$J=G91 X{:.3} F{:.0}", eff_dx, feed)
        } else {
            format!("$J=G91 Y{:.3} F{:.0}", eff_dy, feed)
        };
        self.send_raw(&cmd);
        self.send_realtime(b'?');
    }

    /// Save current laser head position as a named boundary corner
    pub fn save_boundary_corner(&mut self, corner_name: &str) {
        let x = if self.is_physically_connected() { self.work_x } else { self.target_x };
        let y = if self.is_physically_connected() { self.work_y } else { self.target_y };

        match corner_name {
            "bottom_left" => self.settings.custom_boundaries.corner_bottom_left = Some((x, y)),
            "bottom_right" => self.settings.custom_boundaries.corner_bottom_right = Some((x, y)),
            "top_left" => self.settings.custom_boundaries.corner_top_left = Some((x, y)),
            "top_right" => self.settings.custom_boundaries.corner_top_right = Some((x, y)),
            _ => {}
        }
        self.settings.custom_boundaries.update_limits();
        self.settings.custom_boundaries.enabled = true; // Auto-enable when a corner is taught!
        let _ = self.settings.save();
        self.log(&format!("📍 Taught corner [{}]: X={:.2}, Y={:.2} mm", corner_name, x, y));
    }

    /// Clear all custom learned boundary corners
    pub fn clear_boundary_corners(&mut self) {
        self.settings.custom_boundaries = CustomBoundaries::default();
        let _ = self.settings.save();
        self.log("🗑️ Custom learned boundaries cleared.");
    }

    /// Home the laser ($H) / Reset to mechanical switches
    pub fn home(&mut self) {
        if !self.settings.enable_limit_switches {
            self.log("[HOME] ⚠️ Physical limit switches disabled in settings. Enable in: ⚙️ Settings.");
            return;
        }
        self.start_mechanical_home_reset();
    }

    /// Set current position as Work Origin (0, 0)
    pub fn set_origin_here(&mut self) {
        self.target_x = 0.0;
        self.target_y = 0.0;
        self.machine_x = 0.0;
        self.machine_y = 0.0;
        self.work_x = 0.0;
        self.work_y = 0.0;
        self.last_wco = (0.0, 0.0);
        self.send_raw("G92 X0 Y0");
        self.send_realtime(b'?');
        self.log("[ORIGIN] 🎯 Current head position successfully calibrated as Work Origin (0,0) (G92 X0 Y0)!");
    }

    /// Go to Work Origin (0, 0)
    pub fn go_to_origin(&mut self) {
        self.go_to_position(0.0, 0.0);
    }

    /// Get target machine coordinates for a corner ("top_left", "top_right", "bottom_left", "bottom_right")
    pub fn get_corner_coordinates(&self, corner_name: &str) -> (f64, f64) {
        // 1. If the user has saved custom taught corners, use them:
        match corner_name {
            "top_left" => {
                if let Some(pt) = self.settings.custom_boundaries.corner_top_left {
                    return pt;
                }
            }
            "top_right" => {
                if let Some(pt) = self.settings.custom_boundaries.corner_top_right {
                    return pt;
                }
            }
            "bottom_left" => {
                if let Some(pt) = self.settings.custom_boundaries.corner_bottom_left {
                    return pt;
                }
            }
            "bottom_right" => {
                if let Some(pt) = self.settings.custom_boundaries.corner_bottom_right {
                    return pt;
                }
            }
            _ => {}
        }

        // 2. Fallback to machine bed bounds based on origin corner:
        let doc_pt = match corner_name {
            "top_left" => Point2D::new(0.0, 0.0),
            "top_right" => Point2D::new(self.settings.bed_width, 0.0),
            "bottom_left" => Point2D::new(0.0, self.settings.bed_height),
            "bottom_right" => Point2D::new(self.settings.bed_width, self.settings.bed_height),
            _ => Point2D::new(0.0, 0.0),
        };
        let m = self.doc_to_machine(doc_pt);
        (m.x, m.y)
    }

    /// Move laser head to a specific corner ("top_left", "top_right", "bottom_left", "bottom_right")
    pub fn go_to_corner(&mut self, corner_name: &str) {
        let (raw_x, raw_y) = self.get_corner_coordinates(corner_name);
        self.go_to_position(raw_x, raw_y);
    }

    /// Move laser head to specific absolute position (G0 G90 X... Y... F...)
    pub fn go_to_position(&mut self, x: f64, y: f64) {
        let (target_x, target_y) = if self.settings.custom_boundaries.enabled {
            self.settings.custom_boundaries.clamp_target(x, y)
        } else {
            (
                x.clamp(0.0, self.settings.bed_width),
                y.clamp(0.0, self.settings.bed_height),
            )
        };

        self.target_x = target_x;
        self.target_y = target_y;
        let feed = self.settings.rapid_speed;

        // If in Alarm state, unlock first:
        if self.grbl_state.starts_with("Alarm") {
            self.send_raw("$X");
        }

        self.send_raw(&format!("G90 G0 X{:.3} Y{:.3} F{:.0}", target_x, target_y, feed));
        self.send_realtime(b'?');
        self.log(&format!(
            "[JOG] 🎯 Moving to position: X={:.1}, Y={:.1} mm...",
            target_x, target_y
        ));

        if !self.is_physically_connected() {
            self.machine_x = target_x;
            self.machine_y = target_y;
            self.work_x = target_x;
            self.work_y = target_y;
        }
    }

    /// Convert document coordinates (where X=0 is left, Y=0 is top)
    /// to machine coordinates relative to machine's OriginCorner setting.
    pub fn doc_to_machine(&self, pt: Point2D) -> Point2D {
        match self.settings.origin_corner {
            OriginCorner::TopLeft => Point2D::new(
                pt.x.clamp(0.0, self.settings.bed_width),
                pt.y.clamp(0.0, self.settings.bed_height),
            ),
            OriginCorner::TopRight => Point2D::new(
                (self.settings.bed_width - pt.x).clamp(0.0, self.settings.bed_width),
                pt.y.clamp(0.0, self.settings.bed_height),
            ),
            OriginCorner::BottomLeft => Point2D::new(
                pt.x.clamp(0.0, self.settings.bed_width),
                (self.settings.bed_height - pt.y).clamp(0.0, self.settings.bed_height),
            ),
            OriginCorner::BottomRight => Point2D::new(
                (self.settings.bed_width - pt.x).clamp(0.0, self.settings.bed_width),
                (self.settings.bed_height - pt.y).clamp(0.0, self.settings.bed_height),
            ),
        }
    }

    /// Convert machine coordinates (relative to machine OriginCorner) to document coordinates
    pub fn machine_to_doc(&self, m_pt: Point2D) -> Point2D {
        self.doc_to_machine(m_pt)
    }

    /// Go to arbitrary document point (converts from document coordinates to machine coordinates)
    pub fn go_to_point(&mut self, pt: Point2D) {
        let m_pt = self.doc_to_machine(pt);
        self.target_x = m_pt.x;
        self.target_y = m_pt.y;
        let feed = self.settings.rapid_speed;
        self.send_raw(&format!("G90 G0 X{:.3} Y{:.3} F{:.0}", self.target_x, self.target_y, feed));
        self.log(&format!("[CONTROLLER] Moving to point (X{:.1} Y{:.1})...", self.target_x, self.target_y));
        if !self.is_physically_connected() {
            self.machine_x = self.target_x;
            self.machine_y = self.target_y;
            self.work_x = self.target_x;
            self.work_y = self.target_y;
        }
    }

    /// Toggle Weak Light on or off
    /// Uses M3 S<power> in laser mode ($32=1) for gentle diode beam spot
    pub fn toggle_weak_light(&mut self) -> bool {
        self.set_weak_light(!self.is_weak_light_on)
    }

    /// Set Weak Light state
    pub fn set_weak_light(&mut self, enabled: bool) -> bool {
        self.is_weak_light_on = enabled;
        if self.is_weak_light_on {
            let s_val = ((self.settings.weak_light_percent / 100.0) * self.settings.max_s_value).max(1.0).round() as u32;
            // GRBL laser mode requires G1 motion state to emit power
            self.send_raw(&format!("M3 S{} G1 F100", s_val));
            self.log(&format!("[BEAM] 🔦 Weak beam turned on for alignment ({:.1}% power, S{})", self.settings.weak_light_percent, s_val));
        } else {
            self.send_raw("M5");
            self.log("[BEAM] ⭕ Weak beam turned off (M5).");
        }
        self.is_weak_light_on
    }

    /// Start Bounding Box Frame Test with Weak Light
    pub fn start_box_frame_test(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        let pts = vec![
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, min_y),
            Point2D::new(max_x, max_y),
            Point2D::new(min_x, max_y),
            Point2D::new(min_x, min_y),
        ];

        self.start_points_frame_test(pts, "🔲 Box Framing");
    }

    /// Start Contour / Path Trace Test with Weak Light
    pub fn start_contour_frame_test(&mut self, path_points: Vec<Point2D>) {
        if path_points.is_empty() {
            self.log("[ERROR] No contour points to test.");
            return;
        }
        self.start_points_frame_test(path_points, "✨ Contour Framing");
    }

    fn start_points_frame_test(&mut self, points: Vec<Point2D>, test_name: &str) {
        if points.is_empty() {
            return;
        }

        self.framing_points = points.into_iter().map(|p| self.doc_to_machine(p)).collect();
        self.framing_idx = 0;
        self.framing_target_sent = false;
        self.is_framing_active = true;

        // Turn on weak light
        self.set_weak_light(true);

        self.log(&format!("[TEST] {} started ({} points). Laser tracing contour with weak beam...", test_name, self.framing_points.len()));
    }

    /// Advance the test framing simulation / motion
    pub fn step_frame_test(&mut self) -> bool {
        if !self.is_framing_active || self.framing_points.is_empty() {
            return false;
        }

        self.poll_serial();

        if self.framing_idx < self.framing_points.len() {
            let target = self.framing_points[self.framing_idx];

            if !self.framing_target_sent {
                let feed = self.settings.frame_speed;
                self.target_x = target.x;
                self.target_y = target.y;
                let cmd_type = if self.framing_idx == 0 { "G0" } else { "G1" };
                self.send_raw(&format!("G90 {} X{:.3} Y{:.3} F{:.0}", cmd_type, target.x, target.y, feed));
                self.framing_target_sent = true;

                if !self.is_physically_connected() {
                    self.machine_x = target.x;
                    self.machine_y = target.y;
                    self.work_x = target.x;
                    self.work_y = target.y;
                }
            }

            // Check if head has arrived at target
            let arrived = if self.is_physically_connected() {
                let dist = ((self.machine_x - target.x).powi(2) + (self.machine_y - target.y).powi(2)).sqrt();
                dist < 1.0 || (self.grbl_state == "Idle" && self.framing_target_sent)
            } else {
                true
            };

            if arrived {
                self.framing_idx += 1;
                self.framing_target_sent = false;
            }
            true
        } else {
            // Finished framing test
            self.stop_frame_test();
            self.log("[TEST] ✅ Framing test completed successfully!");
            false
        }
    }

    /// Stop test framing and turn off weak light
    pub fn stop_frame_test(&mut self) {
        self.is_framing_active = false;
        self.framing_points.clear();
        self.framing_idx = 0;
        self.framing_target_sent = false;
        self.set_weak_light(false);
    }

    /// Step non-blocking abort sequence to flush planner buffer and preserve coordinate origin
    pub fn step_abort_sequence(&mut self) {
        if let Some(phase) = self.abort_phase {
            match phase {
                AbortSequencePhase::HoldIssued { stopped_x, stopped_y, started_at } => {
                    // Wait 40ms for physical deceleration to halt motion
                    if started_at.elapsed() >= Duration::from_millis(40) {
                        // Soft-reset GRBL (0x18) to flush planner buffer
                        self.send_realtime(0x18);
                        self.abort_phase = Some(AbortSequencePhase::ResetIssued {
                            stopped_x,
                            stopped_y,
                            started_at: Instant::now(),
                        });
                    }
                }
                AbortSequencePhase::ResetIssued { stopped_x, stopped_y, started_at } => {
                    // Wait 60ms for GRBL microcontroller to complete soft reset boot
                    if started_at.elapsed() >= Duration::from_millis(60) {
                        // Clear any alarm lock ($X), force laser off (M5), absolute mode (G90)
                        // and re-establish the coordinate reference (G92) so (0,0) remains the original origin!
                        self.send_raw("$X");
                        self.send_raw("M5");
                        self.send_raw("G90");
                        self.send_raw(&format!("G92 X{:.3} Y{:.3}", stopped_x, stopped_y));
                        self.abort_phase = Some(AbortSequencePhase::RestoreSent {
                            stopped_x,
                            stopped_y,
                            started_at: Instant::now(),
                        });
                        self.log(&format!("[CONTROLLER] Work coordinate origin (0,0) preserved. Laser head at X={:.2}, Y={:.2} mm.", stopped_x, stopped_y));
                    }
                }
                AbortSequencePhase::RestoreSent { stopped_x, stopped_y, started_at } => {
                    // Allow G92 command to be processed by controller
                    if started_at.elapsed() >= Duration::from_millis(100) {
                        self.abort_phase = None;
                        self.target_x = stopped_x;
                        self.target_y = stopped_y;
                        self.machine_x = stopped_x;
                        self.machine_y = stopped_y;
                        self.work_x = stopped_x;
                        self.work_y = stopped_y;
                        self.send_realtime(b'?');
                    }
                }
            }
        }
    }

    /// Starts physical mechanical limit switch homing and zero-point calibration.
    /// Drives X and Y sequentially towards the configured OriginCorner switches regardless of software limits,
    /// stops when the mechanical switches trip, pulls off to release contact, and sets G92 X0 Y0.
    pub fn start_mechanical_home_reset(&mut self) {
        if !self.settings.enable_limit_switches {
            self.log("[RESET] ⚠️ Physical limit switches disabled in settings (switches disabled for machine protection). Enable in: ⚙️ Settings.");
            return;
        }
        if self.homing_phase.is_some() {
            return;
        }

        // Halt any running cutting job or framing test
        self.is_job_running = false;
        self.is_job_paused = false;
        self.is_laser_firing = false;
        self.is_framing_active = false;
        self.job_lines.clear();
        self.set_weak_light(false);

        let corner = self.settings.origin_corner;

        if !self.is_physically_connected() {
            // Virtual Demo / Simulator mode:
            self.log(&format!("[RESET] 🔄 Virtual Mode: Moving head to {:?} origin switches (X0 Y0)...", corner));
            self.target_x = 0.0;
            self.target_y = 0.0;
            self.machine_x = 0.0;
            self.machine_y = 0.0;
            self.work_x = 0.0;
            self.work_y = 0.0;
            self.last_wco = (0.0, 0.0);
            self.homing_phase = None;
            return;
        }

        self.log(&format!("[RESET] 🔄 Starting hardware reset towards {:?} limit switches...", corner));
        if self.grbl_state.starts_with("Alarm") {
            self.send_raw("$X");
        }
        self.send_raw("M5");
        self.send_raw("$21=1");
        self.homing_phase = Some(SwitchHomingPhase::Starting { started_at: Instant::now() });
    }

    /// Step non-blocking mechanical limit switch homing & zero-point calibration sequence
    pub fn step_homing_sequence(&mut self) {
        let (dir_x, dir_y) = self.origin_corner_direction();
        let travel_x = (self.settings.bed_width * 1.2).max(100.0) * dir_x;
        let travel_y = (self.settings.bed_height * 1.2).max(100.0) * dir_y;
        let pull_x = -dir_x * 5.0;
        let pull_y = -dir_y * 5.0;

        if let Some(phase) = self.homing_phase {
            match phase {
                SwitchHomingPhase::Starting { started_at } => {
                    if started_at.elapsed() >= Duration::from_millis(100) {
                        self.log(&format!("[RESET] 🔄 Moving X axis towards limit switch ({:+.1} mm)...", travel_x));
                        self.send_raw(&format!("G91 G1 X{:.1} F1200", travel_x));
                        self.homing_phase = Some(SwitchHomingPhase::MovingX { started_at: Instant::now() });
                    }
                }
                SwitchHomingPhase::MovingX { started_at } => {
                    let alarm_tripped = self.grbl_state.starts_with("Alarm") || self.pin_limit_x;
                    let timed_out = started_at.elapsed() >= Duration::from_secs(35);

                    if alarm_tripped || timed_out {
                        self.send_realtime(b'!');
                        if alarm_tripped {
                            self.log(&format!("[RESET] 🛑 X limit switch triggered (Alarm/Pn:{}). Motion stopped. Pulling off 5 mm...", self.pin_state));
                        } else {
                            self.log("[RESET] ⚠️ Search distance exceeded (35s timeout). Pulling off 5 mm...");
                        }
                        self.send_raw("$X");
                        self.send_raw("$21=0");
                        self.send_raw(&format!("G91 G0 X{:.1} F500", pull_x));
                        self.homing_phase = Some(SwitchHomingPhase::PullingOffX { started_at: Instant::now() });
                    }
                }
                SwitchHomingPhase::PullingOffX { started_at } => {
                    if started_at.elapsed() >= Duration::from_millis(800) {
                        self.send_raw("$21=1");
                        self.send_raw("$X");
                        self.log(&format!("[RESET] ✅ X axis switch disengaged. Moving Y axis towards limit switch ({:+.1} mm)...", travel_y));
                        self.send_raw(&format!("G91 G1 Y{:.1} F1200", travel_y));
                        self.homing_phase = Some(SwitchHomingPhase::MovingY { started_at: Instant::now() });
                    }
                }
                SwitchHomingPhase::MovingY { started_at } => {
                    let alarm_tripped = self.grbl_state.starts_with("Alarm") || self.pin_limit_y;
                    let timed_out = started_at.elapsed() >= Duration::from_secs(35);

                    if alarm_tripped || timed_out {
                        self.send_realtime(b'!');
                        if alarm_tripped {
                            self.log(&format!("[RESET] 🛑 Y limit switch triggered (Alarm/Pn:{}). Motion stopped. Pulling off 5 mm...", self.pin_state));
                        } else {
                            self.log("[RESET] ⚠️ Search distance exceeded (35s timeout). Pulling off 5 mm...");
                        }
                        self.send_raw("$X");
                        self.send_raw("$21=0");
                        self.send_raw(&format!("G91 G0 Y{:.1} F500", pull_y));
                        self.homing_phase = Some(SwitchHomingPhase::PullingOffY { started_at: Instant::now() });
                    }
                }
                SwitchHomingPhase::PullingOffY { started_at } => {
                    if started_at.elapsed() >= Duration::from_millis(800) {
                        self.send_raw("$21=1");
                        self.send_raw("$X");
                        self.send_raw("M5");
                        self.send_raw("G90");
                        self.send_raw("G92 X0 Y0");
                        self.homing_phase = Some(SwitchHomingPhase::Finalizing { started_at: Instant::now() });
                    }
                }
                SwitchHomingPhase::Finalizing { started_at } => {
                    if started_at.elapsed() >= Duration::from_millis(200) {
                        self.homing_phase = None;
                        self.machine_x = 0.0;
                        self.machine_y = 0.0;
                        self.work_x = 0.0;
                        self.work_y = 0.0;
                        self.target_x = 0.0;
                        self.target_y = 0.0;
                        self.last_wco = (0.0, 0.0);
                        self.send_realtime(b'?');
                        self.log("[RESET] 🎯 Zero point (0,0) successfully calibrated and saved at limit switches!");
                    }
                }
            }
        }
    }

    /// Poll serial port for incoming responses (non-blocking) and query status
    pub fn poll_serial(&mut self) {
        self.step_abort_sequence();
        self.step_homing_sequence();

        #[cfg(feature = "gui")]
        {
            if self.is_physically_connected() {
                let now = Instant::now();
                let poll_interval = if self.is_job_running || self.is_framing_active || self.homing_phase.is_some() || self.grbl_state == "Run" || self.grbl_state == "Jog" || self.grbl_state == "Home" {
                    Duration::from_millis(50)
                } else {
                    Duration::from_millis(150)
                };

                if self.last_status_poll.map_or(true, |t| now.duration_since(t) >= poll_interval) {
                    self.last_status_poll = Some(now);
                    self.send_realtime(b'?');
                }

                if let Some(ref mut port) = self.active_port {
                    if let Ok(available) = port.bytes_to_read() {
                        if available > 0 {
                            use std::io::Read;
                            let mut buf = vec![0u8; available as usize];
                            if let Ok(n) = port.read(&mut buf) {
                                if n > 0 {
                                    let s = String::from_utf8_lossy(&buf[..n]);
                                    self.rx_buffer.push_str(&s);
                                }
                            }
                        }
                    }
                }

                // Process complete lines from rx_buffer
                while let Some(idx) = self.rx_buffer.find(|c| c == '\n' || c == '\r') {
                    let line = self.rx_buffer[..idx].trim().to_string();
                    self.rx_buffer = self.rx_buffer[idx + 1..].to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if line.starts_with('<') && line.ends_with('>') {
                        self.parse_grbl_status(&line);
                    } else {
                        self.log(&line);
                        if line == "ok" {
                            self.pending_ok_count = self.pending_ok_count.saturating_sub(1);
                        } else if line.contains("ALARM:1") || line.contains("ALARM") {
                            self.grbl_state = "Alarm".to_string();
                            let (dir_x, dir_y) = self.origin_corner_direction();
                            let pull_x = -dir_x * 5.0;
                            let pull_y = -dir_y * 5.0;

                            if matches!(self.homing_phase, Some(SwitchHomingPhase::MovingX { .. })) {
                                self.log("[RESET] 🛑 X limit switch triggered (ALARM:1). Motion stopped. Pulling off 5 mm...");
                                self.send_raw("$X");
                                self.send_raw("$21=0");
                                self.send_raw(&format!("G91 G0 X{:.1} F500", pull_x));
                                self.homing_phase = Some(SwitchHomingPhase::PullingOffX { started_at: Instant::now() });
                            } else if matches!(self.homing_phase, Some(SwitchHomingPhase::MovingY { .. })) {
                                self.log("[RESET] 🛑 Y limit switch triggered (ALARM:1). Motion stopped. Pulling off 5 mm...");
                                self.send_raw("$X");
                                self.send_raw("$21=0");
                                self.send_raw(&format!("G91 G0 Y{:.1} F500", pull_y));
                                self.homing_phase = Some(SwitchHomingPhase::PullingOffY { started_at: Instant::now() });
                            }
                        }
                    }
                }

                if self.rx_buffer.len() > 2048 {
                    self.rx_buffer.clear();
                }
            }
        }
    }

    /// Parse GRBL real-time status report `<State|MPos:X,Y,Z|...>`
    pub fn parse_grbl_status(&mut self, status_line: &str) {
        let inner = &status_line[1..status_line.len() - 1];
        let parts: Vec<&str> = inner.split('|').collect();
        if parts.is_empty() {
            return;
        }

        self.grbl_state = parts[0].to_string();

        let mut parsed_mpos = None;
        let mut parsed_wpos = None;
        let mut found_pn = false;

        for part in &parts[1..] {
            if let Some(coords_str) = part.strip_prefix("MPos:") {
                let coords: Vec<&str> = coords_str.split(',').collect();
                if coords.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                        parsed_mpos = Some((x, y));
                    }
                }
            } else if let Some(coords_str) = part.strip_prefix("WPos:") {
                let coords: Vec<&str> = coords_str.split(',').collect();
                if coords.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                        parsed_wpos = Some((x, y));
                    }
                }
            } else if let Some(coords_str) = part.strip_prefix("WCO:") {
                let coords: Vec<&str> = coords_str.split(',').collect();
                if coords.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                        self.last_wco = (x, y);
                    }
                }
            } else if let Some(pn_str) = part.strip_prefix("Pn:") {
                found_pn = true;
                if self.pin_state != pn_str {
                    self.log(&format!("[PIN] ⚡ Switch state changed: Pn:{}", pn_str));
                }
                self.pin_state = pn_str.to_string();
                self.pin_limit_x = pn_str.contains('X');
                self.pin_limit_y = pn_str.contains('Y');
                self.pin_limit_z = pn_str.contains('Z');
                self.pin_probe = pn_str.contains('P');
            } else if let Some(fs_str) = part.strip_prefix("FS:") {
                let fs: Vec<&str> = fs_str.split(',').collect();
                if fs.len() >= 2 {
                    if let Ok(s) = fs[1].parse::<f64>() {
                        if s > 0.0 && (self.grbl_state == "Run" || self.grbl_state == "Jog") {
                            self.is_laser_firing = true;
                        } else if s == 0.0 {
                            self.is_laser_firing = false;
                        }
                    }
                }
            }
        }

        if !found_pn {
            self.pin_state.clear();
            self.pin_limit_x = false;
            self.pin_limit_y = false;
            self.pin_limit_z = false;
            self.pin_probe = false;
        }

        // Determine real-time position
        let (raw_x, raw_y) = if let Some((wx, wy)) = parsed_wpos {
            (wx, wy)
        } else if let Some((mx, my)) = parsed_mpos {
            (mx - self.last_wco.0, my - self.last_wco.1)
        } else {
            return;
        };

        // Coordinates in machine bed workspace:
        let real_x = raw_x.abs().clamp(0.0, self.settings.bed_width);
        let real_y = raw_y.abs().clamp(0.0, self.settings.bed_height);

        // Protection during stop / abort sequence:
        // When GRBL resets, it briefly reports 0,0 before our G92 restore takes effect.
        // Prevent that transient 0,0 from overwriting our preserved coordinates!
        if let Some(phase) = self.abort_phase {
            let (sx, sy) = match phase {
                AbortSequencePhase::HoldIssued { stopped_x, stopped_y, .. } => (stopped_x, stopped_y),
                AbortSequencePhase::ResetIssued { stopped_x, stopped_y, .. } => (stopped_x, stopped_y),
                AbortSequencePhase::RestoreSent { stopped_x, stopped_y, .. } => (stopped_x, stopped_y),
            };
            if real_x.abs() < 0.05 && real_y.abs() < 0.05 && (sx.abs() > 0.1 || sy.abs() > 0.1) {
                return;
            }
        }

        self.reported_x = real_x;
        self.reported_y = real_y;
        self.machine_x = real_x;
        self.machine_y = real_y;
        self.work_x = real_x;
        self.work_y = real_y;
    }

    /// Returns true if machine is physically moving or simulating motion
    pub fn is_moving(&self) -> bool {
        self.grbl_state == "Run"
            || self.grbl_state == "Jog"
            || self.grbl_state == "Home"
            || self.is_framing_active
            || self.is_job_running
            || self.homing_phase.is_some()
            || (self.machine_x - self.target_x).abs() > 0.2
            || (self.machine_y - self.target_y).abs() > 0.2
    }

    /// Simulates motion when offline so crosshair glides smoothly at real feed speed
    pub fn update_motion_simulation(&mut self, dt: f64) {
        if !self.is_physically_connected() {
            let dx = self.target_x - self.machine_x;
            let dy = self.target_y - self.machine_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.05 {
                let speed_mm_s = if self.is_framing_active {
                    self.settings.frame_speed / 60.0
                } else if self.is_job_running {
                    self.settings.rapid_speed / 60.0
                } else {
                    self.settings.jog_speed / 60.0
                };
                let step = speed_mm_s * dt;
                if step >= dist {
                    self.machine_x = self.target_x;
                    self.machine_y = self.target_y;
                } else {
                    self.machine_x += (dx / dist) * step;
                    self.machine_y += (dy / dist) * step;
                }
                self.work_x = self.machine_x;
                self.work_y = self.machine_y;
            } else {
                self.machine_x = self.target_x;
                self.machine_y = self.target_y;
                self.work_x = self.machine_x;
                self.work_y = self.machine_y;
            }
        }
    }

    /// Start executing a G-code job
    pub fn start_job(&mut self, gcode: &str) {
        self.job_lines = gcode
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with(';'))
            .collect();
        self.job_idx = 0;
        self.pending_ok_count = 0;
        self.job_total_lines = self.job_lines.len();
        self.is_job_running = !self.job_lines.is_empty();
        self.is_job_paused = false;
        self.job_progress = 0.0;
        self.log(&format!("[JOB] ▶ Cut job started ({} G-code lines).", self.job_total_lines));
    }

    /// Pause the active job
    pub fn pause_job(&mut self) {
        if self.is_job_running && !self.is_job_paused {
            self.is_job_paused = true;
            self.is_laser_firing = false;
            // Real-time feed hold command '!' and laser off M5
            self.send_realtime(b'!');
            self.send_raw("M5");
            self.log("[JOB] ⏸ Job paused.");
        }
    }

    /// Resume the paused job
    pub fn resume_job(&mut self) {
        if self.is_job_running && self.is_job_paused {
            self.is_job_paused = false;
            // Real-time cycle start command '~'
            self.send_realtime(b'~');
            self.log("[JOB] ▶ Job resumed.");
        }
    }

    /// Cleanly and immediately stop the active laser job without losing or resetting the coordinate origin (0,0).
    pub fn stop_job(&mut self) {
        let (stopped_x, stopped_y) = match self.abort_phase {
            Some(AbortSequencePhase::HoldIssued { stopped_x, stopped_y, .. })
            | Some(AbortSequencePhase::ResetIssued { stopped_x, stopped_y, .. })
            | Some(AbortSequencePhase::RestoreSent { stopped_x, stopped_y, .. }) => (stopped_x, stopped_y),
            None => (self.work_x, self.work_y),
        };

        // 1. Immediately halt motion and laser via GRBL real-time Feed Hold character '!' (0x21).
        // Real-time byte bypasses serial buffer and decelerates axes to a complete stop instantly.
        self.send_realtime(b'!');

        // 2. Clear job streaming queue so no further G-code lines are sent
        self.is_job_running = false;
        self.is_job_paused = false;
        self.is_laser_firing = false;
        if self.homing_phase.is_some() {
            self.send_raw("$21=1");
        }
        self.homing_phase = None;
        self.job_lines.clear();
        self.job_idx = 0;
        self.pending_ok_count = 0;
        self.job_progress = 0.0;

        // 3. Keep target and coordinates anchored at the exact stopped location
        self.target_x = stopped_x;
        self.target_y = stopped_y;
        self.machine_x = stopped_x;
        self.machine_y = stopped_y;
        self.work_x = stopped_x;
        self.work_y = stopped_y;

        if self.is_physically_connected() {
            // Begin safe buffer-flush and coordinate-restoration sequence
            self.abort_phase = Some(AbortSequencePhase::HoldIssued {
                stopped_x,
                stopped_y,
                started_at: Instant::now(),
            });
            self.log(&format!("[JOB] ⏹ Emergency stop at X={:.2}, Y={:.2} mm. Halting laser and preserving origin...", stopped_x, stopped_y));
        } else {
            self.abort_phase = None;
            self.log(&format!("[JOB] ⏹ Job stopped at X={:.2}, Y={:.2} mm. Origin (0,0) preserved.", stopped_x, stopped_y));
        }
    }

    /// Step streaming the active job
    pub fn step_job(&mut self) -> bool {
        if !self.is_job_running || self.is_job_paused || self.job_lines.is_empty() {
            return false;
        }

        self.poll_serial();

        if self.is_physically_connected() {
            while self.pending_ok_count < 3 && self.job_idx < self.job_lines.len() {
                let line = self.job_lines[self.job_idx].clone();
                self.current_command = line.clone();
                self.send_raw(&line);
                self.pending_ok_count += 1;

                // Update laser firing state
                if line.contains("M3") || line.contains("M4") {
                    self.is_laser_firing = true;
                } else if line.contains("M5") {
                    self.is_laser_firing = false;
                }

                self.job_idx += 1;
                self.job_progress = (self.job_idx as f32) / (self.job_total_lines as f32);
            }
        } else {
            // Offline / Virtual Demo simulation:
            if self.job_idx < self.job_lines.len() {
                let line = self.job_lines[self.job_idx].clone();
                self.current_command = line.clone();
                self.send_raw(&line);

                if line.contains("M3") || line.contains("M4") {
                    self.is_laser_firing = true;
                } else if line.contains("M5") {
                    self.is_laser_firing = false;
                }

                let upper = line.to_uppercase();
                if upper.starts_with("G0") || upper.starts_with("G1") || upper.starts_with("G00") || upper.starts_with("G01") {
                    for part in upper.split_whitespace() {
                        if part.starts_with('X') {
                            if let Ok(val) = part[1..].parse::<f64>() {
                                self.target_x = val;
                            }
                        } else if part.starts_with('Y') {
                            if let Ok(val) = part[1..].parse::<f64>() {
                                self.target_y = val;
                            }
                        }
                    }
                }

                self.machine_x = self.target_x;
                self.machine_y = self.target_y;
                self.work_x = self.machine_x;
                self.work_y = self.machine_y;

                self.job_idx += 1;
                self.job_progress = (self.job_idx as f32) / (self.job_total_lines as f32);
            }
        }

        if self.job_idx >= self.job_lines.len() {
            let is_done = if self.is_physically_connected() {
                self.pending_ok_count == 0 && self.grbl_state == "Idle"
            } else {
                true
            };

            if is_done {
                self.is_job_running = false;
                self.is_laser_firing = false;
                self.send_raw("M5");
                self.log("[JOB] ✅ Job completed successfully!");
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_init_and_jog() {
        let mut dm = DeviceManager::new();
        dm.settings = DeviceSettings::default();
        assert_eq!(dm.status, ConnectionStatus::Disconnected);
        assert!(!dm.is_physically_connected());
        assert_eq!(dm.machine_x, 0.0);
        assert_eq!(dm.machine_y, 0.0);

        // Connect to virtual demo for offline simulation
        let res = dm.connect("Virtuaaliportti (Demo / Offline)", 115200);
        assert!(res.is_ok());
        assert_eq!(dm.status, ConnectionStatus::Connected);

        dm.jog(15.0, 25.0, 3000.0);
        assert_eq!(dm.machine_x, 15.0);
        assert_eq!(dm.machine_y, 25.0);

        dm.jog(-5.0, -10.0, 3000.0);
        assert_eq!(dm.machine_x, 10.0);
        assert_eq!(dm.machine_y, 15.0);

        // Without limit switches enabled, home() protects the machine and does not move
        dm.home();
        assert_eq!(dm.machine_x, 10.0);

        // With limit switches enabled, home() resets to origin
        dm.settings.enable_limit_switches = true;
        dm.home();
        assert_eq!(dm.machine_x, 0.0);
        assert_eq!(dm.machine_y, 0.0);
    }

    #[test]
    fn test_weak_light_toggle() {
        let mut dm = DeviceManager::new();
        assert!(!dm.is_weak_light_on);

        dm.toggle_weak_light();
        assert!(dm.is_weak_light_on);

        dm.toggle_weak_light();
        assert!(!dm.is_weak_light_on);
    }

    #[test]
    fn test_box_frame_test() {
        let mut dm = DeviceManager::new();
        dm.start_box_frame_test(10.0, 10.0, 100.0, 80.0);
        assert!(dm.is_framing_active);
        assert!(dm.is_weak_light_on);

        while dm.is_framing_active {
            dm.step_frame_test();
        }

        assert!(!dm.is_framing_active);
        assert!(!dm.is_weak_light_on);
    }

    #[test]
    fn test_job_execution() {
        let mut dm = DeviceManager::new();
        let gcode = "G21\nG90\nG0 X15.0 Y25.0\nM4 S600\nG1 X45.0 Y65.0 F3000\nM5\n";
        dm.start_job(gcode);
        assert!(dm.is_job_running);
        assert!(!dm.is_job_paused);
        assert_eq!(dm.job_total_lines, 6);

        // Advance job
        while dm.is_job_running {
            dm.step_job();
        }

        assert!(!dm.is_job_running);
        assert_eq!(dm.machine_x, 45.0);
        assert_eq!(dm.machine_y, 65.0);
        assert!(!dm.is_laser_firing);
    }

    #[test]
    fn test_doc_to_machine_mapping() {
        let mut dm = DeviceManager::new();
        dm.settings.bed_width = 465.0;
        dm.settings.bed_height = 440.0;

        // 1. TopRight: Machine (0,0) is Doc (465, 0)
        dm.settings.origin_corner = OriginCorner::TopRight;
        let doc_pt = Point2D::new(200.0, 220.0);
        let m_pt = dm.doc_to_machine(doc_pt);
        assert_eq!(m_pt.x, 265.0);
        assert_eq!(m_pt.y, 220.0);
        assert_eq!(dm.doc_to_machine(Point2D::new(465.0, 0.0)), Point2D::new(0.0, 0.0));
        assert_eq!(dm.machine_to_doc(Point2D::new(0.0, 0.0)), Point2D::new(465.0, 0.0));

        // 2. TopLeft: Machine (0,0) is Doc (0, 0)
        dm.settings.origin_corner = OriginCorner::TopLeft;
        assert_eq!(dm.doc_to_machine(Point2D::new(0.0, 0.0)), Point2D::new(0.0, 0.0));
        assert_eq!(dm.machine_to_doc(Point2D::new(0.0, 0.0)), Point2D::new(0.0, 0.0));

        // 3. BottomLeft: Machine (0,0) is Doc (0, 440)
        dm.settings.origin_corner = OriginCorner::BottomLeft;
        assert_eq!(dm.doc_to_machine(Point2D::new(0.0, 440.0)), Point2D::new(0.0, 0.0));
        assert_eq!(dm.machine_to_doc(Point2D::new(0.0, 0.0)), Point2D::new(0.0, 440.0));

        // 4. BottomRight: Machine (0,0) is Doc (465, 440)
        dm.settings.origin_corner = OriginCorner::BottomRight;
        assert_eq!(dm.doc_to_machine(Point2D::new(465.0, 440.0)), Point2D::new(0.0, 0.0));
        assert_eq!(dm.machine_to_doc(Point2D::new(0.0, 0.0)), Point2D::new(465.0, 440.0));
    }

    #[test]
    fn test_grbl_status_parsing() {
        let mut dm = DeviceManager::new();
        dm.parse_grbl_status("<Run|MPos:15.250,30.500,0.000|FS:2000,500>");
        assert_eq!(dm.grbl_state, "Run");
        assert!((dm.machine_x - 15.250).abs() < 1e-3);
        assert!((dm.machine_y - 30.500).abs() < 1e-3);
        assert!(dm.is_laser_firing);

        dm.parse_grbl_status("<Idle|MPos:0.000,0.000,0.000|FS:0,0|WCO:0.000,0.000,0.000>");
        assert_eq!(dm.grbl_state, "Idle");
        assert_eq!(dm.machine_x, 0.0);
        assert_eq!(dm.machine_y, 0.0);
        assert!(!dm.is_laser_firing);
    }

    #[test]
    fn test_stop_job_preserves_origin_and_coordinates() {
        let mut dm = DeviceManager::new();
        dm.settings.custom_boundaries = CustomBoundaries::default();
        dm.disconnect();
        // Move to some position and start job
        dm.machine_x = 10.0;
        dm.machine_y = 10.0;
        dm.work_x = 10.0;
        dm.work_y = 10.0;

        let gcode = "G0 X30.0 Y40.0\nM4 S500\nG1 X60.0 Y80.0 F2000\nM5\n";
        dm.start_job(gcode);
        assert!(dm.is_job_running);

        // Step job until mid-execution
        dm.step_job();
        dm.step_job();
        assert_eq!(dm.machine_x, 30.0);
        assert_eq!(dm.machine_y, 40.0);

        // User presses STOP!
        dm.stop_job();

        // Job must be stopped and laser off
        assert!(!dm.is_job_running);
        assert!(!dm.is_laser_firing);

        // Crucial: coordinates MUST remain at stopped location (30.0, 40.0), NOT reset to 0,0!
        assert_eq!(dm.machine_x, 30.0);
        assert_eq!(dm.machine_y, 40.0);
        assert_eq!(dm.work_x, 30.0);
        assert_eq!(dm.work_y, 40.0);

        // Moving to origin returns to (0, 0) relative to original origin
        dm.go_to_origin();
        assert_eq!(dm.target_x, 0.0);
        assert_eq!(dm.target_y, 0.0);
    }

    #[test]
    fn test_abort_sequence_phases_and_grbl_zero_protection() {
        let mut dm = DeviceManager::new();
        dm.work_x = 75.3;
        dm.work_y = 42.1;
        dm.machine_x = 75.3;
        dm.machine_y = 42.1;

        // Simulate physical abort phase
        dm.abort_phase = Some(AbortSequencePhase::ResetIssued {
            stopped_x: 75.3,
            stopped_y: 42.1,
            started_at: Instant::now(),
        });

        // While GRBL is soft-resetting, it reports 0,0,0:
        dm.parse_grbl_status("<Idle|MPos:0.000,0.000,0.000|FS:0,0|WCO:0.000,0.000,0.000>");

        // Protection check: coordinates MUST NOT be overwritten with 0.0!
        assert_eq!(dm.work_x, 75.3);
        assert_eq!(dm.machine_x, 75.3);

        // Once G92 has restored position, GRBL reports WCO or WPos:
        dm.abort_phase = None;
        dm.parse_grbl_status("<Idle|WPos:75.300,42.100,0.000|FS:0,0>");
        assert_eq!(dm.work_x, 75.3);
        assert_eq!(dm.machine_x, 75.3);
    }

    #[test]
    fn test_mechanical_home_reset_sequence() {
        let mut dm = DeviceManager::new();
        dm.settings = DeviceSettings::default();
        dm.machine_x = 120.0;
        dm.machine_y = 80.0;
        dm.work_x = 120.0;
        dm.work_y = 80.0;

        // When switches are disabled (safe default), reset does nothing:
        assert!(!dm.settings.enable_limit_switches);
        dm.start_mechanical_home_reset();
        assert_eq!(dm.machine_x, 120.0);
        assert_eq!(dm.work_x, 120.0);

        // When enabled by user in settings:
        dm.settings.enable_limit_switches = true;
        // Offline mode: resets immediately
        dm.start_mechanical_home_reset();
        assert_eq!(dm.machine_x, 0.0);
        assert_eq!(dm.machine_y, 0.0);
        assert_eq!(dm.work_x, 0.0);
        assert_eq!(dm.work_y, 0.0);
        assert!(dm.homing_phase.is_none());

        // Test physical state machine transitions:
        dm.machine_x = 50.0;
        dm.machine_y = 50.0;
        dm.homing_phase = Some(SwitchHomingPhase::Starting {
            started_at: Instant::now() - Duration::from_millis(150),
        });
        dm.step_homing_sequence();
        assert!(matches!(dm.homing_phase, Some(SwitchHomingPhase::MovingX { .. })));

        // X switch hit (Alarm):
        dm.grbl_state = "Alarm".to_string();
        dm.step_homing_sequence();
        assert!(matches!(dm.homing_phase, Some(SwitchHomingPhase::PullingOffX { .. })));

        // After X pull-off delay, moves to Y axis:
        dm.homing_phase = Some(SwitchHomingPhase::PullingOffX {
            started_at: Instant::now() - Duration::from_millis(800),
        });
        dm.step_homing_sequence();
        assert!(matches!(dm.homing_phase, Some(SwitchHomingPhase::MovingY { .. })));

        // Y switch hit (Alarm):
        dm.grbl_state = "Alarm".to_string();
        dm.step_homing_sequence();
        assert!(matches!(dm.homing_phase, Some(SwitchHomingPhase::PullingOffY { .. })));

        // After Y pull-off delay, transitions to Finalizing:
        dm.homing_phase = Some(SwitchHomingPhase::PullingOffY {
            started_at: Instant::now() - Duration::from_millis(800),
        });
        dm.step_homing_sequence();
        assert!(matches!(dm.homing_phase, Some(SwitchHomingPhase::Finalizing { .. })));

        // After finalizing delay, origin is calibrated and saved as (0, 0):
        dm.homing_phase = Some(SwitchHomingPhase::Finalizing {
            started_at: Instant::now() - Duration::from_millis(250),
        });
        dm.step_homing_sequence();
        assert!(dm.homing_phase.is_none());
        assert_eq!(dm.machine_x, 0.0);
        assert_eq!(dm.machine_y, 0.0);
        assert_eq!(dm.work_x, 0.0);
        assert_eq!(dm.work_y, 0.0);
    }

    #[test]
    fn test_custom_learned_boundaries_and_jog_enforcement() {
        let mut dm = DeviceManager::new();
        dm.settings.custom_boundaries = CustomBoundaries::default();
        dm.target_x = 50.0;
        dm.target_y = 50.0;
        dm.work_x = 50.0;
        dm.work_y = 50.0;

        // Teach 4 corners
        // Bottom-Left at (10, 10)
        dm.target_x = 10.0;
        dm.target_y = 10.0;
        dm.save_boundary_corner("bottom_left");

        // Top-Right at (200, 180)
        dm.target_x = 200.0;
        dm.target_y = 180.0;
        dm.save_boundary_corner("top_right");

        assert!(dm.settings.custom_boundaries.enabled);
        assert_eq!(dm.settings.custom_boundaries.min_x, Some(10.0));
        assert_eq!(dm.settings.custom_boundaries.max_x, Some(200.0));
        assert_eq!(dm.settings.custom_boundaries.min_y, Some(10.0));
        assert_eq!(dm.settings.custom_boundaries.max_y, Some(180.0));

        // Test jogging within boundaries
        dm.target_x = 100.0;
        dm.target_y = 100.0;
        dm.jog(10.0, 10.0, 3000.0);
        assert_eq!(dm.target_x, 110.0);
        assert_eq!(dm.target_y, 110.0);

        // Test jog clamped at boundary
        dm.target_x = 195.0;
        dm.target_y = 175.0;
        dm.jog(15.0, 15.0, 3000.0);
        assert_eq!(dm.target_x, 200.0);
        assert_eq!(dm.target_y, 180.0);

        // Test go to taught corners
        dm.go_to_corner("bottom_left");
        assert_eq!(dm.target_x, 10.0);
        assert_eq!(dm.target_y, 10.0);

        dm.go_to_corner("top_right");
        assert_eq!(dm.target_x, 200.0);
        assert_eq!(dm.target_y, 180.0);

        // Test move exceeding max X (attempt to move to 250 -> clamped to 200)
        dm.jog(140.0, 0.0, 3000.0);
        assert_eq!(dm.target_x, 200.0);

        // Attempting to move past 200.0 is blocked completely
        dm.jog(10.0, 0.0, 3000.0);
        assert_eq!(dm.target_x, 200.0);

        // Clear boundaries
        dm.clear_boundary_corners();
        assert!(!dm.settings.custom_boundaries.enabled);
        assert!(dm.settings.custom_boundaries.min_x.is_none());
    }
}
