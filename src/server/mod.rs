//! LaserBurn Bridge & Web Remote Server
//!
//! Provides a headless HTTP/SSE/WebSocket server and transparent TCP serial bridge
//! for Raspberry Pi (Linux ARM/ARM64) and desktop remote control from iPad, mobile,
//! or web browsers without requiring local display/GUI.

use crate::device::{DeviceManager, ConnectionStatus};
use crate::formats::{GCodeConfig, GCodeGenerator};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const INDEX_HTML: &str = include_str!("web/index.html");
const MANIFEST_JSON: &str = include_str!("web/manifest.json");

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatusReport {
    pub connected: bool,
    pub status: String,
    pub grbl_state: String,
    pub port: String,
    pub machine_x: f64,
    pub machine_y: f64,
    pub work_x: f64,
    pub work_y: f64,
    pub laser_firing: bool,
    pub laser_power: u32,
    pub is_weak_light_on: bool,
    pub is_framing_active: bool,
    pub job_running: bool,
    pub job_paused: bool,
    pub job_progress: f32,
    pub job_idx: usize,
    pub job_total_lines: usize,
    pub elapsed_sec: f64,
    pub pin_state: String,
    pub bed_width: f64,
    pub bed_height: f64,
    pub recent_logs: Vec<String>,
}

pub struct BridgeServer {
    pub device: Arc<Mutex<DeviceManager>>,
    pub port: u16,
    pub tcp_bridge_port: Option<u16>,
    pub running: Arc<Mutex<bool>>,
    pub loaded_gcode: Arc<Mutex<String>>,
    pub loaded_polylines: Arc<Mutex<Vec<Vec<(f64, f64)>>>>,
    pub loaded_filename: Arc<Mutex<String>>,
}

impl BridgeServer {
    pub fn new(port: u16, tcp_bridge_port: Option<u16>) -> Self {
        let mut dm = DeviceManager::new();
        // Default to virtual mode if no port selected yet
        if dm.selected_port.is_empty() {
            dm.selected_port = "Virtual Port (Demo / Offline)".to_string();
            let _ = dm.connect("Virtual Port (Demo / Offline)", 115200);
        }

        Self {
            device: Arc::new(Mutex::new(dm)),
            port,
            tcp_bridge_port,
            running: Arc::new(Mutex::new(true)),
            loaded_gcode: Arc::new(Mutex::new(String::new())),
            loaded_polylines: Arc::new(Mutex::new(Vec::new())),
            loaded_filename: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Detect primary local LAN IP address for user convenience
    pub fn detect_local_ip() -> String {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return addr.ip().to_string();
                }
            }
        }
        "127.0.0.1".to_string()
    }

    /// Start the background motion ticker, HTTP server, and TCP serial bridge
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let local_ip = Self::detect_local_ip();
        println!("\n╔═══════════════════════════════════════════════════════════════════╗");
        println!("║   🔥  LaserBurn Bridge & Web Remote Server v1.0.0                ║");
        println!("╠═══════════════════════════════════════════════════════════════════╣");
        println!("║   🌐  Web UI (Safari / Chrome / iPad):                            ║");
        println!("║       http://localhost:{}                                       ║", self.port);
        println!("║       http://{}:{}                                ║", local_ip, self.port);
        if let Some(tb_port) = self.tcp_bridge_port {
            println!("║   🔌  Raw TCP-to-Serial Bridge:                                   ║");
            println!("║       tcp://{}:{}                                 ║", local_ip, tb_port);
        }
        println!("║   🚀  Ready for wireless workshop control & iPad PWA!             ║");
        println!("╚═══════════════════════════════════════════════════════════════════╝\n");

        // 1. Motion & Serial Ticker Thread (~50Hz)
        let device_ticker = Arc::clone(&self.device);
        let running_ticker = Arc::clone(&self.running);
        thread::spawn(move || {
            let mut last_instant = Instant::now();
            while *running_ticker.lock().unwrap() {
                let now = Instant::now();
                let dt = now.duration_since(last_instant).as_secs_f64();
                last_instant = now;

                {
                    if let Ok(mut dm) = device_ticker.lock() {
                        dm.poll_serial();
                        if !dm.is_physically_connected() {
                            dm.update_motion_simulation(dt);
                        }
                        dm.step_frame_test();
                        dm.step_job();
                    }
                }

                thread::sleep(Duration::from_millis(20));
            }
        });

        // 2. TCP Serial Bridge (Port 2323 / Telnet style)
        if let Some(tb_port) = self.tcp_bridge_port {
            let device_tcp = Arc::clone(&self.device);
            let running_tcp = Arc::clone(&self.running);
            thread::spawn(move || {
                let addr = format!("0.0.0.0:{}", tb_port);
                if let Ok(listener) = TcpListener::bind(&addr) {
                    let _ = listener.set_nonblocking(true);
                    while *running_tcp.lock().unwrap() {
                        if let Ok((stream, _)) = listener.accept() {
                            let dev_clone = Arc::clone(&device_tcp);
                            thread::spawn(move || {
                                let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
                                let mut reader = BufReader::new(stream.try_clone().unwrap());
                                let mut line = String::new();
                                while let Ok(n) = reader.read_line(&mut line) {
                                    if n == 0 { break; }
                                    let trimmed = line.trim();
                                    if !trimmed.is_empty() {
                                        if let Ok(mut dm) = dev_clone.lock() {
                                            dm.send_raw(trimmed);
                                        }
                                    }
                                    line.clear();
                                }
                            });
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            });
        }

        // 3. HTTP / SSE Web Server
        let bind_addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&bind_addr)?;
        let _ = listener.set_nonblocking(true);
        println!("[SERVER] HTTP listening on {}", bind_addr);

        while *self.running.lock().unwrap() {
            match listener.accept() {
                Ok((stream, _)) => {
                    let dev = Arc::clone(&self.device);
                    let gcode_ref = Arc::clone(&self.loaded_gcode);
                    let poly_ref = Arc::clone(&self.loaded_polylines);
                    let fn_ref = Arc::clone(&self.loaded_filename);
                    thread::spawn(move || {
                        handle_http_client(stream, dev, gcode_ref, poly_ref, fn_ref);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    eprintln!("[SERVER] Accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        println!("[SERVER] HTTP server stopped.");
        Ok(())
    }
}

fn handle_http_client(
    mut stream: TcpStream,
    device: Arc<Mutex<DeviceManager>>,
    loaded_gcode: Arc<Mutex<String>>,
    loaded_polylines: Arc<Mutex<Vec<Vec<(f64, f64)>>>>,
    loaded_filename: Arc<Mutex<String>>,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let path = parts[1];

    // Read headers
    let mut content_length: usize = 0;
    let mut content_type = String::new();
    loop {
        let mut header_line = String::new();
        if reader.read_line(&mut header_line).is_err() || header_line.trim().is_empty() {
            break;
        }
        let lower = header_line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = header_line.split(':').nth(1) {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            }
        } else if lower.starts_with("content-type:") {
            if let Some(val) = header_line.split(':').nth(1) {
                content_type = val.trim().to_string();
            }
        }
    }

    // Read body if present
    let mut body = Vec::new();
    if content_length > 0 && content_length < 50 * 1024 * 1024 { // 50MB max upload
        body.resize(content_length, 0);
        let _ = reader.read_exact(&mut body);
    }

    // Route dispatch
    match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => {
            send_response(&mut stream, 200, "text/html; charset=utf-8", INDEX_HTML.as_bytes());
        }
        ("GET", "/manifest.json") => {
            send_response(&mut stream, 200, "application/json", MANIFEST_JSON.as_bytes());
        }
        ("GET", "/api/events") => {
            handle_sse_stream(stream, device);
        }
        ("GET", "/api/status") => {
            let status = get_status_report(&device);
            let json = serde_json::to_string(&status).unwrap_or_else(|_| "{}".to_string());
            send_response(&mut stream, 200, "application/json", json.as_bytes());
        }
        ("GET", "/api/ports") => {
            let ports = DeviceManager::scan_available_ports();
            let json = serde_json::json!({ "ports": ports }).to_string();
            send_response(&mut stream, 200, "application/json", json.as_bytes());
        }
        ("POST", "/api/jog") => {
            #[derive(Deserialize)]
            struct JogReq { dx: f64, dy: f64, feed: Option<f64> }
            if let Ok(req) = serde_json::from_slice::<JogReq>(&body) {
                let feed = req.feed.unwrap_or(3000.0);
                if let Ok(mut dm) = device.lock() {
                    dm.jog(req.dx, req.dy, feed);
                }
                send_json_ok(&mut stream);
            } else {
                send_response(&mut stream, 400, "application/json", b"{\"error\":\"Invalid body\"}");
            }
        }
        ("POST", "/api/home") => {
            if let Ok(mut dm) = device.lock() {
                dm.home();
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/zero") => {
            if let Ok(mut dm) = device.lock() {
                dm.set_origin_here();
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/goto_zero") => {
            if let Ok(mut dm) = device.lock() {
                dm.go_to_origin();
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/frame") => {
            if let Ok(mut dm) = device.lock() {
                // If job loaded, frame around polylines or bed
                let polys = loaded_polylines.lock().unwrap();
                if !polys.is_empty() {
                    let mut min_x = f64::INFINITY;
                    let mut max_x = f64::NEG_INFINITY;
                    let mut min_y = f64::INFINITY;
                    let mut max_y = f64::NEG_INFINITY;
                    for poly in polys.iter() {
                        for pt in poly {
                            min_x = min_x.min(pt.0);
                            max_x = max_x.max(pt.0);
                            min_y = min_y.min(pt.1);
                            max_y = max_y.max(pt.1);
                        }
                    }
                    if min_x.is_finite() && max_x.is_finite() {
                        dm.start_box_frame_test(min_x, min_y, max_x, max_y);
                    } else {
                        dm.start_box_frame_test(0.0, 0.0, 100.0, 100.0);
                    }
                } else {
                    dm.start_box_frame_test(0.0, 0.0, 100.0, 100.0);
                }
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/weak_light") => {
            #[derive(Deserialize)]
            struct LightReq { enabled: bool }
            if let Ok(req) = serde_json::from_slice::<LightReq>(&body) {
                if let Ok(mut dm) = device.lock() {
                    dm.set_weak_light(req.enabled);
                }
                send_json_ok(&mut stream);
            } else {
                send_json_ok(&mut stream);
            }
        }
        ("POST", "/api/unlock") => {
            if let Ok(mut dm) = device.lock() {
                dm.send_raw("$X");
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/reset") => {
            if let Ok(mut dm) = device.lock() {
                dm.send_realtime(0x18); // Soft reset Ctrl-X
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/stop") => {
            if let Ok(mut dm) = device.lock() {
                dm.send_realtime(b'!'); // Feed hold
                dm.send_raw("M5");      // Laser off
                dm.send_realtime(0x18); // Soft reset
                dm.stop_job();
                dm.stop_frame_test();
                dm.set_weak_light(false);
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/connect") => {
            #[derive(Deserialize)]
            struct ConnReq { port: String, baud: Option<u32> }
            if let Ok(req) = serde_json::from_slice::<ConnReq>(&body) {
                let baud = req.baud.unwrap_or(115200);
                let res = {
                    let mut dm = device.lock().unwrap();
                    dm.connect(&req.port, baud)
                };
                match res {
                    Ok(_) => send_json_ok(&mut stream),
                    Err(e) => {
                        let json = serde_json::json!({ "success": false, "error": e }).to_string();
                        send_response(&mut stream, 400, "application/json", json.as_bytes());
                    }
                }
            } else {
                send_response(&mut stream, 400, "application/json", b"{\"error\":\"Invalid body\"}");
            }
        }
        ("POST", "/api/disconnect") => {
            if let Ok(mut dm) = device.lock() {
                dm.disconnect();
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/gcode") => {
            #[derive(Deserialize)]
            struct GCodeReq { command: String }
            if let Ok(req) = serde_json::from_slice::<GCodeReq>(&body) {
                if let Ok(mut dm) = device.lock() {
                    dm.send_raw(&req.command);
                }
                send_json_ok(&mut stream);
            } else {
                send_response(&mut stream, 400, "application/json", b"{\"error\":\"Invalid body\"}");
            }
        }
        ("POST", "/api/start") => {
            let gcode = loaded_gcode.lock().unwrap().clone();
            if !gcode.is_empty() {
                if let Ok(mut dm) = device.lock() {
                    let line_count = gcode.lines().count();
                    let est_sec = (line_count as f64) * 0.05; // Rough estimate
                    dm.start_job(&gcode, est_sec);
                }
                send_json_ok(&mut stream);
            } else {
                let json = serde_json::json!({ "success": false, "error": "No file loaded yet!" }).to_string();
                send_response(&mut stream, 400, "application/json", json.as_bytes());
            }
        }
        ("POST", "/api/pause") => {
            if let Ok(mut dm) = device.lock() {
                dm.pause_job();
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/resume") => {
            if let Ok(mut dm) = device.lock() {
                dm.resume_job();
            }
            send_json_ok(&mut stream);
        }
        ("POST", "/api/upload") => {
            handle_upload(&mut stream, &body, &content_type, loaded_gcode, loaded_polylines, loaded_filename);
        }
        _ => {
            send_response(&mut stream, 404, "text/plain", b"404 Not Found");
        }
    }
}

fn handle_upload(
    stream: &mut TcpStream,
    body: &[u8],
    content_type: &str,
    loaded_gcode: Arc<Mutex<String>>,
    loaded_polylines: Arc<Mutex<Vec<Vec<(f64, f64)>>>>,
    loaded_filename: Arc<Mutex<String>>,
) {
    let mut file_content = Vec::new();
    let mut file_name = "upload.gcode".to_string();

    // Check if multipart form
    if content_type.contains("multipart/form-data") {
        if let Some(boundary_idx) = content_type.find("boundary=") {
            let boundary = &content_type[boundary_idx + 9..];
            let boundary_bytes = format!("--{}", boundary).into_bytes();
            // Simple multipart search
            if let Some(start_pos) = find_subsequence(body, &boundary_bytes) {
                let rest = &body[start_pos + boundary_bytes.len()..];
                if let Some(header_end) = find_subsequence(rest, b"\r\n\r\n") {
                    let headers_str = String::from_utf8_lossy(&rest[..header_end]);
                    if let Some(fn_start) = headers_str.find("filename=\"") {
                        let fn_rest = &headers_str[fn_start + 10..];
                        if let Some(fn_end) = fn_rest.find('"') {
                            file_name = fn_rest[..fn_end].to_string();
                        }
                    }

                    let file_data = &rest[header_end + 4..];
                    if let Some(end_pos) = find_subsequence(file_data, &boundary_bytes) {
                        let trimmed_data = &file_data[..end_pos.saturating_sub(2)]; // trim \r\n before boundary
                        file_content = trimmed_data.to_vec();
                    }
                }
            }
        }
    }

    if file_content.is_empty() {
        file_content = body.to_vec();
    }

    let raw_text = String::from_utf8_lossy(&file_content).to_string();
    let is_lbrn2 = file_name.ends_with(".lbrn2") || raw_text.contains("<LightBurnProject");
    let is_svg = file_name.ends_with(".svg") || raw_text.contains("<svg");

    let mut generated_gcode = String::new();
    let mut extracted_polylines: Vec<Vec<(f64, f64)>> = Vec::new();

    if is_lbrn2 {
        if let Ok(doc) = crate::formats::Lbrn2File::load_from_str(&raw_text, &file_name) {
            let opt = crate::optimizer::CutOptimizer::new();
            let job = opt.optimize(&doc);
            let gen = GCodeGenerator::new(GCodeConfig::default());
            generated_gcode = gen.generate(&doc, &job);
            for shape in &doc.shapes {
                for poly in shape.to_polylines() {
                    let pts: Vec<(f64, f64)> = poly.points.iter().map(|p| (p.x, p.y)).collect();
                    if !pts.is_empty() {
                        extracted_polylines.push(pts);
                    }
                }
            }
        }
    } else if is_svg {
        if let Ok(svg_doc) = crate::formats::SvgFormat::load_from_str(&raw_text, &file_name) {
            let opt = crate::optimizer::CutOptimizer::new();
            let job = opt.optimize(&svg_doc);
            let gen = GCodeGenerator::new(GCodeConfig::default());
            generated_gcode = gen.generate(&svg_doc, &job);
            for shape in &svg_doc.shapes {
                for poly in shape.to_polylines() {
                    let pts: Vec<(f64, f64)> = poly.points.iter().map(|p| (p.x, p.y)).collect();
                    if !pts.is_empty() {
                        extracted_polylines.push(pts);
                    }
                }
            }
        }
    } else {
        // Plain G-code
        generated_gcode = raw_text;
        // Extract 2D preview paths from G0/G1
        let mut cur_poly: Vec<(f64, f64)> = Vec::new();
        let mut cur_x = 0.0;
        let mut cur_y = 0.0;
        let mut is_cutting = false;

        for line in generated_gcode.lines() {
            let upper = line.trim().to_uppercase();
            if upper.starts_with(';') || upper.starts_with('(') { continue; }
            if upper.contains("M3") || upper.contains("M4") {
                is_cutting = true;
            } else if upper.contains("M5") {
                is_cutting = false;
                if cur_poly.len() > 1 {
                    extracted_polylines.push(cur_poly.clone());
                }
                cur_poly.clear();
            }

            let mut nx = cur_x;
            let mut ny = cur_y;
            let mut moved = false;

            for token in upper.split_whitespace() {
                if token.starts_with('X') {
                    if let Ok(v) = token[1..].parse::<f64>() { nx = v; moved = true; }
                } else if token.starts_with('Y') {
                    if let Ok(v) = token[1..].parse::<f64>() { ny = v; moved = true; }
                }
            }

            if moved {
                cur_x = nx;
                cur_y = ny;
                if is_cutting {
                    if cur_poly.is_empty() {
                        cur_poly.push((cur_x, cur_y));
                    }
                    cur_poly.push((cur_x, cur_y));
                }
            }
        }
        if cur_poly.len() > 1 {
            extracted_polylines.push(cur_poly);
        }
    }

    let line_count = generated_gcode.lines().count();
    let est_sec = (line_count as f64) * 0.06;
    let mins = (est_sec / 60.0).floor() as u64;
    let secs = (est_sec % 60.0).floor() as u64;
    let time_str = format!("{:02}:{:02} min", mins, secs);

    *loaded_gcode.lock().unwrap() = generated_gcode;
    *loaded_polylines.lock().unwrap() = extracted_polylines.clone();
    *loaded_filename.lock().unwrap() = file_name.clone();

    let json = serde_json::json!({
        "success": true,
        "filename": file_name,
        "total_lines": line_count,
        "estimated_time": time_str,
        "preview_polylines": extracted_polylines,
    }).to_string();

    send_response(stream, 200, "application/json", json.as_bytes());
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn handle_sse_stream(mut stream: TcpStream, device: Arc<Mutex<DeviceManager>>) {
    let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
    if stream.write_all(header.as_bytes()).is_err() {
        return;
    }

    loop {
        let status = get_status_report(&device);
        if let Ok(json) = serde_json::to_string(&status) {
            let msg = format!("data: {}\n\n", json);
            if stream.write_all(msg.as_bytes()).is_err() {
                break; // Client disconnected
            }
        }
        thread::sleep(Duration::from_millis(100)); // 10Hz telemetry
    }
}

fn get_status_report(device: &Arc<Mutex<DeviceManager>>) -> ServerStatusReport {
    let dm = device.lock().unwrap();

    let recent: Vec<String> = dm.console_history.iter().rev().take(10).rev().cloned().collect();

    ServerStatusReport {
        connected: dm.status == ConnectionStatus::Connected || dm.status == ConnectionStatus::Busy,
        status: format!("{:?}", dm.status),
        grbl_state: dm.grbl_state.clone(),
        port: dm.selected_port.clone(),
        machine_x: dm.machine_x,
        machine_y: dm.machine_y,
        work_x: dm.work_x,
        work_y: dm.work_y,
        laser_firing: dm.is_laser_firing,
        laser_power: if dm.is_laser_firing { 100 } else if dm.is_weak_light_on { 1 } else { 0 },
        is_weak_light_on: dm.is_weak_light_on,
        is_framing_active: dm.is_framing_active,
        job_running: dm.is_job_running,
        job_paused: dm.is_job_paused,
        job_progress: dm.job_progress,
        job_idx: dm.job_idx,
        job_total_lines: dm.job_total_lines,
        elapsed_sec: dm.job_start_instant.map_or(0.0, |t| t.elapsed().as_secs_f64()),
        pin_state: dm.pin_state.clone(),
        bed_width: dm.settings.bed_width,
        bed_height: dm.settings.bed_height,
        recent_logs: recent,
    }
}

fn send_json_ok(stream: &mut TcpStream) {
    send_response(stream, 200, "application/json", b"{\"success\":true}");
}

fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let status_line = match status {
        200 => "200 OK",
        400 => "400 Bad Request",
        404 => "404 Not Found",
        _ => "500 Internal Server Error",
    };

    let headers = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        status_line, content_type, body.len()
    );

    let _ = stream.write_all(headers.as_bytes());
    let _ = stream.write_all(body);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_status_and_jog() {
        let server = BridgeServer::new(0, None); // port 0 not started, just testing logic
        let status = get_status_report(&server.device);
        assert_eq!(status.work_x, 0.0);
        assert_eq!(status.work_y, 0.0);

        // Jog
        {
            let mut dm = server.device.lock().unwrap();
            dm.jog(15.0, 25.0, 3000.0);
        }

        let updated = get_status_report(&server.device);
        assert_eq!(updated.work_x, 15.0);
        assert_eq!(updated.work_y, 25.0);
    }

    #[test]
    fn test_server_static_assets() {
        assert!(INDEX_HTML.contains("PhotonCAM Web Remote"));
        assert!(INDEX_HTML.contains("bedCanvas"));
        assert!(INDEX_HTML.contains("apiPost('jog'"));
        assert!(MANIFEST_JSON.contains("PhotonCAM Web Remote"));
    }
}

