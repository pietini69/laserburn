#[cfg(feature = "gui")]
use crate::document::{CutMode, Document, Shape};
use crate::formats::{GCodeConfig, GCodeGenerator, Lbrn2File, RuidaGenerator, SvgFormat};
use crate::geometry::{BoundingBox, Point2D, Polyline};
use crate::optimizer::CutOptimizer;
use crate::simulator::Simulator;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    DrawRect,
    DrawCircle,
    DrawLine,
    DrawText,
}

#[derive(Debug, Clone)]
pub enum FileDialogAction {
    OpenProject(std::path::PathBuf),
    SaveProject(std::path::PathBuf),
    ExportGCode(std::path::PathBuf),
    ExportRuida(std::path::PathBuf),
    BrowseFont(std::path::PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragHandle {
    None,
    Move,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

#[derive(Debug, Clone)]
pub struct OneShotSnapshot {
    pub started_at: std::time::Instant,
    pub was_streaming: bool,
    pub initial_frame_number: u64,
}

#[cfg(feature = "gui")]
pub struct LaserBurnApp {
    pub doc: Document,
    pub active_tool: ActiveTool,
    pub selected_shape_indices: Vec<usize>, // Supports multi-selection!
    pub selected_layer: usize,
    pub zoom: f32,
    pub pan: egui::Vec2,

    // Interactive Dragging & Sizing
    pub drag_handle: DragHandle,
    pub drag_start_doc: Option<Point2D>,
    pub last_drag_doc: Option<Point2D>,
    pub lock_aspect_ratio: bool,
    pub custom_rot_angle: f64,

    // Rubberband Box Selection
    pub is_box_selecting: bool,
    pub box_select_start: Option<Point2D>,
    pub box_select_current: Option<Point2D>,

    // In-progress shape creation
    pub is_drawing: bool,
    pub draw_start: Option<Point2D>,
    pub draw_current: Option<Point2D>,

    // Machine & Connection Manager
    pub device: crate::device::DeviceManager,
    pub jog_step: f64,
    pub jog_speed: f64,
    pub command_input: String,
    pub available_ports: Vec<String>,

    // Windows / Dialogs State
    pub show_connection_window: bool,
    pub show_settings_window: bool,
    pub show_move_window: bool,
    pub show_camera_window: bool,
    pub show_text_dialog: bool,
    pub text_input: String,
    pub text_font_size: f64,
    pub text_is_singleline: bool,
    pub text_font_path: String,
    pub text_font_name: String,
    pub text_pos_x: f64,
    pub text_pos_y: f64,
    pub text_align: crate::text::TextAlign,

    // Status
    pub status_msg: String,

    // Internal Shape Clipboard for Cut / Copy / Paste
    pub clipboard_shapes: Vec<Shape>,

    // Camera Feed & Manager
    pub camera_manager: crate::camera::CameraManager,
    pub camera_calibration: crate::camera::CameraCalibration,
    pub lens_undistorter: crate::camera::LensUndistorter,
    pub camera_texture: Option<egui::TextureHandle>,
    pub last_camera_frame_number: u64,
    pub last_raw_frame: Option<crate::camera::CameraFrame>,
    pub last_applied_calibration: Option<crate::camera::CameraCalibration>,
    pub camera_window_tab: usize,

    // 4-Point Camera Alignment Wizard State
    pub wizard_step: usize,
    pub wizard_target_idx: usize,
    pub wizard_points: [Option<(f64, f64)>; 4],
    pub wizard_dragged_pt: Option<usize>,
    pub wizard_magnifier_zoom: f32,

    // Keyboard Jogging Control
    pub keyboard_jog_laser: bool,

    // 1-Click Bed Overlay Snapshot Capture State
    pub oneshot_snapshot: Option<OneShotSnapshot>,

    // Non-blocking asynchronous native file dialog channel
    pub file_dialog_tx: std::sync::mpsc::Sender<FileDialogAction>,
    pub file_dialog_rx: std::sync::mpsc::Receiver<FileDialogAction>,
}

#[cfg(feature = "gui")]
impl Default for LaserBurnApp {
    fn default() -> Self {
        let (file_dialog_tx, file_dialog_rx) = std::sync::mpsc::channel();
        let mut app = Self {
            doc: Document::new("LaserBurn_Project", 465.0, 440.0),
            active_tool: ActiveTool::Select,
            selected_shape_indices: vec![2], // Gear selected by default
            selected_layer: 0,
            zoom: 1.6,
            pan: egui::Vec2::new(120.0, 80.0),

            drag_handle: DragHandle::None,
            drag_start_doc: None,
            last_drag_doc: None,
            lock_aspect_ratio: true,
            custom_rot_angle: 1.0,

            is_box_selecting: false,
            box_select_start: None,
            box_select_current: None,

            is_drawing: false,
            draw_start: None,
            draw_current: None,

            device: crate::device::DeviceManager::new(),
            jog_step: 10.0,
            jog_speed: 3000.0,
            command_input: "".to_string(),
            available_ports: Vec::new(),

            show_connection_window: false,
            show_settings_window: false,
            show_move_window: false,
            show_camera_window: false,
            show_text_dialog: false,
            text_input: "Text".to_string(),
            text_font_size: 15.0,
            text_is_singleline: false,
            text_font_path: crate::text::default_system_font(),
            text_font_name: "Arial / System".to_string(),
            text_pos_x: 60.0,
            text_pos_y: 60.0,
            text_align: crate::text::TextAlign::Left,

            status_msg: "LaserBurn Ready - Click inside shapes to select, drag to move, or use Arrow Keys to Jog".to_string(),
            clipboard_shapes: Vec::new(),
            camera_manager: crate::camera::CameraManager::default(),
            camera_calibration: crate::camera::CameraCalibration::load(),
            lens_undistorter: crate::camera::LensUndistorter::default(),
            camera_texture: None,
            last_camera_frame_number: 0,
            last_raw_frame: Self::load_bed_snapshot_from_disk(),
            last_applied_calibration: None,
            camera_window_tab: 0,
            wizard_step: 0,
            wizard_target_idx: 0,
            wizard_points: [None; 4],
            wizard_dragged_pt: None,
            wizard_magnifier_zoom: 4.0,
            keyboard_jog_laser: true,
            oneshot_snapshot: None,
            file_dialog_tx,
            file_dialog_rx,
        };

        app.doc.width = app.device.settings.bed_width;
        app.doc.height = app.device.settings.bed_height;

        if app.camera_calibration.homography_points.len() == 4 {
            for i in 0..4 {
                let pt = &app.camera_calibration.homography_points[i];
                app.wizard_points[i] = Some((pt.camera_u, pt.camera_v));
            }
        }

        app.load_sample_gear(200.0, 220.0, 12, 45.0, 55.0);

        // Add sample Outline TrueType text
        app.text_input = "LaserBurn 2026".to_string();
        app.text_pos_x = 70.0;
        app.text_pos_y = 50.0;
        app.text_font_size = 18.0;
        app.text_is_singleline = false;
        app.selected_layer = 0;
        app.add_text_to_document();

        // Add sample Single-Line Hershey text (perfect for fast laser scoring)
        app.text_input = "SINGLE-LINE STROKE FONT".to_string();
        app.text_pos_x = 70.0;
        app.text_pos_y = 80.0;
        app.text_font_size = 9.0;
        app.text_is_singleline = true;
        app.selected_layer = 1;
        app.add_text_to_document();

        // Reset text input defaults
        app.text_input = "Sample Text".to_string();
        app.text_pos_x = 100.0;
        app.text_pos_y = 120.0;
        app.text_font_size = 15.0;
        app.text_is_singleline = false;
        app.selected_layer = 0;
        app.show_text_dialog = false;

        // Laser starts in safe OFF state with head at origin
        app.device.is_weak_light_on = false;
        app.device.machine_x = 0.0;
        app.device.machine_y = 0.0;

        app
    }
}

#[cfg(feature = "gui")]
impl LaserBurnApp {
    /// Returns "Cmd" on macOS and "Ctrl" on Windows / Linux for native shortcut displays
    pub fn cmd_key() -> &'static str {
        if cfg!(target_os = "macos") {
            "Cmd"
        } else {
            "Ctrl"
        }
    }

    pub fn load_sample_gear(&mut self, cx: f64, cy: f64, teeth: usize, r_inner: f64, r_outer: f64) {
        // Center bore
        self.doc.add_shape(Shape::Circle {
            layer_index: 0,
            center: Point2D::new(cx, cy),
            radius: 12.0,
        });

        // 4 mounting holes
        for i in 0..4 {
            let a = (i as f64) * std::f64::consts::FRAC_PI_2;
            self.doc.add_shape(Shape::Circle {
                layer_index: 0,
                center: Point2D::new(cx + 25.0 * a.cos(), cy + 25.0 * a.sin()),
                radius: 4.0,
            });
        }

        // Outer gear perimeter
        let mut gear_pts = Vec::new();
        for i in 0..teeth {
            let a0 = (i as f64 / teeth as f64) * std::f64::consts::TAU;
            let a1 = a0 + (0.28 / teeth as f64) * std::f64::consts::TAU;
            let a2 = a0 + (0.50 / teeth as f64) * std::f64::consts::TAU;
            let a3 = a0 + (0.78 / teeth as f64) * std::f64::consts::TAU;

            gear_pts.push(Point2D::new(cx + r_inner * a0.cos(), cy + r_inner * a0.sin()));
            gear_pts.push(Point2D::new(cx + r_outer * a1.cos(), cy + r_outer * a1.sin()));
            gear_pts.push(Point2D::new(cx + r_outer * a2.cos(), cy + r_outer * a2.sin()));
            gear_pts.push(Point2D::new(cx + r_inner * a3.cos(), cy + r_inner * a3.sin()));
        }

        self.doc.add_shape(Shape::Path {
            layer_index: 0,
            polyline: Polyline::new(gear_pts, true),
        });
    }

    /// Loads a project file (.lbrn2, .lbrn, .svg) into the editor
    pub fn open_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        let p = path.as_ref();
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        let mut doc = match ext.as_str() {
            "lbrn2" | "lbrn" => Lbrn2File::load_from_file(p)?,
            "svg" => SvgFormat::load_from_file(p)?,
            _ => return Err(format!("Unsupported file format: .{}", ext)),
        };
        // Keep bed dimensions strictly locked to physical machine bed size!
        doc.width = self.device.settings.bed_width;
        doc.height = self.device.settings.bed_height;
        self.doc = doc;
        self.selected_shape_indices = if !self.doc.shapes.is_empty() { vec![0] } else { vec![] };
        if let Some(first_shape) = self.doc.shapes.first() {
            self.selected_layer = first_shape.layer_index();
        } else if let Some(&first_layer) = self.doc.layers.keys().next() {
            self.selected_layer = first_layer;
        } else {
            self.selected_layer = 0;
        }
        self.zoom = 1.2;
        self.pan = egui::Vec2::new(80.0, 60.0);
        self.status_msg = format!(
            "Loaded: {} ({} shapes, {} layers)",
            p.file_name().unwrap_or_default().to_string_lossy(),
            self.doc.shapes.len(),
            self.doc.layers.len()
        );
        Ok(())
    }

    /// Asynchronously prompts the user to open a project file without blocking the UI thread or freezing winit
    pub fn prompt_open_project(&self, ctx: &egui::Context) {
        let tx = self.file_dialog_tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            pollster::block_on(async move {
                if let Some(handle) = rfd::AsyncFileDialog::new()
                    .add_filter("Laser Projects (*.lbrn2, *.lbrn, *.svg)", &["lbrn2", "lbrn", "svg"])
                    .pick_file()
                    .await
                {
                    let _ = tx.send(FileDialogAction::OpenProject(handle.path().to_path_buf()));
                    ctx.request_repaint();
                }
            });
        });
    }

    /// Asynchronously prompts the user to save the current project without blocking the UI thread or freezing winit
    pub fn prompt_save_project(&self, ctx: &egui::Context) {
        let tx = self.file_dialog_tx.clone();
        let ctx = ctx.clone();
        let default_name = if self.doc.name.is_empty() || self.doc.name == "Untitled" || self.doc.name == "LaserBurn_Project" {
            "project.lbrn2".to_string()
        } else if self.doc.name.ends_with(".lbrn2") {
            self.doc.name.clone()
        } else {
            format!("{}.lbrn2", self.doc.name)
        };
        std::thread::spawn(move || {
            pollster::block_on(async move {
                if let Some(handle) = rfd::AsyncFileDialog::new()
                    .add_filter("LightBurn Project (*.lbrn2)", &["lbrn2"])
                    .set_file_name(&default_name)
                    .save_file()
                    .await
                {
                    let _ = tx.send(FileDialogAction::SaveProject(handle.path().to_path_buf()));
                    ctx.request_repaint();
                }
            });
        });
    }

    /// Asynchronously prompts the user to export optimized G-Code
    pub fn prompt_export_gcode(&self, ctx: &egui::Context) {
        let tx = self.file_dialog_tx.clone();
        let ctx = ctx.clone();
        let default_name = format!("{}.gcode", self.doc.name.trim_end_matches(".lbrn2"));
        std::thread::spawn(move || {
            pollster::block_on(async move {
                if let Some(handle) = rfd::AsyncFileDialog::new()
                    .add_filter("G-Code (*.gcode, *.nc)", &["gcode", "nc"])
                    .set_file_name(&default_name)
                    .save_file()
                    .await
                {
                    let _ = tx.send(FileDialogAction::ExportGCode(handle.path().to_path_buf()));
                    ctx.request_repaint();
                }
            });
        });
    }

    /// Asynchronously prompts the user to export Ruida binary .rd
    pub fn prompt_export_ruida(&self, ctx: &egui::Context) {
        let tx = self.file_dialog_tx.clone();
        let ctx = ctx.clone();
        let default_name = format!("{}.rd", self.doc.name.trim_end_matches(".lbrn2"));
        std::thread::spawn(move || {
            pollster::block_on(async move {
                if let Some(handle) = rfd::AsyncFileDialog::new()
                    .add_filter("Ruida Binary (*.rd)", &["rd"])
                    .set_file_name(&default_name)
                    .save_file()
                    .await
                {
                    let _ = tx.send(FileDialogAction::ExportRuida(handle.path().to_path_buf()));
                    ctx.request_repaint();
                }
            });
        });
    }

    /// Asynchronously prompts the user to browse for a font file
    pub fn prompt_browse_font(&self, ctx: &egui::Context) {
        let tx = self.file_dialog_tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            pollster::block_on(async move {
                if let Some(handle) = rfd::AsyncFileDialog::new()
                    .add_filter("Fonts (*.ttf, *.otf, *.ttc)", &["ttf", "otf", "ttc"])
                    .pick_file()
                    .await
                {
                    let _ = tx.send(FileDialogAction::BrowseFont(handle.path().to_path_buf()));
                    ctx.request_repaint();
                }
            });
        });
    }

    /// Combined bounding box of all currently selected shapes
    pub fn selection_bounds(&self) -> Option<BoundingBox> {
        let mut bb = BoundingBox::empty();
        let mut found = false;
        for &idx in &self.selected_shape_indices {
            if idx < self.doc.shapes.len() {
                bb = bb.union(&self.doc.shapes[idx].bounds());
                found = true;
            }
        }
        if found {
            Some(bb)
        } else {
            None
        }
    }

    /// Smart hit testing: finds best shape matching a click point.
    /// Tests if click is inside the shape or near its outline.
    /// Prioritizes smallest enclosing shape so clicking a hole inside a gear selects the hole!
    pub fn find_best_shape_at_point(&self, pt: &Point2D, tolerance: f64) -> Option<usize> {
        let mut candidates: Vec<(usize, f64)> = Vec::new(); // (index, area)

        // Pass 1: exact hit test (inside shape or within tolerance of outline)
        for (idx, shape) in self.doc.shapes.iter().enumerate() {
            if shape.hit_test(pt, tolerance) {
                let b = shape.bounds();
                let area = (b.width() * b.height()).max(1.0);
                candidates.push((idx, area));
            }
        }

        // Pass 2: slightly broader tolerance (3x) for easy clicking near thin lines / borders
        if candidates.is_empty() {
            for (idx, shape) in self.doc.shapes.iter().enumerate() {
                if shape.hit_test(pt, tolerance * 3.0) {
                    let b = shape.bounds();
                    let area = (b.width() * b.height()).max(1.0);
                    candidates.push((idx, area));
                }
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // Sort by area ascending: smallest shape first (selects hole before outer gear)
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        Some(candidates[0].0)
    }

    /// Checks if a document point hits any of the 8 resize handles of the selection
    fn test_handles(&self, doc_pt: &Point2D, b: &BoundingBox, handle_radius: f64) -> DragHandle {
        let cx = (b.min_x + b.max_x) * 0.5;
        let cy = (b.min_y + b.max_y) * 0.5;

        let handles = [
            (DragHandle::TopLeft, Point2D::new(b.min_x, b.min_y)),
            (DragHandle::Top, Point2D::new(cx, b.min_y)),
            (DragHandle::TopRight, Point2D::new(b.max_x, b.min_y)),
            (DragHandle::Right, Point2D::new(b.max_x, cy)),
            (DragHandle::BottomRight, Point2D::new(b.max_x, b.max_y)),
            (DragHandle::Bottom, Point2D::new(cx, b.max_y)),
            (DragHandle::BottomLeft, Point2D::new(b.min_x, b.max_y)),
            (DragHandle::Left, Point2D::new(b.min_x, cy)),
        ];

        for (handle, pt) in handles {
            if doc_pt.distance_to(&pt) <= handle_radius {
                return handle;
            }
        }
        DragHandle::None
    }

    /// Groups currently selected shapes into a single Shape::Group
    pub fn group_selected(&mut self) {
        if self.selected_shape_indices.len() < 2 {
            self.status_msg = "Select at least 2 shapes to group".to_string();
            return;
        }

        // Sort indices descending so removal doesn't shift earlier indices
        let mut sorted = self.selected_shape_indices.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        sorted.dedup();

        let mut group_shapes = Vec::new();
        let layer_idx = self.doc.shapes[sorted[0]].layer_index();

        for idx in sorted {
            if idx < self.doc.shapes.len() {
                group_shapes.push(self.doc.shapes.remove(idx));
            }
        }
        group_shapes.reverse();

        self.doc.shapes.push(Shape::Group {
            layer_index: layer_idx,
            shapes: group_shapes,
            locked: false,
        });

        self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
        self.status_msg = "Shapes locked together in a Group".to_string();
    }

    /// Ungroups selected Shape::Group back into individual shapes
    pub fn ungroup_selected(&mut self) {
        let mut new_selection = Vec::new();
        let mut sorted = self.selected_shape_indices.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        sorted.dedup();

        for idx in sorted {
            if idx < self.doc.shapes.len() {
                if let Shape::Group { shapes, .. } = self.doc.shapes.remove(idx) {
                    for s in shapes {
                        self.doc.shapes.push(s);
                        new_selection.push(self.doc.shapes.len() - 1);
                    }
                }
            }
        }

        if !new_selection.is_empty() {
            self.selected_shape_indices = new_selection;
            self.status_msg = "Group(s) unpacked into individual shapes".to_string();
        } else {
            self.status_msg = "No groups found in current selection to ungroup".to_string();
        }
    }

    /// Locks or unlocks selected shape(s) in place
    pub fn toggle_lock_selected(&mut self) {
        if self.selected_shape_indices.is_empty() {
            return;
        }

        let any_locked = self.selected_shape_indices.iter().any(|&i| {
            i < self.doc.shapes.len() && self.doc.shapes[i].is_locked()
        });

        if any_locked {
            // Unlock selected shapes
            for &idx in &self.selected_shape_indices {
                if idx < self.doc.shapes.len() {
                    if let Shape::Group { shapes, locked, .. } = &mut self.doc.shapes[idx] {
                        *locked = false;
                        if shapes.len() == 1 {
                            let inner = shapes.remove(0);
                            self.doc.shapes[idx] = inner;
                        }
                    }
                }
            }
            self.status_msg = "Selection unlocked (can be moved and edited)".to_string();
        } else {
            // Lock selected shapes
            for &idx in &self.selected_shape_indices {
                if idx < self.doc.shapes.len() {
                    if let Shape::Group { ref mut locked, .. } = self.doc.shapes[idx] {
                        *locked = true;
                    } else {
                        let orig = self.doc.shapes[idx].clone();
                        let layer = orig.layer_index();
                        self.doc.shapes[idx] = Shape::Group {
                            layer_index: layer,
                            shapes: vec![orig],
                            locked: true,
                        };
                    }
                }
            }
            self.status_msg = "Selection locked in place (cannot be moved accidentally)".to_string();
        }
    }

    /// Welds / joins selected open polylines that touch end-to-end
    pub fn weld_selected(&mut self) {
        if self.selected_shape_indices.len() < 2 {
            self.status_msg = "Select multiple paths to join".to_string();
            return;
        }

        let mut polylines = Vec::new();
        let mut sorted = self.selected_shape_indices.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));

        let layer = self.doc.shapes[sorted[0]].layer_index();

        for idx in sorted {
            if idx < self.doc.shapes.len() {
                let s = self.doc.shapes.remove(idx);
                for p in s.to_polylines() {
                    polylines.push(p);
                }
            }
        }

        if polylines.is_empty() {
            return;
        }

        // Connect polylines end-to-end
        let mut current = polylines.remove(0);
        let tolerance = 1.0; // 1mm connection tolerance

        while !polylines.is_empty() {
            let last_pt = *current.points.last().unwrap();
            let mut found_next = None;

            for (i, p) in polylines.iter().enumerate() {
                if p.points.first().unwrap().distance_to(&last_pt) <= tolerance {
                    found_next = Some((i, false));
                    break;
                } else if p.points.last().unwrap().distance_to(&last_pt) <= tolerance {
                    found_next = Some((i, true));
                    break;
                }
            }

            if let Some((idx, should_reverse)) = found_next {
                let mut next_poly = polylines.remove(idx);
                if should_reverse {
                    next_poly.reverse();
                }
                current.points.extend_from_slice(&next_poly.points[1..]);
            } else {
                // Cannot connect further, push whatever was built
                break;
            }
        }

        // Close if ends meet
        if current.points.len() > 2 && current.points.first().unwrap().distance_to(current.points.last().unwrap()) <= tolerance {
            current.is_closed = true;
        }

        self.doc.shapes.push(Shape::Path {
            layer_index: layer,
            polyline: current,
        });

        // Add any remaining unjoined
        for p in polylines {
            self.doc.shapes.push(Shape::Path {
                layer_index: layer,
                polyline: p,
            });
        }

        self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
        self.status_msg = "Joined paths successfully".to_string();
    }

    /// Rotates currently selected shapes around their collective center by angle in degrees
    pub fn rotate_selected(&mut self, angle_deg: f64) {
        if self.selected_shape_indices.is_empty() {
            return;
        }
        if let Some(bounds) = self.selection_bounds() {
            let c = bounds.center();
            let rad = angle_deg.to_radians();
            for &idx in &self.selected_shape_indices {
                if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                    self.doc.shapes[idx].rotate(c, rad);
                }
            }
            self.status_msg = format!("Rotated selection by {:+.1}°", angle_deg);
        }
    }

    /// Flips currently selected shapes horizontally around their collective center
    pub fn flip_selected_h(&mut self) {
        if self.selected_shape_indices.is_empty() {
            return;
        }
        if let Some(bounds) = self.selection_bounds() {
            let c = bounds.center();
            for &idx in &self.selected_shape_indices {
                if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                    self.doc.shapes[idx].scale(c, -1.0, 1.0);
                }
            }
            self.status_msg = "Flipped selection horizontally".to_string();
        }
    }

    /// Flips currently selected shapes vertically around their collective center
    pub fn flip_selected_v(&mut self) {
        if self.selected_shape_indices.is_empty() {
            return;
        }
        if let Some(bounds) = self.selection_bounds() {
            let c = bounds.center();
            for &idx in &self.selected_shape_indices {
                if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                    self.doc.shapes[idx].scale(c, 1.0, -1.0);
                }
            }
            self.status_msg = "Flipped selection vertically".to_string();
        }
    }

    /// Copies selected shapes to the internal clipboard
    pub fn copy_selected(&mut self) {
        if self.selected_shape_indices.is_empty() {
            self.status_msg = "Nothing selected to copy".to_string();
            return;
        }
        self.clipboard_shapes.clear();
        for &idx in &self.selected_shape_indices {
            if idx < self.doc.shapes.len() {
                let mut s = self.doc.shapes[idx].clone();
                s.set_locked(false); // Pasted copies should start unlocked
                self.clipboard_shapes.push(s);
            }
        }
        let count = self.clipboard_shapes.len();
        self.status_msg = format!("Copied {} shape(s) to clipboard", count);
    }

    /// Cuts selected shapes to the clipboard and removes them from the document
    pub fn cut_selected(&mut self) {
        if self.selected_shape_indices.is_empty() {
            self.status_msg = "Nothing selected to cut".to_string();
            return;
        }
        self.copy_selected();
        self.delete_selected();
        let count = self.clipboard_shapes.len();
        self.status_msg = format!("Cut {} shape(s) to clipboard", count);
    }

    /// Pastes shapes from the clipboard into the document with a slight offset (+5mm)
    pub fn paste(&mut self) {
        if self.clipboard_shapes.is_empty() {
            self.status_msg = "Clipboard is empty".to_string();
            return;
        }
        const PASTE_OFFSET: f64 = 5.0;
        let mut new_selection = Vec::new();
        for s in &mut self.clipboard_shapes {
            s.translate(PASTE_OFFSET, PASTE_OFFSET);
            self.doc.add_shape(s.clone());
            new_selection.push(self.doc.shapes.len() - 1);
        }
        let count = new_selection.len();
        self.selected_shape_indices = new_selection;
        self.status_msg = format!("Pasted {} shape(s)", count);
    }

    /// Duplicates selected shapes with a slight offset (+5mm) and selects the duplicates
    pub fn duplicate_selected(&mut self) {
        if self.selected_shape_indices.is_empty() {
            self.status_msg = "Nothing selected to duplicate".to_string();
            return;
        }
        const DUP_OFFSET: f64 = 5.0;
        let mut new_selection = Vec::new();
        for &idx in &self.selected_shape_indices {
            if idx < self.doc.shapes.len() {
                let mut dup = self.doc.shapes[idx].clone();
                dup.set_locked(false);
                dup.translate(DUP_OFFSET, DUP_OFFSET);
                self.doc.add_shape(dup);
                new_selection.push(self.doc.shapes.len() - 1);
            }
        }
        let count = new_selection.len();
        self.selected_shape_indices = new_selection;
        self.status_msg = format!("Duplicated {} shape(s)", count);
    }

    /// Deletes currently selected shapes from the document (protects locked shapes)
    pub fn delete_selected(&mut self) {
        if self.selected_shape_indices.is_empty() {
            return;
        }
        let mut sorted = self.selected_shape_indices.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        sorted.dedup();

        let mut deleted_count = 0;
        let mut locked_count = 0;
        for idx in sorted {
            if idx < self.doc.shapes.len() {
                if self.doc.shapes[idx].is_locked() {
                    locked_count += 1;
                } else {
                    self.doc.shapes.remove(idx);
                    deleted_count += 1;
                }
            }
        }
        if locked_count > 0 && deleted_count == 0 {
            self.status_msg = "Cannot delete locked shape(s) [LOCKED] (Unlock with Cmd+L first)".to_string();
        } else {
            self.selected_shape_indices.clear();
            if locked_count > 0 {
                self.status_msg = format!("Deleted {} shape(s); {} locked shape(s) preserved", deleted_count, locked_count);
            } else {
                self.status_msg = format!("Deleted {} shape(s)", deleted_count);
            }
        }
    }

    /// Converts user text into vector geometry and adds it to the document
    pub fn add_text_to_document(&mut self) {
        if self.text_input.trim().is_empty() {
            return;
        }

        let font_type = if self.text_is_singleline {
            crate::text::FontType::SingleLine
        } else {
            let path = if self.text_font_path.is_empty() {
                crate::text::default_system_font()
            } else {
                self.text_font_path.clone()
            };
            crate::text::FontType::TrueType { path }
        };

        let opt = crate::text::TextOptions {
            text: self.text_input.clone(),
            font_type,
            font_size_mm: self.text_font_size,
            x: self.text_pos_x,
            y: self.text_pos_y,
            align: self.text_align,
            letter_spacing: 1.0,
            line_spacing: 1.2,
        };

        match crate::text::generate_text_polylines(&opt) {
            Ok(polylines) => {
                if polylines.is_empty() {
                    self.status_msg = "No geometry generated for text".to_string();
                    return;
                }

                let layer_idx = self.selected_layer;
                let char_shapes: Vec<Shape> = polylines
                    .into_iter()
                    .map(|poly| Shape::Path {
                        layer_index: layer_idx,
                        polyline: poly,
                    })
                    .collect();

                let count = char_shapes.len();
                let text_group = Shape::Group {
                    layer_index: layer_idx,
                    shapes: char_shapes,
                    locked: false,
                };

                self.doc.add_shape(text_group);
                let new_idx = self.doc.shapes.len() - 1;
                self.selected_shape_indices = vec![new_idx];
                self.status_msg = format!("Added text: \"{}\" ({} paths, group #{})", self.text_input, count, new_idx + 1);
            }
            Err(e) => {
                self.status_msg = format!("Error generating text: {}", e);
            }
        }
    }

    /// Starts a bounding box framing test with weak light
    pub fn start_box_frame(&mut self) {
        let b = if let Some(bounds) = self.selection_bounds() {
            bounds
        } else {
            self.doc.bounds()
        };

        if b.width() > 0.0 && b.height() > 0.0 {
            self.device.start_box_frame_test(b.min_x, b.min_y, b.max_x, b.max_y);
            self.status_msg = format!("Running box framing test with weak beam: {:.1}x{:.1}mm", b.width(), b.height());
        } else {
            self.status_msg = "No shapes available for framing".to_string();
        }
    }

    /// Starts a contour trace framing test with weak light along exact vector paths
    pub fn start_contour_frame(&mut self) {
        let mut points = Vec::new();

        if !self.selected_shape_indices.is_empty() {
            for &idx in &self.selected_shape_indices {
                if idx < self.doc.shapes.len() {
                    for poly in self.doc.shapes[idx].to_polylines() {
                        points.extend_from_slice(&poly.points);
                        if poly.is_closed && !poly.points.is_empty() {
                            points.push(poly.points[0]);
                        }
                    }
                }
            }
        } else {
            for shape in &self.doc.shapes {
                for poly in shape.to_polylines() {
                    points.extend_from_slice(&poly.points);
                    if poly.is_closed && !poly.points.is_empty() {
                        points.push(poly.points[0]);
                    }
                }
            }
        }

        if !points.is_empty() {
            self.device.start_contour_frame_test(points);
            self.status_msg = "Running contour framing test with weak beam...".to_string();
        } else {
            self.status_msg = "No cut paths available to test".to_string();
        }
    }

    /// Convert document coordinate (Point2D in mm) to screen position (egui::Pos2),
    /// strictly preserving natural left-to-right and top-to-bottom orientation (never mirrored).
    pub fn doc_to_screen(&self, pt: Point2D, canvas_origin: egui::Pos2) -> egui::Pos2 {
        let bed_w = self.doc.width as f32 * self.zoom;
        let bed_h = self.doc.height as f32 * self.zoom;
        let bed_rect = egui::Rect::from_min_size(canvas_origin, egui::Vec2::new(bed_w, bed_h));

        egui::pos2(
            bed_rect.left() + pt.x as f32 * self.zoom,
            bed_rect.top() + pt.y as f32 * self.zoom,
        )
    }

    /// Convert screen position (egui::Pos2) to document coordinates (Point2D in mm),
    /// reversing zoom and pan without mirroring.
    pub fn screen_to_doc(&self, screen_pos: egui::Pos2, canvas_origin: egui::Pos2) -> Point2D {
        let bed_w = self.doc.width as f32 * self.zoom;
        let bed_h = self.doc.height as f32 * self.zoom;
        let bed_rect = egui::Rect::from_min_size(canvas_origin, egui::Vec2::new(bed_w, bed_h));

        Point2D::new(
            ((screen_pos.x - bed_rect.left()) / self.zoom) as f64,
            ((screen_pos.y - bed_rect.top()) / self.zoom) as f64,
        )
    }

    /// Calculate adaptive grid line spacing and measurement label spacing based on zoom level:
    /// Returns `(minor_step_mm, major_step_mm, label_step_mm)`.
    pub fn grid_steps_for_zoom(zoom: f32) -> (f64, f64, f64) {
        if zoom >= 10.0 {
            (0.5, 2.0, 2.0)
        } else if zoom >= 5.5 {
            (1.0, 5.0, 5.0)
        } else if zoom >= 2.8 {
            (2.0, 10.0, 10.0)
        } else if zoom >= 1.3 {
            (5.0, 20.0, 20.0)
        } else if zoom >= 0.6 {
            (10.0, 50.0, 50.0)
        } else {
            (50.0, 100.0, 100.0)
        }
    }

    /// Paint live camera background overlay onto the laser bed canvas with perspective, scaling, and rotation
    pub fn paint_camera_on_bed(
        &self,
        painter: &egui::Painter,
        canvas_origin: egui::Pos2,
        texture: &egui::TextureHandle,
    ) {
        let cal = &self.camera_calibration;
        let doc_w = self.doc.width;
        let doc_h = self.doc.height;
        let alpha = (cal.opacity.clamp(0.05, 1.0) * 255.0) as u8;
        let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);

        // High-precision 4-Point Homography perspective mesh
        if cal.use_homography {
            if let Some(ref inv_h) = cal.inv_homography_matrix {
                let nx = 20;
                let ny = 20;
                let mut mesh = egui::Mesh::with_texture(texture.id());

                for k in 0..=ny {
                    let ty = k as f64 / ny as f64;
                    let doc_y = doc_h * ty;
                    for j in 0..=nx {
                        let tx = j as f64 / nx as f64;
                        let doc_x = doc_w * tx;

                        let screen_pos = self.doc_to_screen(Point2D::new(doc_x, doc_y), canvas_origin);
                        let (mut u, mut v) = crate::camera::project_homography(inv_h, doc_x, doc_y);

                        if cal.flip_h {
                            u = 1.0 - u;
                        }
                        if cal.flip_v {
                            v = 1.0 - v;
                        }

                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: screen_pos,
                            uv: egui::pos2(u as f32, v as f32),
                            color,
                        });
                    }
                }

                for k in 0..ny {
                    for j in 0..nx {
                        let top_left = (k * (nx + 1) + j) as u32;
                        let top_right = top_left + 1;
                        let bottom_left = ((k + 1) * (nx + 1) + j) as u32;
                        let bottom_right = bottom_left + 1;

                        mesh.indices.extend_from_slice(&[
                            top_left, top_right, bottom_right,
                            top_left, bottom_right, bottom_left,
                        ]);
                    }
                }

                painter.add(egui::Shape::mesh(mesh));
                return;
            }
        }

        // Fallback: 4-corner affine transform
        let cx = doc_w * 0.5 + cal.offset_x_mm;
        let cy = doc_h * 0.5 + cal.offset_y_mm;

        let eff_scale_x = (cal.uniform_scale * cal.scale_x).max(0.01);
        let eff_scale_y = (cal.uniform_scale * cal.scale_y).max(0.01);

        let half_w = (doc_w * 0.5 * eff_scale_x).max(1.0);
        let half_h = (doc_h * 0.5 * eff_scale_y).max(1.0);

        let rot_rad = (cal.rotation_deg as f64).to_radians();
        let cos_a = rot_rad.cos();
        let sin_a = rot_rad.sin();

        // 4 local corners relative to center: TL, TR, BR, BL
        let corners = [
            (-half_w, -half_h),
            (half_w, -half_h),
            (half_w, half_h),
            (-half_w, half_h),
        ];

        let mut screen_corners = [egui::Pos2::ZERO; 4];
        for (i, &(lx, ly)) in corners.iter().enumerate() {
            let doc_x = cx + lx * cos_a - ly * sin_a;
            let doc_y = cy + lx * sin_a + ly * cos_a;
            screen_corners[i] = self.doc_to_screen(Point2D::new(doc_x, doc_y), canvas_origin);
        }

        let (u_min, u_max) = if cal.flip_h { (1.0, 0.0) } else { (0.0, 1.0) };
        let (v_min, v_max) = if cal.flip_v { (1.0, 0.0) } else { (0.0, 1.0) };

        let uvs = [
            egui::pos2(u_min, v_min),
            egui::pos2(u_max, v_min),
            egui::pos2(u_max, v_max),
            egui::pos2(u_min, v_max),
        ];

        let mut mesh = egui::Mesh::with_texture(texture.id());
        for i in 0..4 {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: screen_corners[i],
                uv: uvs[i],
                color,
            });
        }
        mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

        painter.add(egui::Shape::mesh(mesh));
    }

    pub fn bed_snapshot_path() -> Option<std::path::PathBuf> {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok()?;
        let dir = std::path::PathBuf::from(home).join(".config").join("laser_burn");
        let _ = std::fs::create_dir_all(&dir);
        Some(dir.join("last_bed_snapshot.png"))
    }

    pub fn save_bed_snapshot_to_disk(&self, frame: &crate::camera::CameraFrame) {
        if let Some(path) = Self::bed_snapshot_path() {
            if let Some(img) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.rgba.clone()) {
                let _ = img.save(&path);
            }
        }
    }

    pub fn load_bed_snapshot_from_disk() -> Option<crate::camera::CameraFrame> {
        let path = Self::bed_snapshot_path()?;
        if !path.exists() {
            return None;
        }
        let img = image::open(&path).ok()?.to_rgba8();
        let (width, height) = img.dimensions();
        Some(crate::camera::CameraFrame {
            width,
            height,
            rgba: img.into_raw(),
            frame_number: 1,
        })
    }

    /// Trigger 1-click bed overlay update from camera (starts stream if needed, captures snapshot, saves to disk, and stops stream)
    pub fn trigger_update_overlay(&mut self) {
        if self.oneshot_snapshot.is_some() {
            return;
        }

        self.camera_calibration.show_on_canvas = true;
        let _ = self.camera_calibration.save();

        if self.camera_manager.is_streaming {
            if let Some(f) = self.camera_manager.take_snapshot() {
                self.save_bed_snapshot_to_disk(&f);
                self.last_raw_frame = Some(f);
                self.last_applied_calibration = None;
                self.status_msg = "Bed overlay updated.".to_string();
            }
        } else {
            let initial = self.camera_manager.get_latest_frame().map(|f| f.frame_number).unwrap_or(0);
            self.camera_manager.start_stream();
            self.oneshot_snapshot = Some(OneShotSnapshot {
                started_at: std::time::Instant::now(),
                was_streaming: false,
                initial_frame_number: initial,
            });
            self.status_msg = "Capturing fresh bed overlay photo from camera...".to_string();
        }
    }

    /// Calculate the 4 target coordinates in document mm space:
    /// 0: Top-Left (inset, inset)
    /// 1: Top-Right (width - inset, inset)
    /// 2: Bottom-Right (width - inset, height - inset)
    /// 3: Bottom-Left (inset, height - inset)
    pub fn alignment_target_doc_coords(&self) -> [(f64, f64); 4] {
        let inset = self.camera_calibration.target_inset_mm.clamp(10.0, self.doc.width.min(self.doc.height) * 0.45);
        let w = self.doc.width;
        let h = self.doc.height;
        [
            (inset, inset),           // 0: Top-Left
            (w - inset, inset),       // 1: Top-Right
            (w - inset, h - inset),   // 2: Bottom-Right
            (inset, h - inset),       // 3: Bottom-Left
        ]
    }

    /// Move the physical laser head directly over target index 0..3 with weak light spot turned on
    pub fn jog_to_alignment_target(&mut self, idx: usize) {
        let targets = self.alignment_target_doc_coords();
        if idx < 4 {
            let (doc_x, doc_y) = targets[idx];
            if !self.device.is_weak_light_on {
                self.device.set_weak_light(true);
            }
            self.device.go_to_point(Point2D::new(doc_x, doc_y));
            self.status_msg = format!("Laser positioned at Target {} (X: {:.1}, Y: {:.1} mm). Place paper under the laser spot!", idx + 1, doc_x, doc_y);
        }
    }

    /// Generate rapid G-code for 4 bullseye alignment targets (crosshairs + concentric circles)
    pub fn generate_alignment_targets_gcode(&self) -> String {
        let targets = self.alignment_target_doc_coords();
        let speed = self.camera_calibration.burn_speed.clamp(100.0, 10000.0);
        let max_s = self.device.settings.max_s_value;
        let power_s = (max_s * (self.camera_calibration.burn_power_percent.clamp(1.0, 100.0) as f64 / 100.0)).round();

        let mut lines = Vec::new();
        lines.push("; LaserBurn 4-Point Camera Alignment Targets".to_string());
        lines.push("G21 ; millimeters".to_string());
        lines.push("G90 ; absolute coordinates".to_string());
        lines.push("M5 ; ensure laser off".to_string());

        for (idx, &(doc_x, doc_y)) in targets.iter().enumerate() {
            let m_pt = self.device.doc_to_machine(Point2D::new(doc_x, doc_y));
            let mx = m_pt.x;
            let my = m_pt.y;

            lines.push(format!("; Target {} at doc X{:.1} Y{:.1} -> machine X{:.1} Y{:.1}", idx + 1, doc_x, doc_y, mx, my));
            lines.push(format!("G0 X{:.3} Y{:.3} F{:.0}", mx, my, self.device.settings.rapid_speed));

            // Horizontal crosshair (10mm)
            lines.push(format!("G0 X{:.3} Y{:.3}", mx - 5.0, my));
            lines.push(format!("M3 S{:.0}", power_s));
            lines.push(format!("G1 X{:.3} Y{:.3} F{:.0}", mx + 5.0, my, speed));
            lines.push("M5".to_string());

            // Vertical crosshair (10mm)
            lines.push(format!("G0 X{:.3} Y{:.3}", mx, my - 5.0));
            lines.push(format!("M3 S{:.0}", power_s));
            lines.push(format!("G1 X{:.3} Y{:.3} F{:.0}", mx, my + 5.0, speed));
            lines.push("M5".to_string());

            // Inner circle (diameter 6mm)
            lines.push(format!("G0 X{:.3} Y{:.3}", mx + 3.0, my));
            lines.push(format!("M3 S{:.0}", power_s));
            lines.push(format!("G2 X{:.3} Y{:.3} I-3.0 J0.0 F{:.0}", mx + 3.0, my, speed));
            lines.push("M5".to_string());

            // Outer circle (diameter 12mm)
            lines.push(format!("G0 X{:.3} Y{:.3}", mx + 6.0, my));
            lines.push(format!("M3 S{:.0}", power_s));
            lines.push(format!("G2 X{:.3} Y{:.3} I-6.0 J0.0 F{:.0}", mx + 6.0, my, speed));
            lines.push("M5".to_string());
        }

        // Return to home/origin
        lines.push("M5 ; laser off".to_string());
        lines.push(format!("G0 X0 Y0 F{:.0}", self.device.settings.rapid_speed));
        lines.join("\n")
    }

    /// Burn the 4 alignment targets onto the scrap papers placed on the bed
    pub fn burn_alignment_targets(&mut self) {
        let gcode = self.generate_alignment_targets_gcode();
        self.device.start_job(&gcode);
        self.status_msg = "Burning 4 alignment targets onto scrap papers...".to_string();
    }

    /// Screen position of the Origin point (0, 0) - positioned at the machine's origin corner
    pub fn origin_screen_pos(&self, canvas_origin: egui::Pos2) -> egui::Pos2 {
        let bed_w = self.doc.width as f32 * self.zoom;
        let bed_h = self.doc.height as f32 * self.zoom;
        let bed_rect = egui::Rect::from_min_size(canvas_origin, egui::Vec2::new(bed_w, bed_h));

        match self.device.settings.origin_corner {
            crate::device::OriginCorner::TopLeft => bed_rect.left_top(),
            crate::device::OriginCorner::TopRight => bed_rect.right_top(),
            crate::device::OriginCorner::BottomLeft => bed_rect.left_bottom(),
            crate::device::OriginCorner::BottomRight => bed_rect.right_bottom(),
        }
    }

    /// Convert machine coordinate (Point2D relative to machine origin corner) to screen position (egui::Pos2)
    pub fn machine_to_screen(&self, pt: Point2D, canvas_origin: egui::Pos2) -> egui::Pos2 {
        let bed_w = self.doc.width as f32 * self.zoom;
        let bed_h = self.doc.height as f32 * self.zoom;
        let bed_rect = egui::Rect::from_min_size(canvas_origin, egui::Vec2::new(bed_w, bed_h));

        match self.device.settings.origin_corner {
            crate::device::OriginCorner::TopLeft => egui::pos2(
                bed_rect.left() + pt.x as f32 * self.zoom,
                bed_rect.top() + pt.y as f32 * self.zoom,
            ),
            crate::device::OriginCorner::TopRight => egui::pos2(
                bed_rect.right() - pt.x as f32 * self.zoom,
                bed_rect.top() + pt.y as f32 * self.zoom,
            ),
            crate::device::OriginCorner::BottomLeft => egui::pos2(
                bed_rect.left() + pt.x as f32 * self.zoom,
                bed_rect.bottom() - pt.y as f32 * self.zoom,
            ),
            crate::device::OriginCorner::BottomRight => egui::pos2(
                bed_rect.right() - pt.x as f32 * self.zoom,
                bed_rect.bottom() - pt.y as f32 * self.zoom,
            ),
        }
    }

    /// Screen position of the physical laser head, relative to the machine's origin corner
    pub fn machine_head_screen_pos(&self, canvas_origin: egui::Pos2) -> egui::Pos2 {
        self.machine_to_screen(
            Point2D::new(self.device.machine_x, self.device.machine_y),
            canvas_origin,
        )
    }

    /// Starts real laser cutting / engraving job
    pub fn start_job(&mut self) {
        if self.doc.shapes.is_empty() {
            self.status_msg = "No shapes to cut!".to_string();
            return;
        }

        if self.device.status != crate::device::ConnectionStatus::Connected {
            self.status_msg = "Connect laser first (Connection Settings)!".to_string();
            self.show_connection_window = true;
            return;
        }

        let opt = CutOptimizer::new();
        let job = opt.optimize(&self.doc);

        let mut cfg = GCodeConfig::default();
        cfg.max_s_value = self.device.settings.max_s_value;
        cfg.rapid_speed = self.device.settings.rapid_speed;
        cfg.bed_width = self.device.settings.bed_width;
        cfg.bed_height = self.device.settings.bed_height;
        cfg.mirror_x = self.device.settings.origin_corner == crate::device::OriginCorner::TopRight
            || self.device.settings.origin_corner == crate::device::OriginCorner::BottomRight;
        cfg.mirror_y = self.device.settings.origin_corner == crate::device::OriginCorner::BottomLeft
            || self.device.settings.origin_corner == crate::device::OriginCorner::BottomRight;
        let gen = GCodeGenerator::new(cfg);
        let gcode = gen.generate(&self.doc, &job);

        self.device.start_job(&gcode);
        self.status_msg = format!("Cut job started ({} paths, {} commands)...", job.paths.len(), self.device.job_total_lines);
    }
}

#[cfg(feature = "gui")]
impl eframe::App for LaserBurnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll incoming responses / status from controller (including real-time ? status)
        self.device.poll_serial();

        // Motion simulation for smooth moves in offline / demo mode
        let dt = ctx.input(|i| i.stable_dt).min(0.1) as f64;
        self.device.update_motion_simulation(dt);

        // Step active laser job if running
        if self.device.is_job_running {
            self.device.step_job();
            ctx.request_repaint_after(std::time::Duration::from_millis(15));
        }

        // Step framing test motion if active
        if self.device.is_framing_active {
            self.device.step_frame_test();
            ctx.request_repaint_after(std::time::Duration::from_millis(20));
        }

        // Continuous smooth repaint when camera video stream is active (~30-40 FPS)
        if self.camera_manager.is_streaming {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
        }

        // Advance One-Shot Bed Snapshot capture
        if let Some(ref snapshot_state) = self.oneshot_snapshot {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));

            let elapsed = snapshot_state.started_at.elapsed();
            let latest = self.camera_manager.get_latest_frame();
            let has_new_frame = latest.as_ref().map_or(false, |f| f.frame_number > snapshot_state.initial_frame_number + 2);

            // Wait ~350ms (or 2+ new frames) for camera auto-exposure to stabilize, max 1800ms
            if (has_new_frame && elapsed >= std::time::Duration::from_millis(350)) || elapsed >= std::time::Duration::from_millis(1800) {
                let was_streaming = snapshot_state.was_streaming;
                if let Some(frame) = self.camera_manager.take_snapshot() {
                    self.save_bed_snapshot_to_disk(&frame);
                    self.last_raw_frame = Some(frame);
                    self.last_applied_calibration = None; // Force texture refresh
                    self.status_msg = "Bed overlay updated from camera.".to_string();
                } else {
                    self.status_msg = "Could not capture camera frame.".to_string();
                }

                if !was_streaming {
                    self.camera_manager.stop_stream();
                }
                self.oneshot_snapshot = None;
            }
        }

        // Fetch and undistort incoming camera frame or re-process when calibration changes
        let latest_frame = self.camera_manager.get_latest_frame();
        if let Some(ref f) = latest_frame {
            self.last_raw_frame = Some(f.clone());
        }

        let is_new_frame = latest_frame.as_ref().map_or(false, |f| f.frame_number != self.last_camera_frame_number);
        let cal_changed = self.last_applied_calibration.as_ref() != Some(&self.camera_calibration);

        if is_new_frame || (cal_changed && self.last_raw_frame.is_some()) {
            if let Some(ref raw) = self.last_raw_frame {
                let undistorted = self.lens_undistorter.undistort_frame(raw, &self.camera_calibration);
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [undistorted.width as usize, undistorted.height as usize],
                    &undistorted.rgba,
                );
                self.camera_texture = Some(ctx.load_texture(
                    "camera_live_texture",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
                if let Some(ref f) = latest_frame {
                    self.last_camera_frame_number = f.frame_number;
                }
                self.last_applied_calibration = Some(self.camera_calibration.clone());
            }
        }

        // Advance stop/abort sequence smoothly if active
        if self.device.abort_phase.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(15));
        }

        // Advance mechanical homing & zero-point calibration sequence if active
        if self.device.homing_phase.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(15));
        }

        // Continuously repaint while physical machine is moving or active!
        if self.device.is_moving() {
            ctx.request_repaint();
        } else if self.device.is_physically_connected() {
            // Keep polling serial port and updating live indicators (switches, position) at 20 Hz
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // Drain any completed asynchronous native file dialog results
        while let Ok(action) = self.file_dialog_rx.try_recv() {
            match action {
                FileDialogAction::OpenProject(path) => {
                    let _ = self.open_file(&path);
                }
                FileDialogAction::SaveProject(path) => {
                    let save_path = if path.extension().is_none() {
                        path.with_extension("lbrn2")
                    } else {
                        path
                    };
                    if let Err(e) = Lbrn2File::save_to_file(&self.doc, &save_path) {
                        self.status_msg = format!("Save Error: {}", e);
                    } else {
                        self.doc.name = save_path.file_stem().and_then(|s| s.to_str()).unwrap_or("project").to_string();
                        self.status_msg = format!("Saved: {}", save_path.file_name().unwrap_or_default().to_string_lossy());
                    }
                }
                FileDialogAction::ExportGCode(path) => {
                    let save_path = if path.extension().is_none() {
                        path.with_extension("gcode")
                    } else {
                        path
                    };
                    let opt = CutOptimizer::new();
                    let job = opt.optimize(&self.doc);
                    let gen = GCodeGenerator::new(GCodeConfig::default());
                    let gcode = gen.generate(&self.doc, &job);
                    if let Err(e) = std::fs::write(&save_path, gcode) {
                        self.status_msg = format!("Export Error: {}", e);
                    } else {
                        self.status_msg = format!("Exported G-Code to {}", save_path.file_name().unwrap_or_default().to_string_lossy());
                    }
                }
                FileDialogAction::ExportRuida(path) => {
                    let save_path = if path.extension().is_none() {
                        path.with_extension("rd")
                    } else {
                        path
                    };
                    let opt = CutOptimizer::new();
                    let job = opt.optimize(&self.doc);
                    let gen = RuidaGenerator::new();
                    let bytes = gen.generate_rd(&self.doc, &job);
                    if let Err(e) = std::fs::write(&save_path, bytes) {
                        self.status_msg = format!("Export Error: {}", e);
                    } else {
                        self.status_msg = format!("Exported Ruida to {}", save_path.file_name().unwrap_or_default().to_string_lossy());
                    }
                }
                FileDialogAction::BrowseFont(path) => {
                    self.text_font_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Custom").to_string();
                    self.text_font_path = path.to_string_lossy().to_string();
                }
            }
        }

        // Drag-and-drop file loading support (.lbrn2, .lbrn, .svg)
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for df in dropped {
            if let Some(ref path) = df.path {
                let _ = self.open_file(path);
            }
        }

        // -------------------------------------------------------------
        // KEYBOARD SHORTCUTS & ARROW KEYS HANDLING
        // -------------------------------------------------------------
        if !ctx.wants_keyboard_input() {
            // Cmd+O (Open Project)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
                self.prompt_open_project(ctx);
            }

            // Cmd+S (Save Project)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
                self.prompt_save_project(ctx);
            }

            // Cmd+A (Select All)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::A)) {
                self.selected_shape_indices = (0..self.doc.shapes.len()).collect();
                self.status_msg = format!("Selected all {} shape(s)", self.doc.shapes.len());
            }

            // Cmd+C (Copy)
            if ctx.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::C)) || i.events.iter().any(|e| matches!(e, egui::Event::Copy))) {
                self.copy_selected();
            }

            // Cmd+V (Paste)
            if ctx.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::V)) || i.events.iter().any(|e| matches!(e, egui::Event::Paste(_)))) {
                self.paste();
            }

            // Cmd+X (Cut)
            if ctx.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::X)) || i.events.iter().any(|e| matches!(e, egui::Event::Cut))) {
                self.cut_selected();
            }

            // Cmd+D (Duplicate)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::D)) {
                self.duplicate_selected();
            }

            // Delete / Backspace key to delete
            if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                self.delete_selected();
            }

            // Cmd+T (Open Text Tool)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::T)) {
                if self.text_pos_x == 50.0 && self.text_pos_y == 50.0 {
                    self.text_pos_x = self.doc.width * 0.25;
                    self.text_pos_y = self.doc.height * 0.35;
                }
                self.show_text_dialog = true;
            }

            // Cmd+M (Toggle Move & Laser Control Window)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::M)) {
                self.show_move_window = !self.show_move_window;
            }

            // Cmd+Comma (Toggle Settings Window)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Comma)) {
                self.show_settings_window = !self.show_settings_window;
            }

            // Cmd+K (Toggle Camera Window)
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::K)) {
                self.show_camera_window = !self.show_camera_window;
            }

            // Cmd+Shift+C (Toggle Camera Background on Grid Canvas)
            if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::C)) {
                self.camera_calibration.show_on_canvas = !self.camera_calibration.show_on_canvas;
                let _ = self.camera_calibration.save();
                self.status_msg = if self.camera_calibration.show_on_canvas {
                    "Camera background enabled on bed canvas".to_string()
                } else {
                    "Camera background hidden".to_string()
                };
            }

            // Cmd+U or Cmd+Shift+U (1-Click Update Bed Overlay Snapshot from Camera)
            if ctx.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::U)) || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::U))) {
                self.trigger_update_overlay();
            }

            // Keyboard Arrow Handling: Direct Laser Jogging vs Shape Nudging
            let wants_keyboard = ctx.wants_keyboard_input();
            if !wants_keyboard {
                let cmd_down = ctx.input(|i| i.modifiers.command);
                let shift_down = ctx.input(|i| i.modifiers.shift);
                let alt_down = ctx.input(|i| i.modifiers.alt);

                let mut arrow_x: f64 = 0.0;
                let mut arrow_y: f64 = 0.0;

                if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    arrow_x -= 1.0;
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    arrow_x += 1.0;
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    arrow_y += 1.0; // Y+ on laser bed
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    arrow_y -= 1.0; // Y- on laser bed
                }

                if arrow_x.abs() > 0.0 || arrow_y.abs() > 0.0 {
                    // Holding Cmd OR keyboard_jog_laser is false AND shapes are selected: nudge vector shapes!
                    let nudge_shapes = (cmd_down || !self.keyboard_jog_laser) && !self.selected_shape_indices.is_empty();

                    if nudge_shapes {
                        let step: f64 = if shift_down { 10.0 } else if alt_down { 0.1 } else { 1.0 };
                        let dx = arrow_x * step;
                        let dy = -arrow_y * step; // visual Y inversion for canvas
                        for &idx in &self.selected_shape_indices {
                            if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                self.doc.shapes[idx].translate(dx, dy);
                            }
                        }
                        self.status_msg = format!("Nudged selection by X: {:.1}, Y: {:.1} mm (Cmd+Arrow)", dx, dy);
                    } else if self.keyboard_jog_laser {
                        // Direct Laser Head Jogging via keyboard arrows!
                        let step = if shift_down {
                            (self.jog_step * 5.0).max(10.0) // Shift = fast coarse move
                        } else if alt_down {
                            (self.jog_step * 0.1).max(0.1)  // Alt/Option = fine move
                        } else {
                            self.jog_step
                        };
                        self.device.jog_arrow(arrow_x, arrow_y, step, self.jog_speed);
                        self.status_msg = format!(
                            "Laser Jog (Keyboard): X: {:.2}, Y: {:.2} mm (Step: {:.1}mm)",
                            self.device.work_x, self.device.work_y, step
                        );
                        ctx.request_repaint();
                    }
                }
            }

            if !self.selected_shape_indices.is_empty() {
                // Cmd+G (Group), Cmd+U (Ungroup), Cmd+L (Lock/Unlock)
                if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::G)) {
                    self.group_selected();
                }
                if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::U)) {
                    self.ungroup_selected();
                }
                if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::L)) {
                    self.toggle_lock_selected();
                }
                // Rotate selection:
                // Counter-clockwise (CCW): Q, Comma (,), OpenBracket ([), or Option+8
                // Clockwise (CW): E, Period (.), CloseBracket (]), or Option+9
                // Shift = 5°, Cmd = 90°, plain = 1°
                let shift_down = ctx.input(|i| i.modifiers.shift);
                let cmd_down = ctx.input(|i| i.modifiers.command);
                let alt_down = ctx.input(|i| i.modifiers.alt);

                let rot_ccw = (!cmd_down && ctx.input(|i| i.key_pressed(egui::Key::Q)))
                    || (!cmd_down && ctx.input(|i| i.key_pressed(egui::Key::Comma)))
                    || ctx.input(|i| i.key_pressed(egui::Key::OpenBracket))
                    || (alt_down && ctx.input(|i| i.key_pressed(egui::Key::Num8)));

                let rot_cw = (!cmd_down && ctx.input(|i| i.key_pressed(egui::Key::E)))
                    || (!cmd_down && ctx.input(|i| i.key_pressed(egui::Key::Period)))
                    || ctx.input(|i| i.key_pressed(egui::Key::CloseBracket))
                    || (alt_down && ctx.input(|i| i.key_pressed(egui::Key::Num9)));

                if rot_ccw {
                    let angle = if shift_down { -5.0 } else if cmd_down { -90.0 } else { -1.0 };
                    self.rotate_selected(angle);
                }
                if rot_cw {
                    let angle = if shift_down { 5.0 } else if cmd_down { 90.0 } else { 1.0 };
                    self.rotate_selected(angle);
                }
            }
        }

        // -------------------------------------------------------------
        // TOP MENU & TOOLBAR
        // -------------------------------------------------------------
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            let cmd = Self::cmd_key();
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        self.doc = Document::new("Untitled", self.device.settings.bed_width, self.device.settings.bed_height);
                        self.selected_shape_indices.clear();
                        self.selected_layer = 0;
                        ui.close_menu();
                    }
                    if ui.button(format!("Open .lbrn2 / .lbrn / .svg... ({}+O)", cmd)).clicked() {
                        self.prompt_open_project(ctx);
                        ui.close_menu();
                    }
                    if ui.button(format!("Save Project (.lbrn2)... ({}+S)", cmd)).clicked() {
                        self.prompt_save_project(ctx);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export G-Code (GRBL)...").clicked() {
                        self.prompt_export_gcode(ctx);
                        ui.close_menu();
                    }
                    if ui.button("Export Ruida (.rd)...").clicked() {
                        self.prompt_export_ruida(ctx);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Connection Settings...").clicked() {
                        self.show_connection_window = true;
                        ui.close_menu();
                    }
                    if ui.button(format!("Settings... ({}+,)", cmd)).clicked() {
                        self.show_settings_window = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let has_selection = !self.selected_shape_indices.is_empty();
                    let has_clipboard = !self.clipboard_shapes.is_empty();

                    if ui.add_enabled(has_selection, egui::Button::new(format!("Cut ({}+X)", cmd))).clicked() {
                        self.cut_selected();
                        ui.close_menu();
                    }
                    if ui.add_enabled(has_selection, egui::Button::new(format!("Copy ({}+C)", cmd))).clicked() {
                        self.copy_selected();
                        ui.close_menu();
                    }
                    if ui.add_enabled(has_clipboard, egui::Button::new(format!("Paste ({}+V)", cmd))).clicked() {
                        self.paste();
                        ui.close_menu();
                    }
                    if ui.add_enabled(has_selection, egui::Button::new(format!("Duplicate ({}+D)", cmd))).clicked() {
                        self.duplicate_selected();
                        ui.close_menu();
                    }
                    if ui.add_enabled(has_selection, egui::Button::new("Delete (Del)")).clicked() {
                        self.delete_selected();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.add_enabled(self.selected_shape_indices.len() >= 2, egui::Button::new(format!("Group ({}+G)", cmd))).clicked() {
                        self.group_selected();
                        ui.close_menu();
                    }
                    if ui.add_enabled(has_selection, egui::Button::new(format!("Ungroup ({}+U)", cmd))).clicked() {
                        self.ungroup_selected();
                        ui.close_menu();
                    }
                    if ui.add_enabled(has_selection, egui::Button::new(format!("Toggle Lock ({}+L)", cmd))).clicked() {
                        self.toggle_lock_selected();
                        ui.close_menu();
                    }
                    if ui.add_enabled(self.selected_shape_indices.len() >= 2, egui::Button::new("Join / Weld Paths")).clicked() {
                        self.weld_selected();
                        ui.close_menu();
                    }
                    if ui.button(format!("Add Text... ({}+T)", cmd)).clicked() {
                        self.show_text_dialog = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(format!("Select All ({}+A)", cmd)).clicked() {
                        self.selected_shape_indices = (0..self.doc.shapes.len()).collect();
                        ui.close_menu();
                    }
                    if ui.button("Clear Canvas").clicked() {
                        self.doc.shapes.clear();
                        self.selected_shape_indices.clear();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Laser", |ui| {
                    if ui.button(format!("Laser Control & Jogging ({}+M)", cmd)).clicked() {
                        self.show_move_window = true;
                        ui.close_menu();
                    }
                    let is_on = self.device.is_weak_light_on;
                    let weak_label = if is_on { "Turn Off Weak Beam" } else { "Turn On Weak Beam" };
                    if ui.button(weak_label).clicked() {
                        self.device.toggle_weak_light();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Test Box Frame").clicked() {
                        self.start_box_frame();
                        ui.close_menu();
                    }
                    if ui.button("Test Cut Contour (Shapes)").clicked() {
                        self.start_contour_frame();
                        ui.close_menu();
                    }
                    ui.separator();
                    if self.device.settings.enable_limit_switches {
                        if ui.button("Reset Origin to Limit Switches").clicked() {
                            self.device.start_mechanical_home_reset();
                            ui.close_menu();
                        }
                        if ui.button("Home Machine ($H)").clicked() {
                            self.device.home();
                            ui.close_menu();
                        }
                    }
                    if ui.button("Set Origin Here (G92)").clicked() {
                        self.device.set_origin_here();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Camera", |ui| {
                    if ui.button(format!("Camera Feed... ({}+K)", cmd)).clicked() {
                        self.show_camera_window = true;
                        ui.close_menu();
                    }
                    if ui.button("Scan Devices").clicked() {
                        self.camera_manager.refresh_devices();
                        ui.close_menu();
                    }
                    ui.separator();
                    if !self.camera_manager.is_streaming {
                        if ui.button("Start Stream").clicked() {
                            self.camera_manager.start_stream();
                            ui.close_menu();
                        }
                    } else {
                        if ui.button("Stop Stream").clicked() {
                            self.camera_manager.stop_stream();
                            ui.close_menu();
                        }
                    }
                    if ui.button("Update Bed Overlay (Cmd+U)").clicked() {
                        self.trigger_update_overlay();
                        ui.close_menu();
                    }
                });

                ui.separator();
                ui.label(egui::RichText::new("LaserBurn v0.1.0").strong().color(egui::Color32::from_rgb(255, 140, 0)));
                ui.label(egui::RichText::new("(Arrow keys nudge selection)").italics().color(egui::Color32::GRAY));
            });

            ui.separator();

            // Main Action Toolbar (Wraps cleanly if screen is smaller)
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.active_tool, ActiveTool::Select, "Select / Move");
                ui.selectable_value(&mut self.active_tool, ActiveTool::DrawRect, "Rect");
                ui.selectable_value(&mut self.active_tool, ActiveTool::DrawCircle, "Circle");
                ui.selectable_value(&mut self.active_tool, ActiveTool::DrawLine, "Line");
                ui.selectable_value(&mut self.active_tool, ActiveTool::DrawText, "Text");

                ui.separator();

                // Clipboard & Editing Operations
                let has_selection = !self.selected_shape_indices.is_empty();
                let multi_selected = self.selected_shape_indices.len() >= 2;
                let has_clipboard = !self.clipboard_shapes.is_empty();

                if ui.add_enabled(has_selection, egui::Button::new("Copy")).on_hover_text(format!("Copy selected shapes to clipboard ({}+C)", cmd)).clicked() {
                    self.copy_selected();
                }
                if ui.add_enabled(has_clipboard, egui::Button::new("Paste")).on_hover_text(format!("Paste shapes from clipboard ({}+V)", cmd)).clicked() {
                    self.paste();
                }
                if ui.add_enabled(has_selection, egui::Button::new("Duplicate")).on_hover_text(format!("Duplicate selected shapes ({}+D)", cmd)).clicked() {
                    self.duplicate_selected();
                }
                if ui.add_enabled(has_selection, egui::Button::new("Delete")).on_hover_text("Delete selected shapes (Backspace/Delete)").clicked() {
                    self.delete_selected();
                }

                ui.separator();

                // Group, Ungroup, Lock, Weld Buttons
                if ui.add_enabled(multi_selected, egui::Button::new("Group")).on_hover_text(format!("Group selected shapes into a single item ({}+G)", cmd)).clicked() {
                    self.group_selected();
                }
                if ui.add_enabled(has_selection, egui::Button::new("Ungroup")).on_hover_text(format!("Ungroup selected shapes into separate items ({}+U)", cmd)).clicked() {
                    self.ungroup_selected();
                }
                if ui.add_enabled(multi_selected, egui::Button::new("Weld")).on_hover_text("Join / weld paths into continuous geometry").clicked() {
                    self.weld_selected();
                }

                // Lock toggle button
                if has_selection {
                    let is_locked = self.selected_shape_indices.iter().any(|&i| i < self.doc.shapes.len() && self.doc.shapes[i].is_locked());
                    let lock_btn_text = if is_locked { format!("Unlock ({}+L)", cmd) } else { format!("Lock ({}+L)", cmd) };
                    if ui.button(lock_btn_text).on_hover_text("Lock or unlock selected shapes in place").clicked() {
                        self.toggle_lock_selected();
                    }
                }

                ui.separator();

                if ui.button("+ Gear").clicked() {
                    self.load_sample_gear(180.0, 180.0, 12, 40.0, 50.0);
                    self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                    self.status_msg = "Added gear".to_string();
                }
                if ui.button("+ Square").clicked() {
                    self.doc.add_shape(Shape::Rect {
                        layer_index: self.selected_layer,
                        x: 80.0,
                        y: 80.0,
                        width: 40.0,
                        height: 40.0,
                        corner_radius: 0.0,
                    });
                    self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                    self.status_msg = "Added square".to_string();
                }
                if ui.button("+ Circle").clicked() {
                    self.doc.add_shape(Shape::Circle {
                        layer_index: self.selected_layer,
                        center: Point2D::new(120.0, 120.0),
                        radius: 25.0,
                    });
                    self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                    self.status_msg = "Added circle".to_string();
                }
                if ui.button("+ Text").on_hover_text("Create new text with TrueType or Single-Line font (Cmd+T)").clicked() {
                    if self.text_pos_x == 50.0 && self.text_pos_y == 50.0 {
                        self.text_pos_x = self.doc.width * 0.25;
                        self.text_pos_y = self.doc.height * 0.35;
                    }
                    self.show_text_dialog = true;
                }

                ui.separator();

                // Laser Execution Buttons (Start / Pause / Resume / Stop)
                if !self.device.is_job_running {
                    let start_btn = egui::Button::new(
                        egui::RichText::new(" START ").strong().color(egui::Color32::WHITE),
                    ).fill(egui::Color32::from_rgb(34, 139, 34));
                    if ui.add(start_btn).on_hover_text("Start cut job (send G-code to laser)").clicked() {
                        self.start_job();
                    }
                } else {
                    if self.device.is_job_paused {
                        let resume_btn = egui::Button::new(
                            egui::RichText::new(" RESUME ").strong().color(egui::Color32::WHITE),
                        ).fill(egui::Color32::from_rgb(0, 140, 255));
                        if ui.add(resume_btn).on_hover_text("Resume paused job").clicked() {
                            self.device.resume_job();
                        }
                    } else {
                        let pause_btn = egui::Button::new(
                            egui::RichText::new(" PAUSE ").strong().color(egui::Color32::BLACK),
                        ).fill(egui::Color32::from_rgb(255, 193, 7));
                        if ui.add(pause_btn).on_hover_text("Pause job temporarily").clicked() {
                            self.device.pause_job();
                        }
                    }

                    let stop_btn = egui::Button::new(
                        egui::RichText::new(" STOP ").strong().color(egui::Color32::WHITE),
                    ).fill(egui::Color32::RED);
                    if ui.add(stop_btn).on_hover_text("Emergency stop cut job (M5)").clicked() {
                        self.device.stop_job();
                    }
                }

                ui.separator();

                // Move / Jog Window
                if ui.button("Jog").on_hover_text("Open laser control and jog window (Cmd+M)").clicked() {
                    self.show_move_window = !self.show_move_window;
                }

                // Weak light button
                let is_weak_on = self.device.is_weak_light_on;
                let valo_btn = egui::Button::new(
                    egui::RichText::new(if is_weak_on { "Beam ON" } else { "Weak Beam" })
                        .strong()
                        .color(if is_weak_on { egui::Color32::WHITE } else { egui::Color32::from_rgb(200, 230, 255) }),
                ).fill(if is_weak_on { egui::Color32::from_rgb(255, 69, 0) } else { egui::Color32::from_rgb(40, 70, 110) });

                if ui.add(valo_btn).on_hover_text("Toggle weak alignment beam on/off").clicked() {
                    self.device.toggle_weak_light();
                }

                // Frame tests
                if ui.button("Box Frame").on_hover_text("Test bounding box frame with weak beam").clicked() {
                    self.start_box_frame();
                }
                if ui.button("Contour").on_hover_text("Test cut contour with weak beam").clicked() {
                    self.start_contour_frame();
                }

                if self.device.settings.enable_limit_switches {
                    if ui.button("Home").on_hover_text("Home laser machine to physical switches ($H)").clicked() {
                        self.device.home();
                    }
                }

                ui.separator();
                if ui.button("Connection").on_hover_text("Device connection & serial console").clicked() {
                    self.show_connection_window = true;
                }
                if ui.button("Camera").on_hover_text(format!("Open Camera Feed & Alignment window ({}+K)", cmd)).clicked() {
                    self.show_camera_window = !self.show_camera_window;
                }
                let bg_btn_text = if self.camera_calibration.show_on_canvas { "Cam BG" } else { "Cam BG" };
                let bg_btn_color = if self.camera_calibration.show_on_canvas {
                    egui::Color32::from_rgb(100, 210, 255)
                } else {
                    egui::Color32::GRAY
                };
                if ui.button(egui::RichText::new(bg_btn_text).color(bg_btn_color)).on_hover_text(format!("Toggle camera background overlay on the laser bed canvas ({}+Shift+C)", cmd)).clicked() {
                    self.camera_calibration.show_on_canvas = !self.camera_calibration.show_on_canvas;
                    let _ = self.camera_calibration.save();
                }

                let is_capturing = self.oneshot_snapshot.is_some();
                let snap_btn = egui::Button::new(
                    egui::RichText::new(if is_capturing { "Capturing..." } else { "Update Overlay" })
                        .strong()
                        .color(if is_capturing { egui::Color32::from_rgb(255, 230, 80) } else { egui::Color32::WHITE })
                ).fill(if is_capturing { egui::Color32::from_rgb(180, 100, 20) } else { egui::Color32::from_rgb(33, 115, 185) });

                if ui.add(snap_btn).on_hover_text(format!("Capture a fresh photo of the bed with your material (Cmd+U / {}+Shift+U)", cmd)).clicked() {
                    self.trigger_update_overlay();
                }
                if ui.button("Settings").on_hover_text(format!("Device & bed settings ({}+,)", cmd)).clicked() {
                    self.show_settings_window = true;
                }

                ui.separator();
                if ui.button("Fit").on_hover_text("Fit bed to view").clicked() {
                    self.zoom = 1.8;
                    self.pan = egui::Vec2::new(100.0, 80.0);
                }
                if ui.button(egui::RichText::new(" + ").strong()).on_hover_text("Zoom in").clicked() {
                    self.zoom = (self.zoom * 1.25).clamp(0.2, 20.0);
                }
                if ui.button(egui::RichText::new(" - ").strong()).on_hover_text("Zoom out").clicked() {
                    self.zoom = (self.zoom * 0.8).clamp(0.2, 20.0);
                }
            });
        });

        // -------------------------------------------------------------
        // BOTTOM STATUS BAR
        // -------------------------------------------------------------
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let is_conn = self.device.status == crate::device::ConnectionStatus::Connected;
                let (badge_color, badge_text) = if is_conn {
                    if self.device.is_physically_connected() {
                        (egui::Color32::from_rgb(0, 220, 0), "Connected (USB)")
                    } else {
                        (egui::Color32::from_rgb(255, 200, 50), "Simulator (Demo)")
                    }
                } else {
                    (egui::Color32::RED, "Disconnected")
                };

                if ui.button(egui::RichText::new(badge_text).color(badge_color)).on_hover_text("Open device connection & console window").clicked() {
                    self.show_connection_window = true;
                }
                ui.label(format!("Controller: {}", self.device.controller_type.name()));
                ui.separator();

                ui.label(format!("Machine Pos: X: {:.1} mm, Y: {:.1} mm", self.device.machine_x, self.device.machine_y));
                ui.separator();

                if self.device.is_job_running {
                    let pct = self.device.job_progress * 100.0;
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), format!("CUTTING: {:.1}% ({}/{})", pct, self.device.job_idx, self.device.job_total_lines));
                    ui.separator();
                } else if self.device.is_weak_light_on {
                    ui.colored_label(egui::Color32::from_rgb(0, 220, 255), "WEAK BEAM (1.0%)");
                    ui.separator();
                }

                if self.device.is_framing_active {
                    ui.colored_label(egui::Color32::YELLOW, "FRAMING IN PROGRESS...");
                    ui.separator();
                }

                ui.label(egui::RichText::new(&self.status_msg).strong());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Bed: {:.0} x {:.0} mm | Shapes: {} | Selected: {}", self.doc.width, self.doc.height, self.doc.shapes.len(), self.selected_shape_indices.len()));
                });
            });
        });

        // -------------------------------------------------------------
        // RIGHT SIDE PANEL: PROPERTIES, TRANSFORM, LAYERS, JOG
        // -------------------------------------------------------------
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .min_width(340.0)
            .max_width(450.0)
            .default_width(380.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                    // -------------------------------------------------
                    // 1. TRANSFORM & SIZE INSPECTOR
                    // -------------------------------------------------
                    ui.heading("Transform & Size");
                    ui.separator();

                    if let Some(bounds) = self.selection_bounds() {
                        let mut cur_x = bounds.min_x;
                        let mut cur_y = bounds.min_y;
                        let mut cur_w = bounds.width();
                        let mut cur_h = bounds.height();

                        let count = self.selected_shape_indices.len();
                        ui.horizontal(|ui| {
                            let label = if count == 1 {
                                let idx = self.selected_shape_indices[0];
                                let is_locked = self.doc.shapes[idx].is_locked();
                                format!("Shape #{} {} Selected", idx + 1, if is_locked { "[LOCKED]" } else { "" })
                            } else {
                                format!("{} Shapes Selected", count)
                            };
                            ui.label(egui::RichText::new(label).strong().color(egui::Color32::from_rgb(255, 140, 0)));
                            if ui.button("Deselect").clicked() {
                                self.selected_shape_indices.clear();
                            }
                        });

                        // Position controls (allows shapes anywhere, even beyond the bed)
                        ui.horizontal(|ui| {
                            ui.label("X (mm):");
                            if ui.add(egui::DragValue::new(&mut cur_x).speed(1.0).range(-2000.0..=5000.0)).changed() {
                                let dx = cur_x - bounds.min_x;
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].translate(dx, 0.0);
                                    }
                                }
                            }
                            ui.label("Y (mm):");
                            if ui.add(egui::DragValue::new(&mut cur_y).speed(1.0).range(-2000.0..=5000.0)).changed() {
                                let dy = cur_y - bounds.min_y;
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].translate(0.0, dy);
                                    }
                                }
                            }
                        });

                        // Size controls
                        ui.horizontal(|ui| {
                            ui.label("Width:");
                            if ui.add(egui::DragValue::new(&mut cur_w).speed(1.0).range(0.1..=5000.0)).changed() {
                                let new_h = if self.lock_aspect_ratio && bounds.width() > 1e-4 {
                                    cur_w * (bounds.height() / bounds.width())
                                } else {
                                    cur_h
                                };
                                let sx = cur_w / bounds.width().max(1e-4);
                                let sy = new_h / bounds.height().max(1e-4);
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].scale(Point2D::new(bounds.min_x, bounds.min_y), sx, sy);
                                    }
                                }
                            }

                            ui.label("Height:");
                            if ui.add(egui::DragValue::new(&mut cur_h).speed(1.0).range(0.1..=5000.0)).changed() {
                                let new_w = if self.lock_aspect_ratio && bounds.height() > 1e-4 {
                                    cur_h * (bounds.width() / bounds.height())
                                } else {
                                    cur_w
                                };
                                let sx = new_w / bounds.width().max(1e-4);
                                let sy = cur_h / bounds.height().max(1e-4);
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].scale(Point2D::new(bounds.min_x, bounds.min_y), sx, sy);
                                    }
                                }
                            }
                        });

                        ui.checkbox(&mut self.lock_aspect_ratio, "Lock Aspect Ratio");

                        // Nudge Buttons (also mention keyboard arrow keys!)
                        ui.horizontal(|ui| {
                            ui.label("Nudge (5mm):");
                            if ui.button("X-").clicked() {
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].translate(-5.0, 0.0);
                                    }
                                }
                            }
                            if ui.button("X+").clicked() {
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].translate(5.0, 0.0);
                                    }
                                }
                            }
                            if ui.button("Y-").clicked() {
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].translate(0.0, -5.0);
                                    }
                                }
                            }
                            if ui.button("Y+").clicked() {
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].translate(0.0, 5.0);
                                    }
                                }
                            }
                        });

                        ui.label(egui::RichText::new("Tip: You can nudge directly with arrow keys (Shift = 10mm, Alt = 0.1mm)").italics().size(11.0).color(egui::Color32::from_rgb(100, 160, 255)));

                        // Quick Scaling and Rotation
                        ui.horizontal(|ui| {
                            ui.label("Scale:");
                            let c = bounds.center();
                            if ui.button("+10%").clicked() {
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].scale(c, 1.10, 1.10);
                                    }
                                }
                            }
                            if ui.button("-10%").clicked() {
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].scale(c, 0.90, 0.90);
                                    }
                                }
                            }
                            if ui.button("2x").clicked() {
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].scale(c, 2.0, 2.0);
                                    }
                                }
                            }
                            if ui.button("0.5x").clicked() {
                                for &idx in &self.selected_shape_indices {
                                    if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                        self.doc.shapes[idx].scale(c, 0.5, 0.5);
                                    }
                                }
                            }
                        });

                        // Rotation controls (1°, 5°, 15°, 45°, 90°, custom step)
                        ui.horizontal(|ui| {
                            ui.label("Rotate (fine):");
                            if ui.button("-1°").on_hover_text("Rotate -1° counter-clockwise (Keys: Q, Comma ,, or Option+8)").clicked() {
                                self.rotate_selected(-1.0);
                            }
                            if ui.button("+1°").on_hover_text("Rotate +1° clockwise (Keys: E, Period ., or Option+9)").clicked() {
                                self.rotate_selected(1.0);
                            }
                            if ui.button("-5°").on_hover_text("Rotate -5° counter-clockwise (Shift+Q, Shift+Comma)").clicked() {
                                self.rotate_selected(-5.0);
                            }
                            if ui.button("+5°").on_hover_text("Rotate +5° clockwise (Shift+E, Shift+Period)").clicked() {
                                self.rotate_selected(5.0);
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Rotate (coarse):");
                            if ui.button("-15°").on_hover_text("Rotate -15° counter-clockwise").clicked() {
                                self.rotate_selected(-15.0);
                            }
                            if ui.button("+15°").on_hover_text("Rotate +15° clockwise").clicked() {
                                self.rotate_selected(15.0);
                            }
                            if ui.button("-45°").on_hover_text("Rotate -45° counter-clockwise").clicked() {
                                self.rotate_selected(-45.0);
                            }
                            if ui.button("+45°").on_hover_text("Rotate +45° clockwise").clicked() {
                                self.rotate_selected(45.0);
                            }
                            if ui.button("-90°").on_hover_text("Rotate -90° counter-clockwise").clicked() {
                                self.rotate_selected(-90.0);
                            }
                            if ui.button("+90°").on_hover_text("Rotate +90° clockwise").clicked() {
                                self.rotate_selected(90.0);
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Angle Step:");
                            ui.add(
                                egui::DragValue::new(&mut self.custom_rot_angle)
                                    .speed(0.1)
                                    .range(0.01..=360.0)
                                    .suffix("°")
                            );
                            if ui.button("- Step").on_hover_text("Rotate counter-clockwise by step angle").clicked() {
                                self.rotate_selected(-self.custom_rot_angle);
                            }
                            if ui.button("+ Step").on_hover_text("Rotate clockwise by step angle").clicked() {
                                self.rotate_selected(self.custom_rot_angle);
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Flip:");
                            if ui.button("Flip H").on_hover_text("Flip selected shapes horizontally").clicked() {
                                self.flip_selected_h();
                            }
                            if ui.button("Flip V").on_hover_text("Flip selected shapes vertically").clicked() {
                                self.flip_selected_v();
                            }
                        });

                        // Layer Assignment (single or multi-selection)
                        if count >= 1 {
                            let first_idx = self.selected_shape_indices[0];
                            let cur_layer = self.doc.shapes[first_idx].layer_index();
                            let cur_layer_name = self.doc.layers.get(&cur_layer).map(|l| l.name.as_str()).unwrap_or("");
                            ui.horizontal(|ui| {
                                ui.label("Layer:");
                                egui::ComboBox::from_id_salt("shape_layer_combo")
                                    .selected_text(format!("[C{:02}] {}", cur_layer, cur_layer_name))
                                    .show_ui(ui, |ui| {
                                        for (&layer_idx, layer) in &self.doc.layers {
                                            if ui.selectable_label(cur_layer == layer_idx, format!("[C{:02}] {}", layer_idx, layer.name)).clicked() {
                                                for &s_idx in &self.selected_shape_indices {
                                                    if s_idx < self.doc.shapes.len() {
                                                        self.doc.shapes[s_idx].set_layer_index(layer_idx);
                                                    }
                                                }
                                                self.selected_layer = layer_idx;
                                            }
                                        }
                                    });
                            });
                        }
                    } else {
                        ui.label(egui::RichText::new("No shape selected.").italics().color(egui::Color32::GRAY));
                        ui.label("Click directly INSIDE any shape or click on the Objects List below.");
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // -------------------------------------------------
                    // 2. OBJECTS / SHAPES LIST (100% Reliable selection!)
                    // -------------------------------------------------
                    ui.heading("Objects List");
                    egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                        for (idx, shape) in self.doc.shapes.iter().enumerate() {
                            let is_sel = self.selected_shape_indices.contains(&idx);
                            let b = shape.bounds();
                            let lock_str = if shape.is_locked() { " [Locked]" } else { "" };
                            let desc = match shape {
                                Shape::Group { shapes, .. } => format!("#{}: Group ({} shapes) [C{:02}]{}", idx + 1, shapes.len(), shape.layer_index(), lock_str),
                                Shape::Path { .. } => format!("#{}: Path ({:.1}x{:.1}mm) [C{:02}]{}", idx + 1, b.width(), b.height(), shape.layer_index(), lock_str),
                                Shape::Rect { .. } => format!("#{}: Rect ({:.1}x{:.1}mm) [C{:02}]{}", idx + 1, b.width(), b.height(), shape.layer_index(), lock_str),
                                Shape::Circle { radius, .. } => format!("#{}: Circle (D={:.1}mm) [C{:02}]{}", idx + 1, radius * 2.0, shape.layer_index(), lock_str),
                                Shape::Image { .. } => format!("#{}: Image [C{:02}]{}", idx + 1, shape.layer_index(), lock_str),
                            };

                            let shift_held = ui.input(|i| i.modifiers.shift);
                            if ui.selectable_label(is_sel, desc).clicked() {
                                if shift_held {
                                    if is_sel {
                                        self.selected_shape_indices.retain(|&i| i != idx);
                                    } else {
                                        self.selected_shape_indices.push(idx);
                                    }
                                } else {
                                    self.selected_shape_indices = vec![idx];
                                }
                                if idx < self.doc.shapes.len() {
                                    self.selected_layer = self.doc.shapes[idx].layer_index();
                                }
                                self.status_msg = format!("Selected shape #{}", idx + 1);
                            }
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();

                    // -------------------------------------------------
                    // 3. CUT & LAYER SETTINGS
                    // -------------------------------------------------
                    ui.horizontal(|ui| {
                        ui.heading("Cut & Layer Settings");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("+ Add Layer").on_hover_text("Add a new color/cut layer").clicked() {
                                for idx in 0..32 {
                                    if !self.doc.layers.contains_key(&idx) {
                                        self.doc.get_or_create_layer(idx);
                                        self.selected_layer = idx;
                                        break;
                                    }
                                }
                            }
                        });
                    });
                    egui::ScrollArea::vertical().max_height(140.0).id_salt("layers_list_scroll").show(ui, |ui| {
                        for (idx, layer) in &mut self.doc.layers {
                            let is_selected = *idx == self.selected_layer;
                            let text = format!("[C{:02}] {}", idx, layer.name);

                            let color = egui::Color32::from_rgb(
                                u8::from_str_radix(&layer.color_hex[1..3], 16).unwrap_or(0),
                                u8::from_str_radix(&layer.color_hex[3..5], 16).unwrap_or(0),
                                u8::from_str_radix(&layer.color_hex[5..7], 16).unwrap_or(0),
                            );

                            ui.horizontal(|ui| {
                                let (rect, _response) = ui.allocate_exact_size(egui::Vec2::new(14.0, 14.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 2.0, color);

                                if ui.selectable_label(is_selected, text).clicked() {
                                    self.selected_layer = *idx;
                                }
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(format!("{:.0}mm/s | {:.0}%", layer.cut_setting.speed, layer.cut_setting.power_max));
                                });
                            });
                        }
                    });

                    // Controls for currently selected layer
                    if let Some(layer) = self.doc.layers.get_mut(&self.selected_layer) {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_label("Mode")
                                    .selected_text(format!("{:?}", layer.cut_setting.mode))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut layer.cut_setting.mode, CutMode::Line, "Line (Cut/Score)");
                                        ui.selectable_value(&mut layer.cut_setting.mode, CutMode::Fill, "Fill (Engrave)");
                                        ui.selectable_value(&mut layer.cut_setting.mode, CutMode::Image, "Image (Dither)");
                                    });
                                ui.checkbox(&mut layer.cut_setting.air_assist, "Air Assist");
                            });

                            ui.add_space(4.0);

                            egui::Grid::new("layer_cut_settings_grid")
                                .num_columns(3)
                                .spacing([6.0, 6.0])
                                .show(ui, |ui| {
                                    // 1. Speed (Numeric input + Slider)
                                    ui.label("Speed:");
                                    ui.add(
                                        egui::DragValue::new(&mut layer.cut_setting.speed)
                                            .speed(1.0)
                                            .range(1.0..=2000.0)
                                            .suffix(" mm/s"),
                                    ).on_hover_text("Click to type speed directly or drag");
                                    ui.add(
                                        egui::Slider::new(&mut layer.cut_setting.speed, 1.0..=600.0)
                                            .clamping(egui::SliderClamping::Never)
                                            .show_value(false),
                                    );
                                    ui.end_row();

                                    // 2. Max Power (Numeric input + Slider)
                                    ui.label("Power:");
                                    ui.add(
                                        egui::DragValue::new(&mut layer.cut_setting.power_max)
                                            .speed(0.5)
                                            .range(0.0..=100.0)
                                            .suffix(" %"),
                                    ).on_hover_text("Click to type power directly or drag");
                                    ui.add(
                                        egui::Slider::new(&mut layer.cut_setting.power_max, 0.0..=100.0)
                                            .show_value(false),
                                    );
                                    ui.end_row();

                                    // 3. Passes (Numeric input + Slider)
                                    ui.label("Passes:");
                                    ui.add(
                                        egui::DragValue::new(&mut layer.cut_setting.num_passes)
                                            .speed(1)
                                            .range(1..=100),
                                    ).on_hover_text("Click to type passes count directly or drag");
                                    ui.add(
                                        egui::Slider::new(&mut layer.cut_setting.num_passes, 1..=20)
                                            .clamping(egui::SliderClamping::Never)
                                            .show_value(false),
                                    );
                                    ui.end_row();
                                });
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // -------------------------------------------------
                    // 4. LASER JOGGING & CONTROL
                    // -------------------------------------------------
                    ui.horizontal(|ui| {
                        ui.heading("Laser Jogging");
                        if ui.button("Open Jog Window").on_hover_text("Open full laser control & jog window (Cmd+M)").clicked() {
                            self.show_move_window = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Step:");
                        ui.selectable_value(&mut self.jog_step, 1.0, "1mm");
                        ui.selectable_value(&mut self.jog_step, 10.0, "10mm");
                        ui.selectable_value(&mut self.jog_step, 50.0, "50mm");
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.keyboard_jog_laser, "Keyboard Arrow Jogging");
                    });

                    if self.device.is_physically_connected() || self.device.settings.enable_limit_switches {
                        ui.horizontal(|ui| {
                            ui.label("Switches:");
                            let x_active = self.device.pin_limit_x;
                            let y_active = self.device.pin_limit_y;

                            let x_color = if x_active { egui::Color32::from_rgb(255, 60, 60) } else { egui::Color32::from_rgb(130, 150, 170) };
                            let y_color = if y_active { egui::Color32::from_rgb(255, 60, 60) } else { egui::Color32::from_rgb(130, 150, 170) };

                            ui.label(egui::RichText::new(format!("X: {}", if x_active { "TRIGGERED" } else { "Open" })).color(x_color).strong());
                            ui.label(egui::RichText::new(format!("Y: {}", if y_active { "TRIGGERED" } else { "Open" })).color(y_color).strong());

                            if self.device.pin_limit_z {
                                ui.label(egui::RichText::new("Z: Triggered").color(egui::Color32::from_rgb(255, 120, 0)).strong());
                            }

                            if ui.small_button("Sync ($10=19, $21=1)").on_hover_text("Send $10=19 (real-time pin reporting) and $21=1 (hard limits protection) to GRBL").clicked() {
                                self.device.send_raw("$10=19");
                                self.device.send_raw("$21=1");
                                self.device.send_realtime(b'?');
                                self.status_msg = "Sent $10=19 and $21=1 to GRBL".to_string();
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Home Speed:");
                            if ui.add(egui::Slider::new(&mut self.device.settings.homing_speed, 100.0..=1200.0).suffix(" mm/min")).on_hover_text("Gentle seek speed for mechanical switch calibration (default 400 mm/min = 6.6 mm/s)").changed() {
                                let _ = self.device.settings.save();
                            }
                        });
                    }

                    if self.device.grbl_state.starts_with("Alarm") {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("Status: {}", self.device.grbl_state)).color(egui::Color32::from_rgb(255, 60, 60)).strong());
                            if ui.button("Unlock ($X)").clicked() {
                                self.device.send_raw("$X");
                                self.device.send_raw("$21=1");
                            }
                        });
                    }

                    let step = self.jog_step;
                    let speed = self.jog_speed;

                    let btn_size = egui::vec2(60.0, 26.0);
                    let grid_w = 3.0 * btn_size.x + 2.0 * 4.0;
                    let pad = ((ui.available_width() - grid_w) / 2.0).max(0.0);

                    ui.horizontal(|ui| {
                        ui.add_space(pad);
                        egui::Grid::new("jog_side_grid")
                            .spacing([4.0, 4.0])
                            .min_col_width(btn_size.x)
                            .show(ui, |ui| {
                                // Row 1: NW, Y+, NE
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X-Y+").size(11.0))).on_hover_text("X- / Y+").clicked() {
                                    self.device.jog_arrow(-1.0, 1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("Y+").strong().size(14.0))).on_hover_text("Jog Y+").clicked() {
                                    self.device.jog_arrow(0.0, 1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X+Y+").size(11.0))).on_hover_text("X+ / Y+").clicked() {
                                    self.device.jog_arrow(1.0, 1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                ui.end_row();

                                // Row 2: X-, 0,0, X+
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X-").strong().size(14.0))).on_hover_text("Jog X-").clicked() {
                                    self.device.jog_arrow(-1.0, 0.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("0,0").strong().size(14.0))).on_hover_text("Move to origin (X0 Y0)").clicked() {
                                    self.device.go_to_origin();
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X+").strong().size(14.0))).on_hover_text("Jog X+").clicked() {
                                    self.device.jog_arrow(1.0, 0.0, step, speed);
                                    ctx.request_repaint();
                                }
                                ui.end_row();

                                // Row 3: SW, Y-, SE
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X-Y-").size(11.0))).on_hover_text("X- / Y-").clicked() {
                                    self.device.jog_arrow(-1.0, -1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("Y-").strong().size(14.0))).on_hover_text("Jog Y-").clicked() {
                                    self.device.jog_arrow(0.0, -1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X+Y-").size(11.0))).on_hover_text("X+ / Y-").clicked() {
                                    self.device.jog_arrow(1.0, -1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                ui.end_row();
                            });
                    });

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Go to Corner:").strong());

                    let corner_btn_w = (ui.available_width() - 4.0) * 0.5;
                    let corner_btn_size = egui::vec2(corner_btn_w, 24.0);

                    ui.horizontal(|ui| {
                        let (tl_x, tl_y) = self.device.get_corner_coordinates("top_left");
                        let tl_text = format!("Top-Left ({:.0},{:.0})", tl_x, tl_y);
                        if ui.add_sized(corner_btn_size, egui::Button::new(tl_text))
                            .on_hover_text(format!("Move laser head directly to top-left corner (X={:.1}, Y={:.1} mm)", tl_x, tl_y))
                            .clicked()
                        {
                            self.device.go_to_corner("top_left");
                            ctx.request_repaint();
                        }

                        let (tr_x, tr_y) = self.device.get_corner_coordinates("top_right");
                        let tr_text = format!("Top-Right ({:.0},{:.0})", tr_x, tr_y);
                        if ui.add_sized(corner_btn_size, egui::Button::new(tr_text))
                            .on_hover_text(format!("Move laser head directly to top-right corner (X={:.1}, Y={:.1} mm)", tr_x, tr_y))
                            .clicked()
                        {
                            self.device.go_to_corner("top_right");
                            ctx.request_repaint();
                        }
                    });

                    ui.horizontal(|ui| {
                        let (bl_x, bl_y) = self.device.get_corner_coordinates("bottom_left");
                        let bl_text = format!("Bottom-Left ({:.0},{:.0})", bl_x, bl_y);
                        if ui.add_sized(corner_btn_size, egui::Button::new(bl_text))
                            .on_hover_text(format!("Move laser head directly to bottom-left corner (X={:.1}, Y={:.1} mm)", bl_x, bl_y))
                            .clicked()
                        {
                            self.device.go_to_corner("bottom_left");
                            ctx.request_repaint();
                        }

                        let (br_x, br_y) = self.device.get_corner_coordinates("bottom_right");
                        let br_text = format!("Bottom-Right ({:.0},{:.0})", br_x, br_y);
                        if ui.add_sized(corner_btn_size, egui::Button::new(br_text))
                            .on_hover_text(format!("Move laser head directly to bottom-right corner (X={:.1}, Y={:.1} mm)", br_x, br_y))
                            .clicked()
                        {
                            self.device.go_to_corner("bottom_right");
                            ctx.request_repaint();
                        }
                    });

                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        let set_origin_btn = egui::Button::new(
                            egui::RichText::new("Set Origin (0,0)").strong().color(egui::Color32::WHITE)
                        ).fill(egui::Color32::from_rgb(46, 125, 50)); // Forest green

                        let has_switches = self.device.settings.enable_limit_switches;

                        if has_switches {
                            let is_homing = self.device.homing_phase.is_some();
                            let half_w = (ui.available_width() - 4.0) * 0.55;

                            if ui.add_sized([half_w, 26.0], set_origin_btn)
                                .on_hover_text("Set current laser head position as (0,0) Work Origin (G92 X0 Y0)")
                                .clicked()
                            {
                                self.device.set_origin_here();
                            }

                            let reset_btn = egui::Button::new(
                                egui::RichText::new(if is_homing { "Abort" } else { "Switches" })
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            ).fill(if is_homing { egui::Color32::from_rgb(220, 50, 40) } else { egui::Color32::from_rgb(33, 115, 185) });

                            if ui.add_sized([ui.available_width(), 26.0], reset_btn)
                                .on_hover_text(if is_homing { "Click to immediately abort homing move" } else { "Move head to physical limit switches (requires working switches)" })
                                .clicked()
                            {
                                if is_homing {
                                    self.device.stop_job();
                                } else {
                                    self.device.start_mechanical_home_reset();
                                }
                            }
                        } else {
                            if ui.add_sized([ui.available_width(), 26.0], set_origin_btn)
                                .on_hover_text("Set current laser head position as (0,0) Work Origin (G92 X0 Y0)")
                                .clicked()
                            {
                                self.device.set_origin_here();
                            }
                        }
                    });
                    ui.add_space(3.0);

                    ui.horizontal(|ui| {
                        let is_on = self.device.is_weak_light_on;
                        let weak_btn = egui::Button::new(
                            egui::RichText::new(if is_on { "Beam ON" } else { "Weak Beam" })
                                .color(if is_on { egui::Color32::WHITE } else { egui::Color32::from_rgb(200, 230, 255) }),
                        ).fill(if is_on { egui::Color32::from_rgb(255, 69, 0) } else { egui::Color32::from_rgb(40, 70, 110) });

                        if ui.add(weak_btn).clicked() {
                            self.device.toggle_weak_light();
                        }

                        if ui.button("Box Frame").clicked() {
                            self.start_box_frame();
                        }

                        if ui.button("Contour").clicked() {
                            self.start_contour_frame();
                        }
                    });

                    ui.add_space(2.0);
                    let is_capturing = self.oneshot_snapshot.is_some();
                    let snap_btn = egui::Button::new(
                        egui::RichText::new(if is_capturing { "Capturing Bed Photo..." } else { "Update Bed Overlay (Camera)" })
                            .strong()
                            .color(if is_capturing { egui::Color32::from_rgb(255, 230, 80) } else { egui::Color32::WHITE })
                    ).fill(if is_capturing { egui::Color32::from_rgb(180, 100, 20) } else { egui::Color32::from_rgb(33, 115, 185) });

                    if ui.add_sized([ui.available_width(), 25.0], snap_btn)
                        .on_hover_text("Capture a fresh photo of the bed with your material (Cmd+U)")
                        .clicked()
                    {
                        self.trigger_update_overlay();
                    }

                    ui.add_space(3.0);
                    ui.collapsing("Custom Soft Limits (Corners)", |ui| {
                        if ui.checkbox(&mut self.device.settings.custom_boundaries.enabled, "Enforce learned soft limits").changed() {
                            let _ = self.device.settings.save();
                        }
                        ui.label("Jog laser to corner and save:");
                        ui.horizontal_wrapped(|ui| {
                            let bl_label = match self.device.settings.custom_boundaries.corner_bottom_left {
                                Some((x, y)) => format!("[Set] Bottom-Left ({:.0},{:.0})", x, y),
                                None => "Bottom-Left".to_string(),
                            };
                            if ui.button(bl_label).on_hover_text("Save current position as bottom-left corner").clicked() {
                                self.device.save_boundary_corner("bottom_left");
                            }

                            let br_label = match self.device.settings.custom_boundaries.corner_bottom_right {
                                Some((x, y)) => format!("[Set] Bottom-Right ({:.0},{:.0})", x, y),
                                None => "Bottom-Right".to_string(),
                            };
                            if ui.button(br_label).on_hover_text("Save current position as bottom-right corner").clicked() {
                                self.device.save_boundary_corner("bottom_right");
                            }
                        });

                        ui.horizontal_wrapped(|ui| {
                            let tl_label = match self.device.settings.custom_boundaries.corner_top_left {
                                Some((x, y)) => format!("[Set] Top-Left ({:.0},{:.0})", x, y),
                                None => "Top-Left".to_string(),
                            };
                            if ui.button(tl_label).on_hover_text("Save current position as top-left corner").clicked() {
                                self.device.save_boundary_corner("top_left");
                            }

                            let tr_label = match self.device.settings.custom_boundaries.corner_top_right {
                                Some((x, y)) => format!("[Set] Top-Right ({:.0},{:.0})", x, y),
                                None => "Top-Right".to_string(),
                            };
                            if ui.button(tr_label).on_hover_text("Save current position as top-right corner").clicked() {
                                self.device.save_boundary_corner("top_right");
                            }
                        });

                        if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
                            self.device.settings.custom_boundaries.min_x,
                            self.device.settings.custom_boundaries.max_x,
                            self.device.settings.custom_boundaries.min_y,
                            self.device.settings.custom_boundaries.max_y,
                        ) {
                            let b_color = if self.device.settings.custom_boundaries.enabled {
                                egui::Color32::from_rgb(0, 220, 100)
                            } else {
                                egui::Color32::GRAY
                            };
                            ui.label(egui::RichText::new(format!(
                                "Limits: X: {:.1}..{:.1}, Y: {:.1}..{:.1} mm ({:.0}x{:.0})",
                                min_x, max_x, min_y, max_y, (max_x - min_x).abs(), (max_y - min_y).abs()
                            )).color(b_color).strong());
                        }

                        if ui.button("Clear Custom Limits").clicked() {
                            self.device.clear_boundary_corners();
                        }
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.heading("Job Execution (G-Code)");

                    if !self.device.is_job_running {
                        let start_btn = egui::Button::new(
                            egui::RichText::new(" START JOB (CUT) ").strong().color(egui::Color32::WHITE),
                        ).fill(egui::Color32::from_rgb(34, 139, 34));
                        if ui.add_sized([ui.available_width(), 32.0], start_btn).on_hover_text("Optimize and run cut job with laser").clicked() {
                            self.start_job();
                        }
                    } else {
                        ui.horizontal(|ui| {
                            if self.device.is_job_paused {
                                let resume_btn = egui::Button::new(
                                    egui::RichText::new(" RESUME ").strong().color(egui::Color32::WHITE),
                                ).fill(egui::Color32::from_rgb(0, 140, 255));
                                if ui.add_sized([ui.available_width() * 0.5 - 4.0, 30.0], resume_btn).clicked() {
                                    self.device.resume_job();
                                }
                            } else {
                                let pause_btn = egui::Button::new(
                                    egui::RichText::new(" PAUSE ").strong().color(egui::Color32::BLACK),
                                ).fill(egui::Color32::from_rgb(255, 193, 7));
                                if ui.add_sized([ui.available_width() * 0.5 - 4.0, 30.0], pause_btn).clicked() {
                                    self.device.pause_job();
                                }
                            }

                            let stop_btn = egui::Button::new(
                                egui::RichText::new(" STOP ").strong().color(egui::Color32::WHITE),
                            ).fill(egui::Color32::RED);
                            if ui.add_sized([ui.available_width(), 30.0], stop_btn).clicked() {
                                self.device.stop_job();
                            }
                        });

                        let pct = self.device.job_progress;
                        ui.add(egui::ProgressBar::new(pct).text(format!("{:.1}% ({}/{})", pct * 100.0, self.device.job_idx, self.device.job_total_lines)));
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.heading("Simulation Stats");

                    let opt = CutOptimizer::new();
                    let job = opt.optimize(&self.doc);
                    let sim = Simulator::new();
                    let stats = sim.simulate(&self.doc, &job);

                    ui.label(format!("Cut Distance: {:.1} mm", stats.total_cut_distance_mm));
                    ui.label(format!("Rapid Distance: {:.1} mm", stats.total_rapid_distance_mm));
                    ui.label(egui::RichText::new(format!("Est. Time: {:.1} s ({:.2} min)", stats.total_estimated_time_sec, stats.total_estimated_time_sec / 60.0)).strong());
                });
            });

        // -------------------------------------------------------------
        // CENTER 2D CAD CANVAS (Full interactive moving, sizing, drawing)
        // -------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
            // Pan canvas with right mouse or middle mouse
            if response.dragged_by(egui::PointerButton::Secondary) || response.dragged_by(egui::PointerButton::Middle) {
                self.pan += response.drag_delta();
            }

            // Zoom with wheel ONLY when mouse is hovering over the canvas (not over sidebars/toolbars/dialogs)
            if response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll.abs() > 0.0 {
                    let old_zoom = self.zoom;
                    let factor = 1.0 + scroll * 0.002;
                    let new_zoom = (self.zoom * factor).clamp(0.2, 20.0);

                    let canvas_origin_current = response.rect.min + self.pan;
                    if let Some(mouse_pos) = response.hover_pos() {
                        let pointer_offset = mouse_pos - canvas_origin_current;
                        self.pan -= pointer_offset * (new_zoom / old_zoom - 1.0);
                    }
                    self.zoom = new_zoom;
                }
            }

            let canvas_origin = response.rect.min + self.pan;

            // Laser bed outline
            let bed_w = self.doc.width as f32 * self.zoom;
            let bed_h = self.doc.height as f32 * self.zoom;
            let bed_rect = egui::Rect::from_min_size(canvas_origin, egui::Vec2::new(bed_w, bed_h));

            painter.rect_filled(bed_rect, 0.0, egui::Color32::from_rgb(250, 250, 250));

            // Draw live camera background overlay strictly clipped to the laser bed grid
            if self.camera_calibration.show_on_canvas {
                if let Some(ref texture) = self.camera_texture {
                    let bed_painter = painter.with_clip_rect(bed_rect);
                    self.paint_camera_on_bed(&bed_painter, canvas_origin, texture);
                }
            }

            painter.rect_stroke(bed_rect, 0.0, egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(180, 180, 180)), egui::StrokeKind::Outside);

            // Grid lines radiating from the machine origin corner (so 0,0 is always a clean grid corner)
            let is_origin_right = self.device.settings.origin_corner == crate::device::OriginCorner::TopRight
                || self.device.settings.origin_corner == crate::device::OriginCorner::BottomRight;
            let is_origin_bottom = self.device.settings.origin_corner == crate::device::OriginCorner::BottomLeft
                || self.device.settings.origin_corner == crate::device::OriginCorner::BottomRight;

            // Adaptive grid subdivisions and millimeter ruler labels based on zoom level:
            // (minor_step_mm, major_step_mm, label_step_mm)
            let (minor_step, major_step, label_step) = Self::grid_steps_for_zoom(self.zoom);

            let minor_tenths = ((minor_step * 10.0).round() as i64).max(1);
            let major_tenths = ((major_step * 10.0).round() as i64).max(1);
            let label_tenths = ((label_step * 10.0).round() as i64).max(1);
            let doc_w_tenths = (self.doc.width * 10.0).round() as i64;
            let doc_h_tenths = (self.doc.height * 10.0).round() as i64;
            let canvas_rect = response.rect;

            // X grid lines radiating from origin corner outwards
            let mut dist_tenths_x = 0;
            while dist_tenths_x <= doc_w_tenths {
                let dist_x = dist_tenths_x as f64 / 10.0;
                let doc_x = if is_origin_right {
                    self.doc.width - dist_x
                } else {
                    dist_x
                };
                let p1 = self.doc_to_screen(Point2D::new(doc_x, 0.0), canvas_origin);
                let p2 = self.doc_to_screen(Point2D::new(doc_x, self.doc.height), canvas_origin);

                // Only render if line is within or near the visible canvas viewport horizontally
                if p1.x >= canvas_rect.left() - 20.0 && p1.x <= canvas_rect.right() + 20.0 {
                    let is_major = dist_tenths_x % major_tenths == 0;
                    let is_label = dist_tenths_x % label_tenths == 0;

                    let stroke_color = if is_major {
                        egui::Color32::from_rgb(205, 210, 220)
                    } else {
                        egui::Color32::from_rgb(238, 240, 243)
                    };
                    let stroke_w = if is_major { 0.8_f32 } else { 0.5_f32 };
                    painter.line_segment([p1, p2], egui::Stroke::new(stroke_w, stroke_color));

                    // Millimeter distance label from origin
                    if is_label && dist_tenths_x > 0 {
                        let base_y = if is_origin_bottom {
                            p2.y + 10.0
                        } else {
                            p1.y - 10.0
                        };

                        // Keep label visible on screen even when zoomed into bed
                        let text_y = if is_origin_bottom {
                            if p2.y > canvas_rect.bottom() && p1.y < canvas_rect.bottom() {
                                canvas_rect.bottom() - 10.0
                            } else {
                                base_y
                            }
                        } else {
                            if p1.y < canvas_rect.top() && p2.y > canvas_rect.top() {
                                canvas_rect.top() + 10.0
                            } else {
                                base_y
                            }
                        };

                        let label_str = if dist_tenths_x % 10 == 0 {
                            format!("{}", dist_tenths_x / 10)
                        } else {
                            format!("{:.1}", dist_x)
                        };

                        let text_pos = egui::pos2(p1.x, text_y);
                        let pill_w = (label_str.len() as f32) * 6.5 + 6.0;
                        let pill_rect = egui::Rect::from_center_size(text_pos, egui::vec2(pill_w, 13.0));
                        painter.rect_filled(pill_rect, 2.0, egui::Color32::from_rgba_unmultiplied(250, 250, 250, 210));
                        painter.text(
                            text_pos,
                            egui::Align2::CENTER_CENTER,
                            &label_str,
                            egui::FontId::proportional(9.5),
                            egui::Color32::from_rgb(120, 125, 135),
                        );
                    }
                }

                dist_tenths_x += minor_tenths;
            }

            // Y grid lines radiating from origin corner outwards
            let mut dist_tenths_y = 0;
            while dist_tenths_y <= doc_h_tenths {
                let dist_y = dist_tenths_y as f64 / 10.0;
                let doc_y = if is_origin_bottom {
                    self.doc.height - dist_y
                } else {
                    dist_y
                };
                let p1 = self.doc_to_screen(Point2D::new(0.0, doc_y), canvas_origin);
                let p2 = self.doc_to_screen(Point2D::new(self.doc.width, doc_y), canvas_origin);

                // Only render if line is within or near the visible canvas viewport vertically
                if p1.y >= canvas_rect.top() - 20.0 && p1.y <= canvas_rect.bottom() + 20.0 {
                    let is_major = dist_tenths_y % major_tenths == 0;
                    let is_label = dist_tenths_y % label_tenths == 0;

                    let stroke_color = if is_major {
                        egui::Color32::from_rgb(205, 210, 220)
                    } else {
                        egui::Color32::from_rgb(238, 240, 243)
                    };
                    let stroke_w = if is_major { 0.8_f32 } else { 0.5_f32 };
                    painter.line_segment([p1, p2], egui::Stroke::new(stroke_w, stroke_color));

                    // Millimeter distance label from origin
                    if is_label && dist_tenths_y > 0 {
                        let base_x = if is_origin_right {
                            p2.x + 14.0
                        } else {
                            p1.x - 14.0
                        };

                        // Keep label visible on screen even when zoomed into bed
                        let text_x = if is_origin_right {
                            if p2.x > canvas_rect.right() && p1.x < canvas_rect.right() {
                                canvas_rect.right() - 14.0
                            } else {
                                base_x
                            }
                        } else {
                            if p1.x < canvas_rect.left() && p2.x > canvas_rect.left() {
                                canvas_rect.left() + 14.0
                            } else {
                                base_x
                            }
                        };

                        let label_str = if dist_tenths_y % 10 == 0 {
                            format!("{}", dist_tenths_y / 10)
                        } else {
                            format!("{:.1}", dist_y)
                        };

                        let text_pos = egui::pos2(text_x, p1.y);
                        let pill_w = (label_str.len() as f32) * 6.5 + 6.0;
                        let pill_rect = egui::Rect::from_center_size(text_pos, egui::vec2(pill_w, 13.0));
                        painter.rect_filled(pill_rect, 2.0, egui::Color32::from_rgba_unmultiplied(250, 250, 250, 210));
                        painter.text(
                            text_pos,
                            egui::Align2::CENTER_CENTER,
                            &label_str,
                            egui::FontId::proportional(9.5),
                            egui::Color32::from_rgb(120, 125, 135),
                        );
                    }
                }

                dist_tenths_y += minor_tenths;
            }

            // Origin marker (0, 0) - Respects origin_corner (TopRight, TopLeft, etc.)
            let origin_pos = self.origin_screen_pos(canvas_origin);
            painter.circle_filled(origin_pos, 5.0, egui::Color32::RED);
            painter.circle_stroke(origin_pos, 8.0, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 120, 120)));

            let is_right = self.device.settings.origin_corner == crate::device::OriginCorner::TopRight
                || self.device.settings.origin_corner == crate::device::OriginCorner::BottomRight;
            let is_bot = self.device.settings.origin_corner == crate::device::OriginCorner::BottomLeft
                || self.device.settings.origin_corner == crate::device::OriginCorner::BottomRight;

            painter.text(
                origin_pos + egui::vec2(if is_right { -12.0 } else { 12.0 }, if is_bot { -8.0 } else { 8.0 }),
                if is_right { egui::Align2::RIGHT_CENTER } else { egui::Align2::LEFT_CENTER },
                "Origin (0, 0)",
                egui::FontId::proportional(11.0),
                egui::Color32::RED,
            );

            // Visual learned safety boundary envelope (anchored to configured machine origin 0,0)
            if self.device.settings.custom_boundaries.enabled {
                if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
                    self.device.settings.custom_boundaries.min_x,
                    self.device.settings.custom_boundaries.max_x,
                    self.device.settings.custom_boundaries.min_y,
                    self.device.settings.custom_boundaries.max_y,
                ) {
                    let p_min = self.machine_to_screen(Point2D::new(min_x, min_y), canvas_origin);
                    let p_max = self.machine_to_screen(Point2D::new(max_x, max_y), canvas_origin);
                    let bound_rect = egui::Rect::from_two_pos(p_min, p_max);

                    // Clear boundary outline (interior left completely transparent so camera feed & materials remain crystal clear)
                    painter.rect_stroke(
                        bound_rect,
                        0.0,
                        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(46, 125, 50)),
                        egui::StrokeKind::Outside,
                    );

                    // Draw target markers for individual taught corners
                    let taught_corners = [
                        ("Top-Left", self.device.settings.custom_boundaries.corner_top_left),
                        ("Top-Right", self.device.settings.custom_boundaries.corner_top_right),
                        ("Bottom-Left", self.device.settings.custom_boundaries.corner_bottom_left),
                        ("Bottom-Right", self.device.settings.custom_boundaries.corner_bottom_right),
                    ];
                    for (_label, corner_opt) in &taught_corners {
                        if let Some((cx, cy)) = *corner_opt {
                            let c_screen = self.machine_to_screen(Point2D::new(cx, cy), canvas_origin);
                            painter.circle_filled(c_screen, 3.5, egui::Color32::from_rgb(46, 125, 50));
                            painter.circle_stroke(
                                c_screen,
                                5.5,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(120, 200, 130)),
                            );
                        }
                    }

                    // Position label away from the origin corner to prevent overlap
                    let (label_pos, align) = match self.device.settings.origin_corner {
                        crate::device::OriginCorner::TopLeft => (bound_rect.right_top() + egui::vec2(-8.0, 8.0), egui::Align2::RIGHT_TOP),
                        crate::device::OriginCorner::TopRight => (bound_rect.left_top() + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP),
                        crate::device::OriginCorner::BottomLeft => (bound_rect.right_bottom() + egui::vec2(-8.0, -8.0), egui::Align2::RIGHT_BOTTOM),
                        crate::device::OriginCorner::BottomRight => (bound_rect.left_bottom() + egui::vec2(8.0, -8.0), egui::Align2::LEFT_BOTTOM),
                    };
                    painter.text(
                        label_pos,
                        align,
                        "Safe Work Envelope (Soft Limits)",
                        egui::FontId::proportional(11.0),
                        egui::Color32::from_rgb(46, 125, 50),
                    );
                }
            }

            // ---------------------------------------------------------
            // MOUSE INTERACTION: CLICK INSIDE, DRAG-MOVE, DRAG-RESIZE
            // ---------------------------------------------------------
            let pointer_pos = response.interact_pointer_pos().or_else(|| response.hover_pos());
            if let Some(mouse_pos) = pointer_pos {
                let doc_pt = self.screen_to_doc(mouse_pos, canvas_origin);

                let handle_radius_doc = 10.0 / self.zoom as f64;
                let hit_tolerance_doc = (10.0 / self.zoom as f64).max(3.0);

                let shift_held = ui.input(|i| i.modifiers.shift);

                // Cursor management
                if self.active_tool == ActiveTool::Select {
                    if let Some(bounds) = self.selection_bounds() {
                        let handle_hit = self.test_handles(&doc_pt, &bounds, handle_radius_doc);
                        match handle_hit {
                            DragHandle::TopLeft | DragHandle::BottomRight => ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe),
                            DragHandle::TopRight | DragHandle::BottomLeft => ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw),
                            DragHandle::Left | DragHandle::Right => ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal),
                            DragHandle::Top | DragHandle::Bottom => ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical),
                            DragHandle::None | DragHandle::Move => {
                                if self.drag_handle == DragHandle::Move {
                                    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                                } else if self.find_best_shape_at_point(&doc_pt, hit_tolerance_doc).is_some() {
                                    ctx.set_cursor_icon(egui::CursorIcon::Grab);
                                }
                            }
                        }
                    } else if self.find_best_shape_at_point(&doc_pt, hit_tolerance_doc).is_some() {
                        ctx.set_cursor_icon(egui::CursorIcon::Grab);
                    }
                }

                // 1. Mouse Click (Clean single click without dragging)
                if response.clicked_by(egui::PointerButton::Primary) {
                    if self.active_tool == ActiveTool::DrawText {
                        self.text_pos_x = doc_pt.x.clamp(0.0, self.doc.width);
                        self.text_pos_y = doc_pt.y.clamp(0.0, self.doc.height);
                        self.show_text_dialog = true;
                        self.active_tool = ActiveTool::Select;
                    } else if self.active_tool == ActiveTool::Select {
                        if let Some(best_idx) = self.find_best_shape_at_point(&doc_pt, hit_tolerance_doc) {
                            if shift_held {
                                if self.selected_shape_indices.contains(&best_idx) {
                                    self.selected_shape_indices.retain(|&i| i != best_idx);
                                } else {
                                    self.selected_shape_indices.push(best_idx);
                                }
                            } else {
                                self.selected_shape_indices = vec![best_idx];
                            }
                            self.status_msg = format!("Selected shape #{} - Drag or use Arrow Keys to move", best_idx + 1);
                        } else if !shift_held {
                            self.selected_shape_indices.clear();
                        }
                    }
                }
                // 2. Mouse Drag Start
                else if response.drag_started_by(egui::PointerButton::Primary) {
                    if self.active_tool == ActiveTool::DrawText {
                        self.text_pos_x = doc_pt.x.clamp(0.0, self.doc.width);
                        self.text_pos_y = doc_pt.y.clamp(0.0, self.doc.height);
                        self.show_text_dialog = true;
                        self.active_tool = ActiveTool::Select;
                    } else if self.active_tool == ActiveTool::Select {
                        // First check if clicking on resize handle of existing selection
                        let mut handle_hit = DragHandle::None;
                        if let Some(bounds) = self.selection_bounds() {
                            handle_hit = self.test_handles(&doc_pt, &bounds, handle_radius_doc);
                        }

                        if handle_hit != DragHandle::None {
                            self.drag_handle = handle_hit;
                            self.drag_start_doc = Some(doc_pt);
                            self.last_drag_doc = Some(doc_pt);
                        } else {
                            // Check if drag started on an already selected shape
                            let hit_already_selected = self.selected_shape_indices.iter().any(|&idx| {
                                idx < self.doc.shapes.len() && self.doc.shapes[idx].hit_test(&doc_pt, hit_tolerance_doc)
                            });

                            if hit_already_selected {
                                self.drag_handle = DragHandle::Move;
                                self.drag_start_doc = Some(doc_pt);
                                self.last_drag_doc = Some(doc_pt);
                            } else if let Some(best_idx) = self.find_best_shape_at_point(&doc_pt, hit_tolerance_doc) {
                                if shift_held {
                                    if !self.selected_shape_indices.contains(&best_idx) {
                                        self.selected_shape_indices.push(best_idx);
                                    }
                                } else {
                                    self.selected_shape_indices = vec![best_idx];
                                }
                                if best_idx < self.doc.shapes.len() {
                                    self.selected_layer = self.doc.shapes[best_idx].layer_index();
                                }
                                self.drag_handle = DragHandle::Move;
                                self.drag_start_doc = Some(doc_pt);
                                self.last_drag_doc = Some(doc_pt);
                                self.status_msg = format!("Selected shape #{} - Drag or use Arrow Keys to move", best_idx + 1);
                            } else {
                                // Clicked on empty space: start rubberband marquee box select or deselect
                                if !shift_held {
                                    self.selected_shape_indices.clear();
                                }
                                self.is_box_selecting = true;
                                self.box_select_start = Some(doc_pt);
                                self.box_select_current = Some(doc_pt);
                                self.drag_handle = DragHandle::None;
                            }
                        }
                    } else {
                        // Drawing new shape mode
                        self.is_drawing = true;
                        self.draw_start = Some(doc_pt);
                        self.draw_current = Some(doc_pt);
                    }
                }
                // 2. Mouse Dragging
                else if response.dragged_by(egui::PointerButton::Primary) {
                    if self.active_tool == ActiveTool::Select {
                        if self.is_box_selecting {
                            self.box_select_current = Some(doc_pt);
                        } else if !self.selected_shape_indices.is_empty() {
                            if let Some(last_pt) = self.last_drag_doc {
                                let delta_x = doc_pt.x - last_pt.x;
                                let delta_y = doc_pt.y - last_pt.y;

                                match self.drag_handle {
                                    DragHandle::Move => {
                                        for &idx in &self.selected_shape_indices {
                                            if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                                self.doc.shapes[idx].translate(delta_x, delta_y);
                                            }
                                        }
                                        if let Some(b) = self.selection_bounds() {
                                            self.status_msg = format!("Moving selection to X: {:.1}, Y: {:.1} mm", b.min_x, b.min_y);
                                        }
                                    }
                                    DragHandle::Right => {
                                        if let Some(b) = self.selection_bounds() {
                                            let new_w = (doc_pt.x - b.min_x).max(2.0);
                                            let sx = new_w / b.width().max(1e-4);
                                            for &idx in &self.selected_shape_indices {
                                                if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                                    self.doc.shapes[idx].scale(Point2D::new(b.min_x, b.min_y), sx, 1.0);
                                                }
                                            }
                                        }
                                    }
                                    DragHandle::Bottom => {
                                        if let Some(b) = self.selection_bounds() {
                                            let new_h = (doc_pt.y - b.min_y).max(2.0);
                                            let sy = new_h / b.height().max(1e-4);
                                            for &idx in &self.selected_shape_indices {
                                                if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                                    self.doc.shapes[idx].scale(Point2D::new(b.min_x, b.min_y), 1.0, sy);
                                                }
                                            }
                                        }
                                    }
                                    DragHandle::BottomRight => {
                                        if let Some(b) = self.selection_bounds() {
                                            let new_w = (doc_pt.x - b.min_x).max(2.0);
                                            let new_h = if self.lock_aspect_ratio && b.width() > 1e-4 {
                                                new_w * (b.height() / b.width())
                                            } else {
                                                (doc_pt.y - b.min_y).max(2.0)
                                            };
                                            let sx = new_w / b.width().max(1e-4);
                                            let sy = new_h / b.height().max(1e-4);
                                            for &idx in &self.selected_shape_indices {
                                                if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                                    self.doc.shapes[idx].scale(Point2D::new(b.min_x, b.min_y), sx, sy);
                                                }
                                            }
                                        }
                                    }
                                    DragHandle::TopLeft => {
                                        if let Some(b) = self.selection_bounds() {
                                            let new_w = (b.max_x - doc_pt.x).max(2.0);
                                            let new_h = if self.lock_aspect_ratio && b.width() > 1e-4 {
                                                new_w * (b.height() / b.width())
                                            } else {
                                                (b.max_y - doc_pt.y).max(2.0)
                                            };
                                            let sx = new_w / b.width().max(1e-4);
                                            let sy = new_h / b.height().max(1e-4);
                                            for &idx in &self.selected_shape_indices {
                                                if idx < self.doc.shapes.len() && !self.doc.shapes[idx].is_locked() {
                                                    self.doc.shapes[idx].scale(Point2D::new(b.max_x, b.max_y), sx, sy);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            self.last_drag_doc = Some(doc_pt);
                        }
                    } else if self.is_drawing {
                        self.draw_current = Some(doc_pt);
                    }
                }
                // 3. Mouse Drag Stopped
                else if response.drag_stopped() {
                    self.drag_handle = DragHandle::None;
                    self.drag_start_doc = None;
                    self.last_drag_doc = None;

                    // Finalize rubberband box selection
                    if self.is_box_selecting {
                        if let (Some(start), Some(curr)) = (self.box_select_start, self.box_select_current) {
                            let select_box = BoundingBox::new(
                                start.x.min(curr.x),
                                start.y.min(curr.y),
                                start.x.max(curr.x),
                                start.y.max(curr.y),
                            );

                            if select_box.width() > 2.0 && select_box.height() > 2.0 {
                                for (idx, shape) in self.doc.shapes.iter().enumerate() {
                                    let b = shape.bounds();
                                    if select_box.contains_point(&b.center()) || (b.min_x >= select_box.min_x && b.max_x <= select_box.max_x && b.min_y >= select_box.min_y && b.max_y <= select_box.max_y) {
                                        if !self.selected_shape_indices.contains(&idx) {
                                            self.selected_shape_indices.push(idx);
                                        }
                                    }
                                }
                            }
                        }
                        self.is_box_selecting = false;
                        self.box_select_start = None;
                        self.box_select_current = None;
                    }

                    // Finalize new shape drawing
                    if self.is_drawing {
                        if let (Some(start), Some(curr)) = (self.draw_start, self.draw_current) {
                            let w = (curr.x - start.x).abs();
                            let h = (curr.y - start.y).abs();
                            let min_x = start.x.min(curr.x);
                            let min_y = start.y.min(curr.y);

                            match self.active_tool {
                                ActiveTool::DrawRect => {
                                    if w > 2.0 && h > 2.0 {
                                        self.doc.add_shape(Shape::Rect {
                                            layer_index: self.selected_layer,
                                            x: min_x,
                                            y: min_y,
                                            width: w,
                                            height: h,
                                            corner_radius: 0.0,
                                        });
                                        self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                                        self.status_msg = format!("Created Rect {:.1}x{:.1}mm", w, h);
                                    }
                                }
                                ActiveTool::DrawCircle => {
                                    let r = (w.powi(2) + h.powi(2)).sqrt() * 0.5;
                                    if r > 2.0 {
                                        self.doc.add_shape(Shape::Circle {
                                            layer_index: self.selected_layer,
                                            center: Point2D::new((start.x + curr.x) * 0.5, (start.y + curr.y) * 0.5),
                                            radius: r,
                                        });
                                        self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                                        self.status_msg = format!("Created Circle D={:.1}mm", r * 2.0);
                                    }
                                }
                                ActiveTool::DrawLine => {
                                    if start.distance_to(&curr) > 2.0 {
                                        self.doc.add_shape(Shape::Path {
                                            layer_index: self.selected_layer,
                                            polyline: Polyline::new(vec![start, curr], false),
                                        });
                                        self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                                        self.status_msg = "Created Line".to_string();
                                    }
                                }
                                _ => {}
                            }
                        }
                        self.is_drawing = false;
                        self.draw_start = None;
                        self.draw_current = None;
                    }
                }
            }

            // Render rubberband box selection preview
            if self.is_box_selecting {
                if let (Some(start), Some(curr)) = (self.box_select_start, self.box_select_current) {
                    let p1 = self.doc_to_screen(start, canvas_origin);
                    let p2 = self.doc_to_screen(curr, canvas_origin);
                    let select_rect = egui::Rect::from_two_pos(p1, p2);

                    painter.rect_filled(select_rect, 0.0, egui::Color32::from_rgba_premultiplied(0, 120, 255, 30));
                    painter.rect_stroke(select_rect, 0.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 150, 255)), egui::StrokeKind::Outside);
                }
            }

            // Render drawing preview
            if self.is_drawing {
                if let (Some(start), Some(curr)) = (self.draw_start, self.draw_current) {
                    let p1 = self.doc_to_screen(start, canvas_origin);
                    let p2 = self.doc_to_screen(curr, canvas_origin);
                    let preview_stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 120, 255));

                    match self.active_tool {
                        ActiveTool::DrawRect => {
                            painter.rect_stroke(egui::Rect::from_two_pos(p1, p2), 0.0, preview_stroke, egui::StrokeKind::Outside);
                        }
                        ActiveTool::DrawCircle => {
                            let center = egui::pos2((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
                            let r = p1.distance(p2) * 0.5;
                            painter.circle_stroke(center, r, preview_stroke);
                        }
                        ActiveTool::DrawLine => {
                            painter.line_segment([p1, p2], preview_stroke);
                        }
                        _ => {}
                    }
                }
            }

            // Render all Document Shapes
            for (shape_idx, shape) in self.doc.shapes.iter().enumerate() {
                let layer = self.doc.layers.get(&shape.layer_index());
                let color_hex = layer.map(|l| l.color_hex.as_str()).unwrap_or("#000000");

                let is_selected = self.selected_shape_indices.contains(&shape_idx);

                let color = if is_selected {
                    egui::Color32::from_rgb(255, 140, 0) // Vibrant orange highlight
                } else {
                    egui::Color32::from_rgb(
                        u8::from_str_radix(&color_hex[1..3], 16).unwrap_or(0),
                        u8::from_str_radix(&color_hex[3..5], 16).unwrap_or(0),
                        u8::from_str_radix(&color_hex[5..7], 16).unwrap_or(0),
                    )
                };

                let stroke_width = if is_selected { 2.5_f32 } else { 1.5_f32 };

                for poly in shape.to_polylines() {
                    let pts: Vec<egui::Pos2> = poly
                        .points
                        .iter()
                        .map(|p| self.doc_to_screen(*p, canvas_origin))
                        .collect();

                    if pts.len() >= 2 {
                        if poly.is_closed {
                            painter.add(egui::Shape::closed_line(pts, egui::Stroke::new(stroke_width, color)));
                        } else {
                            painter.add(egui::Shape::line(pts, egui::Stroke::new(stroke_width, color)));
                        }
                    }
                }
            }

            // Draw Combined Selection Bounding Box & 8 Resize Handles
            if let Some(b) = self.selection_bounds() {
                let min_p = self.doc_to_screen(Point2D::new(b.min_x, b.min_y), canvas_origin);
                let max_p = self.doc_to_screen(Point2D::new(b.max_x, b.max_y), canvas_origin);
                let sel_rect = egui::Rect::from_two_pos(min_p, max_p);

                // Dashed selection outline
                painter.rect_stroke(
                    sel_rect,
                    0.0,
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 150, 255)),
                    egui::StrokeKind::Outside,
                );

                // Subtle blue tint to highlight selected region
                painter.rect_filled(sel_rect, 0.0, egui::Color32::from_rgba_unmultiplied(0, 150, 255, 18));

                let any_locked = self.selected_shape_indices.iter().any(|&i| i < self.doc.shapes.len() && self.doc.shapes[i].is_locked());
                if any_locked {
                    painter.text(
                        sel_rect.left_top() + egui::vec2(2.0, -4.0),
                        egui::Align2::LEFT_BOTTOM,
                        "LOCKED",
                        egui::FontId::proportional(12.0),
                        egui::Color32::from_rgb(255, 60, 60),
                    );
                }

                // 8 interactive resize handles
                let handle_pts = [
                    sel_rect.left_top(),
                    egui::pos2(sel_rect.center().x, sel_rect.top()),
                    sel_rect.right_top(),
                    egui::pos2(sel_rect.right(), sel_rect.center().y),
                    sel_rect.right_bottom(),
                    egui::pos2(sel_rect.center().x, sel_rect.bottom()),
                    sel_rect.left_bottom(),
                    egui::pos2(sel_rect.left(), sel_rect.center().y),
                ];

                for hp in handle_pts {
                    painter.rect_filled(egui::Rect::from_center_size(hp, egui::Vec2::splat(8.0)), 1.5, egui::Color32::WHITE);
                    painter.rect_stroke(
                        egui::Rect::from_center_size(hp, egui::Vec2::splat(8.0)),
                        1.5,
                        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 120, 255)),
                        egui::StrokeKind::Outside,
                    );
                }
            }

            // Laser Head Position Crosshair & Firing / Weak Light Indicator (Relative to Origin Corner)
            let head_pos = self.machine_head_screen_pos(canvas_origin);
            let head_px = head_pos.x;
            let head_py = head_pos.y;

            if self.device.is_job_running && self.device.is_laser_firing {
                // Intense cutting beam with spark
                painter.circle_filled(head_pos, 22.0, egui::Color32::from_rgba_unmultiplied(0, 160, 255, 60));
                painter.circle_filled(head_pos, 11.0, egui::Color32::from_rgba_unmultiplied(255, 120, 0, 190));
                painter.circle_filled(head_pos, 4.0, egui::Color32::WHITE);
                painter.text(
                    head_pos + egui::vec2(16.0, -8.0),
                    egui::Align2::LEFT_BOTTOM,
                    "CUTTING...",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgb(255, 160, 0),
                );
            } else if self.device.is_weak_light_on {
                // Glowing cyan/blue laser spot!
                painter.circle_filled(head_pos, 15.0, egui::Color32::from_rgba_unmultiplied(0, 220, 255, 45));
                painter.circle_filled(head_pos, 7.5, egui::Color32::from_rgba_unmultiplied(0, 255, 255, 170));
                painter.circle_filled(head_pos, 3.0, egui::Color32::WHITE);
                painter.text(
                    head_pos + egui::vec2(16.0, -8.0),
                    egui::Align2::LEFT_BOTTOM,
                    "WEAK BEAM (AIM)",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgb(0, 220, 255),
                );
            } else {
                painter.line_segment([egui::pos2(head_px - 10.0, head_py), egui::pos2(head_px + 10.0, head_py)], egui::Stroke::new(1.5_f32, egui::Color32::RED));
                painter.line_segment([egui::pos2(head_px, head_py - 10.0), egui::pos2(head_px, head_py + 10.0)], egui::Stroke::new(1.5_f32, egui::Color32::RED));
                painter.circle_stroke(head_pos, 6.0, egui::Stroke::new(1.5_f32, egui::Color32::RED));
            }
        });

        // -------------------------------------------------------------
        // TEXT TOOL DIALOG WINDOW
        // -------------------------------------------------------------
        if self.show_text_dialog {
            let mut close_dialog = false;
            let mut do_add_text = false;
            let mut is_open = true;

            egui::Window::new("Text Tool")
                .collapsible(false)
                .resizable(false)
                .default_width(380.0)
                .open(&mut is_open)
                .show(ctx, |ui| {
                    ui.heading("Text Creation for Laser");
                    ui.add_space(4.0);

                    ui.label(egui::RichText::new("Enter Text:").strong());
                    ui.text_edit_multiline(&mut self.text_input);

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Font Type:").strong());
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.text_is_singleline, false, "Outline (TrueType / Cut & Engrave)");
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.text_is_singleline, true, "Single-Line (Hershey Simplex / Fast Marking)");
                    });

                    if self.text_is_singleline {
                        ui.label(
                            egui::RichText::new("Single-Line (stroke font) draws each character with a single laser beam path without double outlines. Ideal for fast serial numbers and marking.")
                                .italics()
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 160, 255)),
                        );
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Font:");
                            let available = crate::text::available_system_fonts();
                            egui::ComboBox::from_id_salt("text_font_picker")
                                .selected_text(&self.text_font_name)
                                .show_ui(ui, |ui| {
                                    for (name, path) in available {
                                        if ui.selectable_label(self.text_font_path == path, name).clicked() {
                                            self.text_font_name = name.to_string();
                                            self.text_font_path = path.to_string();
                                        }
                                    }
                                });

                            if ui.button("Browse...").on_hover_text("Select a custom .ttf or .otf file").clicked() {
                                self.prompt_browse_font(ctx);
                            }
                        });
                        ui.label(
                            egui::RichText::new("Outline font generates closed vector paths for cutting (Line) or engraving (Fill).")
                                .italics()
                                .size(11.0)
                                .color(egui::Color32::GRAY),
                        );
                    }

                    ui.add_space(6.0);
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Height (mm):");
                        ui.add(egui::Slider::new(&mut self.text_font_size, 3.0..=120.0).suffix(" mm"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut self.text_pos_x).speed(1.0).suffix(" mm"));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut self.text_pos_y).speed(1.0).suffix(" mm"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Alignment:");
                        ui.selectable_value(&mut self.text_align, crate::text::TextAlign::Left, "Left");
                        ui.selectable_value(&mut self.text_align, crate::text::TextAlign::Center, "Center");
                        ui.selectable_value(&mut self.text_align, crate::text::TextAlign::Right, "Right");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Layer:");
                        let cur_layer_name = self.doc.layers.get(&self.selected_layer).map(|l| l.name.clone()).unwrap_or_default();
                        egui::ComboBox::from_id_salt("text_dialog_layer_picker")
                            .selected_text(format!("[C{:02}] {}", self.selected_layer, cur_layer_name))
                            .show_ui(ui, |ui| {
                                for (&idx, layer) in &self.doc.layers {
                                    if ui.selectable_label(self.selected_layer == idx, format!("[C{:02}] {}", idx, layer.name)).clicked() {
                                        self.selected_layer = idx;
                                    }
                                }
                            });
                    });

                    ui.add_space(8.0);
                    ui.separator();

                    ui.horizontal(|ui| {
                        let add_btn = egui::Button::new(
                            egui::RichText::new(" Add Text to Bed ")
                                .strong()
                                .color(egui::Color32::WHITE),
                        ).fill(egui::Color32::from_rgb(0, 120, 215));

                        if ui.add(add_btn).clicked() {
                            do_add_text = true;
                            close_dialog = true;
                        }

                        if ui.button("Close").clicked() {
                            close_dialog = true;
                        }
                    });
                });

            if do_add_text {
                self.add_text_to_document();
            }
            if close_dialog || !is_open {
                self.show_text_dialog = false;
            }
        }

        // -------------------------------------------------------------
        // 1. DEVICE CONNECTION & CONSOLE WINDOW
        // -------------------------------------------------------------
        if self.show_connection_window {
            egui::Window::new("Device Connection & Console")
                .collapsible(false)
                .resizable(true)
                .default_width(450.0)
                .default_height(480.0)
                .open(&mut self.show_connection_window)
                .show(ctx, |ui| {
                    ui.heading("Serial Port & Controller Connection");
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Controller Type:");
                        egui::ComboBox::from_id_salt("conn_controller_picker")
                            .selected_text(self.device.controller_type.name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.device.controller_type, crate::device::ControllerType::Grbl, crate::device::ControllerType::Grbl.name());
                                ui.selectable_value(&mut self.device.controller_type, crate::device::ControllerType::Smoothieware, crate::device::ControllerType::Smoothieware.name());
                                ui.selectable_value(&mut self.device.controller_type, crate::device::ControllerType::RuidaUdp, crate::device::ControllerType::RuidaUdp.name());
                                ui.selectable_value(&mut self.device.controller_type, crate::device::ControllerType::VirtualDemo, crate::device::ControllerType::VirtualDemo.name());
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Port:");
                        if self.available_ports.is_empty() {
                            self.available_ports = crate::device::DeviceManager::scan_available_ports();
                        }

                        if self.device.selected_port.is_empty() && !self.available_ports.is_empty() {
                            self.device.selected_port = self.available_ports[0].clone();
                        }

                        egui::ComboBox::from_id_salt("conn_port_picker")
                            .selected_text(if self.device.selected_port.is_empty() { "Select port..." } else { &self.device.selected_port })
                            .show_ui(ui, |ui| {
                                for p in &self.available_ports {
                                    ui.selectable_value(&mut self.device.selected_port, p.clone(), p);
                                }
                            });

                        if ui.button("Scan Ports").on_hover_text("Refresh connected USB serial ports").clicked() {
                            self.available_ports = crate::device::DeviceManager::scan_available_ports();
                            if !self.available_ports.is_empty() {
                                self.device.selected_port = self.available_ports[0].clone();
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Baud Rate:");
                        egui::ComboBox::from_id_salt("conn_baud_picker")
                            .selected_text(format!("{}", self.device.selected_baud))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.device.selected_baud, 115200, "115200 (GRBL standard)");
                                ui.selectable_value(&mut self.device.selected_baud, 250000, "250000");
                                ui.selectable_value(&mut self.device.selected_baud, 9600, "9600");
                                ui.selectable_value(&mut self.device.selected_baud, 57600, "57600");
                            });
                    });

                    ui.add_space(4.0);
                    ui.checkbox(&mut self.device.settings.auto_connect, "Auto-connect to this port on startup");

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let is_conn = self.device.status == crate::device::ConnectionStatus::Connected;
                        if is_conn {
                            if self.device.is_physically_connected() {
                                ui.colored_label(egui::Color32::from_rgb(0, 220, 0), "CONNECTED (USB)");
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(255, 200, 50), "SIMULATOR (DEMO)");
                            }
                            if ui.button("Disconnect").clicked() {
                                self.device.disconnect();
                            }
                        } else {
                            ui.colored_label(egui::Color32::RED, "NOT CONNECTED");
                            let conn_btn = egui::Button::new(
                                egui::RichText::new(" Connect to Device ").strong().color(egui::Color32::WHITE),
                            ).fill(egui::Color32::from_rgb(0, 140, 255));
                            if ui.add(conn_btn).clicked() {
                                let port = self.device.selected_port.clone();
                                let baud = self.device.selected_baud;
                                let _ = self.device.connect(&port, baud);
                            }
                        }

                        if ui.button("Save Connection").on_hover_text("Save this port as default for startup").clicked() {
                            self.device.settings.default_port = self.device.selected_port.clone();
                            self.device.settings.default_baud = self.device.selected_baud;
                            let _ = self.device.settings.save();
                            self.status_msg = "Connection settings saved (~/.config/laser_burn/settings.json)".to_string();
                        }
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.label(egui::RichText::new("G-Code Console & Log Terminal:").strong());

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.add_space(2.0);
                            for line in &self.device.console_history {
                                let color = if line.starts_with('>') {
                                    egui::Color32::from_rgb(100, 200, 255)
                                } else if line.contains("error") || line.contains("WARNING") || line.contains("VAROITUS") {
                                    egui::Color32::from_rgb(255, 120, 100)
                                } else if line.contains("ok") || line.contains("CONNECTION") || line.contains("BEAM") || line.contains("YHTEYS") || line.contains("VALO") {
                                    egui::Color32::from_rgb(140, 255, 140)
                                } else {
                                    egui::Color32::LIGHT_GRAY
                                };
                                ui.colored_label(color, line);
                            }
                        });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let text_edit = ui.text_edit_singleline(&mut self.command_input);
                        let send_pressed = ui.button("Send").clicked() || (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                        if send_pressed && !self.command_input.trim().is_empty() {
                            let cmd = self.command_input.trim().to_string();
                            self.device.send_raw(&cmd);
                            self.command_input.clear();
                            text_edit.request_focus();
                        }

                        if ui.button("Clear Log").clicked() {
                            self.device.console_history.clear();
                        }
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Quick Commands:");
                        if ui.button("$$").on_hover_text("Show GRBL settings").clicked() {
                            self.device.send_raw("$$");
                        }
                        if ui.button("?").on_hover_text("Query device status and coordinates").clicked() {
                            self.device.send_raw("?");
                        }
                        if ui.button("$10=19").on_hover_text("Enable pin reporting ($10=19)").clicked() {
                            self.device.send_raw("$10=19");
                        }
                        if ui.button("$21=1").on_hover_text("Enable hard limit protection ($21=1)").clicked() {
                            self.device.send_raw("$21=1");
                        }
                        if self.device.settings.enable_limit_switches {
                            if ui.button("$H").on_hover_text("Home machine ($H)").clicked() {
                                self.device.home();
                            }
                        }
                        if ui.button("$X").on_hover_text("Unlock alarm ($X)").clicked() {
                            self.device.send_raw("$X");
                            self.device.send_raw("$21=1");
                        }
                    });
                });
        }

        // -------------------------------------------------------------
        // 2. LASER CONTROL & POSITIONING WINDOW (MOVE & WEAK LIGHT)
        // -------------------------------------------------------------
        if self.show_move_window {
            let sel_center = self.selection_bounds().map(|b| b.center());
            let mut do_start_box_frame = false;
            let mut do_start_contour_frame = false;

            egui::Window::new("Laser Control & Positioning")
                .collapsible(false)
                .resizable(false)
                .default_width(380.0)
                .open(&mut self.show_move_window)
                .show(ctx, |ui| {
                    ui.heading("Laser Head Positioning & Jog");
                    ui.add_space(4.0);

                    // Coordinates box
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Machine Coordinates:").strong());
                            ui.label(format!("X: {:.2} mm   Y: {:.2} mm", self.device.machine_x, self.device.machine_y));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Work Coordinates:").strong());
                            ui.label(format!("X: {:.2} mm   Y: {:.2} mm", self.device.work_x, self.device.work_y));
                        });
                    });

                    ui.add_space(6.0);

                    // Step size buttons
                    ui.horizontal(|ui| {
                        ui.label("Step Size:");
                        ui.selectable_value(&mut self.jog_step, 0.1, "0.1mm");
                        ui.selectable_value(&mut self.jog_step, 1.0, "1mm");
                        ui.selectable_value(&mut self.jog_step, 5.0, "5mm");
                        ui.selectable_value(&mut self.jog_step, 10.0, "10mm");
                        ui.selectable_value(&mut self.jog_step, 50.0, "50mm");
                        ui.selectable_value(&mut self.jog_step, 100.0, "100mm");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Speed (mm/min):");
                        ui.add(egui::Slider::new(&mut self.jog_speed, 500.0..=8000.0).suffix(" mm/min"));
                    });

                    ui.add_space(4.0);
                    if self.device.is_physically_connected() || self.device.settings.enable_limit_switches {
                        ui.horizontal(|ui| {
                            ui.label("Limit Switches:");
                            let x_active = self.device.pin_limit_x;
                            let y_active = self.device.pin_limit_y;

                            let x_color = if x_active { egui::Color32::from_rgb(255, 60, 60) } else { egui::Color32::from_rgb(130, 150, 170) };
                            let y_color = if y_active { egui::Color32::from_rgb(255, 60, 60) } else { egui::Color32::from_rgb(130, 150, 170) };

                            ui.label(egui::RichText::new(format!("X: {}", if x_active { "TRIGGERED" } else { "Open" })).color(x_color).strong());
                            ui.label(egui::RichText::new(format!("Y: {}", if y_active { "TRIGGERED" } else { "Open" })).color(y_color).strong());

                            if self.device.pin_limit_z {
                                ui.label(egui::RichText::new("Z: Triggered").color(egui::Color32::from_rgb(255, 120, 0)).strong());
                            }

                            if ui.small_button("Sync ($10=19, $21=1)").on_hover_text("Send $10=19 (real-time pin reporting) and $21=1 (hard limits protection) to GRBL").clicked() {
                                self.device.send_raw("$10=19");
                                self.device.send_raw("$21=1");
                                self.device.send_realtime(b'?');
                                self.status_msg = "Sent $10=19 and $21=1 to GRBL".to_string();
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Home Speed:");
                            if ui.add(egui::Slider::new(&mut self.device.settings.homing_speed, 100.0..=1200.0).suffix(" mm/min")).on_hover_text("Gentle seek speed for mechanical switch calibration (default 400 mm/min = 6.6 mm/s)").changed() {
                                let _ = self.device.settings.save();
                            }
                        });
                    }

                    if self.device.grbl_state.starts_with("Alarm") {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("Status: {}", self.device.grbl_state)).color(egui::Color32::from_rgb(255, 60, 60)).strong());
                            if ui.button("Unlock ($X)").clicked() {
                                self.device.send_raw("$X");
                                self.device.send_raw("$21=1");
                            }
                        });
                    }
                    ui.add_space(4.0);

                    // Interactive D-Pad Grid
                    let step = self.jog_step;
                    let speed = self.jog_speed;

                    let btn_size = egui::vec2(64.0, 32.0);
                    let grid_w = 3.0 * btn_size.x + 2.0 * 4.0;
                    let pad = ((ui.available_width() - grid_w) / 2.0).max(0.0);

                    ui.horizontal(|ui| {
                        ui.add_space(pad);
                        egui::Grid::new("move_window_grid")
                            .spacing([4.0, 4.0])
                            .min_col_width(btn_size.x)
                            .show(ui, |ui| {
                                // Row 1: NW, Y+, NE
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X-Y+").size(11.0))).on_hover_text("X- / Y+").clicked() {
                                    self.device.jog_arrow(-1.0, 1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("Y+").strong().size(15.0))).on_hover_text("Jog Y+").clicked() {
                                    self.device.jog_arrow(0.0, 1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X+Y+").size(11.0))).on_hover_text("X+ / Y+").clicked() {
                                    self.device.jog_arrow(1.0, 1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                ui.end_row();

                                // Row 2: X-, Origin, X+
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X-").strong().size(15.0))).on_hover_text("Jog X-").clicked() {
                                    self.device.jog_arrow(-1.0, 0.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("0,0").strong().size(14.0))).on_hover_text("Move to origin (G0 X0 Y0)").clicked() {
                                    self.device.go_to_origin();
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X+").strong().size(15.0))).on_hover_text("Jog X+").clicked() {
                                    self.device.jog_arrow(1.0, 0.0, step, speed);
                                    ctx.request_repaint();
                                }
                                ui.end_row();

                                // Row 3: SW, Y-, SE
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X-Y-").size(11.0))).on_hover_text("X- / Y-").clicked() {
                                    self.device.jog_arrow(-1.0, -1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("Y-").strong().size(15.0))).on_hover_text("Jog Y-").clicked() {
                                    self.device.jog_arrow(0.0, -1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                if ui.add_sized(btn_size, egui::Button::new(egui::RichText::new("X+Y-").size(11.0))).on_hover_text("X+ / Y-").clicked() {
                                    self.device.jog_arrow(1.0, -1.0, step, speed);
                                    ctx.request_repaint();
                                }
                                ui.end_row();
                            });
                    });

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Go to Corner:").strong());
                    ui.horizontal(|ui| {
                        let (tl_x, tl_y) = self.device.get_corner_coordinates("top_left");
                        let tl_text = format!("Top-Left ({:.0},{:.0})", tl_x, tl_y);
                        if ui.button(tl_text)
                            .on_hover_text(format!("Move laser head directly to top-left corner (X={:.1}, Y={:.1} mm)", tl_x, tl_y))
                            .clicked()
                        {
                            self.device.go_to_corner("top_left");
                            ctx.request_repaint();
                        }

                        let (tr_x, tr_y) = self.device.get_corner_coordinates("top_right");
                        let tr_text = format!("Top-Right ({:.0},{:.0})", tr_x, tr_y);
                        if ui.button(tr_text)
                            .on_hover_text(format!("Move laser head directly to top-right corner (X={:.1}, Y={:.1} mm)", tr_x, tr_y))
                            .clicked()
                        {
                            self.device.go_to_corner("top_right");
                            ctx.request_repaint();
                        }

                        let (bl_x, bl_y) = self.device.get_corner_coordinates("bottom_left");
                        let bl_text = format!("Bottom-Left ({:.0},{:.0})", bl_x, bl_y);
                        if ui.button(bl_text)
                            .on_hover_text(format!("Move laser head directly to bottom-left corner (X={:.1}, Y={:.1} mm)", bl_x, bl_y))
                            .clicked()
                        {
                            self.device.go_to_corner("bottom_left");
                            ctx.request_repaint();
                        }

                        let (br_x, br_y) = self.device.get_corner_coordinates("bottom_right");
                        let br_text = format!("Bottom-Right ({:.0},{:.0})", br_x, br_y);
                        if ui.button(br_text)
                            .on_hover_text(format!("Move laser head directly to bottom-right corner (X={:.1}, Y={:.1} mm)", br_x, br_y))
                            .clicked()
                        {
                            self.device.go_to_corner("bottom_right");
                            ctx.request_repaint();
                        }
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if self.device.settings.enable_limit_switches {
                            let is_homing = self.device.homing_phase.is_some();
                            let reset_btn = egui::Button::new(
                                egui::RichText::new(if is_homing { "Abort Reset" } else { "Reset Origin (Switches)" })
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            ).fill(if is_homing { egui::Color32::from_rgb(220, 50, 40) } else { egui::Color32::from_rgb(33, 115, 185) });

                            if ui.add(reset_btn).on_hover_text(if is_homing { "Click to immediately abort homing move" } else { "Move head to physical limit switches (ignoring soft limits) and calibrate origin (0,0)" }).clicked() {
                                if is_homing {
                                    self.device.stop_job();
                                } else {
                                    self.device.start_mechanical_home_reset();
                                }
                            }
                        }

                        if ui.button("Set Origin (G92)").on_hover_text("Set this position as work origin (0,0)").clicked() {
                            self.device.set_origin_here();
                        }
                        if ui.button("Center on Selection").clicked() {
                            if let Some(c) = sel_center {
                                self.device.go_to_point(c);
                            }
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();

                    // -------------------------------------------------
                    // WEAK LIGHT / AIM BEAM SECTION
                    // -------------------------------------------------
                    ui.heading("Weak Beam / Alignment Pointer");
                    ui.add_space(2.0);

                    let is_on = self.device.is_weak_light_on;
                    let weak_btn_text = if is_on {
                        "WEAK BEAM ON (Click to turn off)"
                    } else {
                        "Turn on weak beam for alignment"
                    };

                    let weak_btn = egui::Button::new(
                        egui::RichText::new(weak_btn_text)
                            .strong()
                            .color(if is_on { egui::Color32::WHITE } else { egui::Color32::from_rgb(220, 240, 255) }),
                    ).fill(if is_on { egui::Color32::from_rgb(255, 69, 0) } else { egui::Color32::from_rgb(40, 70, 110) });

                    if ui.add(weak_btn).clicked() {
                        self.device.toggle_weak_light();
                    }

                    ui.horizontal(|ui| {
                        ui.label("Weak Beam Power:");
                        ui.add(egui::Slider::new(&mut self.device.settings.weak_light_percent, 0.2..=5.0).suffix(" %"));
                    });

                    ui.label(
                        egui::RichText::new("Weak beam (1.0%) turns on laser at low power so you can see the focal point without burning the workpiece.")
                            .italics()
                            .size(11.0)
                            .color(egui::Color32::GRAY),
                    );

                    ui.add_space(8.0);
                    ui.separator();

                    // -------------------------------------------------
                    // FRAMING TEST SECTION
                    // -------------------------------------------------
                    ui.heading("Framing Test with Weak Beam");
                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        let box_btn = egui::Button::new(
                            egui::RichText::new(" Box Framing Test ").strong().color(egui::Color32::WHITE),
                        ).fill(egui::Color32::from_rgb(0, 120, 215));

                        if ui.add(box_btn).on_hover_text("Traces bounding box with weak beam").clicked() {
                            do_start_box_frame = true;
                        }

                        let contour_btn = egui::Button::new(
                            egui::RichText::new(" Contour Framing Test ").strong().color(egui::Color32::WHITE),
                        ).fill(egui::Color32::from_rgb(140, 40, 180));

                        if ui.add(contour_btn).on_hover_text("Traces actual cut paths with weak beam").clicked() {
                            do_start_contour_frame = true;
                        }
                    });

                    if self.device.is_framing_active {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::YELLOW, "Framing test in progress...");
                            if ui.button("Stop Test").clicked() {
                                self.device.stop_frame_test();
                            }
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // Custom Soft Limits (Corners)
                    ui.heading("Custom Soft Limits (Corners)");
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut self.device.settings.custom_boundaries.enabled, "Enforce learned soft limits").changed() {
                            let _ = self.device.settings.save();
                        }
                    });

                    ui.label("Jog laser to target corner and save:");
                    ui.horizontal(|ui| {
                        let bl_label = match self.device.settings.custom_boundaries.corner_bottom_left {
                            Some((x, y)) => format!("[Set] Bottom-Left ({:.0},{:.0})", x, y),
                            None => "Bottom-Left".to_string(),
                        };
                        if ui.button(bl_label).on_hover_text("Save current position as bottom-left corner").clicked() {
                            self.device.save_boundary_corner("bottom_left");
                        }

                        let br_label = match self.device.settings.custom_boundaries.corner_bottom_right {
                            Some((x, y)) => format!("[Set] Bottom-Right ({:.0},{:.0})", x, y),
                            None => "Bottom-Right".to_string(),
                        };
                        if ui.button(br_label).on_hover_text("Save current position as bottom-right corner").clicked() {
                            self.device.save_boundary_corner("bottom_right");
                        }
                    });

                    ui.horizontal(|ui| {
                        let tl_label = match self.device.settings.custom_boundaries.corner_top_left {
                            Some((x, y)) => format!("[Set] Top-Left ({:.0},{:.0})", x, y),
                            None => "Top-Left".to_string(),
                        };
                        if ui.button(tl_label).on_hover_text("Save current position as top-left corner").clicked() {
                            self.device.save_boundary_corner("top_left");
                        }

                        let tr_label = match self.device.settings.custom_boundaries.corner_top_right {
                            Some((x, y)) => format!("[Set] Top-Right ({:.0},{:.0})", x, y),
                            None => "Top-Right".to_string(),
                        };
                        if ui.button(tr_label).on_hover_text("Save current position as top-right corner").clicked() {
                            self.device.save_boundary_corner("top_right");
                        }
                    });

                    if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (
                        self.device.settings.custom_boundaries.min_x,
                        self.device.settings.custom_boundaries.max_x,
                        self.device.settings.custom_boundaries.min_y,
                        self.device.settings.custom_boundaries.max_y,
                    ) {
                        let b_color = if self.device.settings.custom_boundaries.enabled {
                            egui::Color32::from_rgb(0, 220, 100)
                        } else {
                            egui::Color32::GRAY
                        };
                        ui.label(egui::RichText::new(format!(
                            "Limits: X: {:.1}..{:.1}, Y: {:.1}..{:.1} mm (Size: {:.1} x {:.1} mm)",
                            min_x, max_x, min_y, max_y, (max_x - min_x).abs(), (max_y - min_y).abs()
                        )).color(b_color).strong());
                    }

                    if ui.button("Clear Custom Limits").on_hover_text("Clear all taught corners and disable soft limits").clicked() {
                        self.device.clear_boundary_corners();
                    }
                });

            if do_start_box_frame {
                self.start_box_frame();
            }
            if do_start_contour_frame {
                self.start_contour_frame();
            }
        }

        // -------------------------------------------------------------
        // 3. DEVICE & WORKBED SETTINGS WINDOW
        // -------------------------------------------------------------
        if self.show_settings_window {
            let mut close_settings = false;

            egui::Window::new("Device & Bed Settings")
                .collapsible(false)
                .resizable(false)
                .default_width(420.0)
                .open(&mut self.show_settings_window)
                .show(ctx, |ui| {
                    ui.heading("LaserBurn Settings");
                    ui.add_space(4.0);

                    ui.label(egui::RichText::new("Bed Dimensions (mm):").strong());
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        if ui.add(egui::DragValue::new(&mut self.device.settings.bed_width).speed(1.0).range(50.0..=2000.0).suffix(" mm")).changed() {
                            self.doc.width = self.device.settings.bed_width;
                        }
                        ui.label("Height:");
                        if ui.add(egui::DragValue::new(&mut self.device.settings.bed_height).speed(1.0).range(50.0..=2000.0).suffix(" mm")).changed() {
                            self.doc.height = self.device.settings.bed_height;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Origin Corner:");
                        egui::ComboBox::from_id_salt("settings_origin_picker")
                            .selected_text(match self.device.settings.origin_corner {
                                crate::device::OriginCorner::TopLeft => "Top-Left (Rear-Left)",
                                crate::device::OriginCorner::BottomLeft => "Bottom-Left (Front-Left)",
                                crate::device::OriginCorner::TopRight => "Top-Right (Rear-Right)",
                                crate::device::OriginCorner::BottomRight => "Bottom-Right (Front-Right)",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.device.settings.origin_corner, crate::device::OriginCorner::TopLeft, "Top-Left (Rear-Left)");
                                ui.selectable_value(&mut self.device.settings.origin_corner, crate::device::OriginCorner::BottomLeft, "Bottom-Left (Front-Left)");
                                ui.selectable_value(&mut self.device.settings.origin_corner, crate::device::OriginCorner::TopRight, "Top-Right (Rear-Right)");
                                ui.selectable_value(&mut self.device.settings.origin_corner, crate::device::OriginCorner::BottomRight, "Bottom-Right (Front-Right)");
                            });
                    });

                    ui.add_space(6.0);
                    ui.separator();

                    ui.label(egui::RichText::new("Laser & Beam Parameters:").strong());
                    ui.horizontal(|ui| {
                        ui.label("Laser Type:");
                        egui::ComboBox::from_id_salt("settings_laser_type")
                            .selected_text(match self.device.settings.laser_type {
                                crate::device::LaserType::Diode => "Diode Laser (Blue 450nm)",
                                crate::device::LaserType::Co2 => "CO2 Laser (10600nm)",
                                crate::device::LaserType::FiberGalvo => "Fiber / Galvo (1064nm)",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.device.settings.laser_type, crate::device::LaserType::Diode, "Diode Laser (Blue 450nm)");
                                ui.selectable_value(&mut self.device.settings.laser_type, crate::device::LaserType::Co2, "CO2 Laser (10600nm)");
                                ui.selectable_value(&mut self.device.settings.laser_type, crate::device::LaserType::FiberGalvo, "Fiber / Galvo (1064nm)");
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Max Power ($30 / S-max):");
                        ui.add(egui::DragValue::new(&mut self.device.settings.max_s_value).speed(10.0).range(1.0..=10000.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Weak Beam Power for Alignment:");
                        ui.add(egui::Slider::new(&mut self.device.settings.weak_light_percent, 0.2..=5.0).suffix(" %"));
                    });

                    ui.add_space(6.0);
                    ui.separator();

                    ui.label(egui::RichText::new("Speed Limits:").strong());
                    ui.horizontal(|ui| {
                        ui.label("Rapid Speed G0:");
                        ui.add(egui::DragValue::new(&mut self.device.settings.rapid_speed).speed(50.0).suffix(" mm/min"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Jog Speed:");
                        ui.add(egui::DragValue::new(&mut self.device.settings.jog_speed).speed(50.0).suffix(" mm/min"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Framing Test Speed:");
                        ui.add(egui::DragValue::new(&mut self.device.settings.frame_speed).speed(50.0).suffix(" mm/min"));
                    });

                    ui.add_space(8.0);
                    ui.separator();

                    ui.label(egui::RichText::new("Limit Switches:").strong());
                    ui.checkbox(&mut self.device.settings.enable_limit_switches, "Enable physical limit switches (Switches installed)");
                    if !self.device.settings.enable_limit_switches {
                        ui.label(egui::RichText::new("Switch-related buttons and homing are disabled by default for machine protection.").small().weak());
                    } else {
                        ui.label(egui::RichText::new("Ensure your machine has working switches and wiring before homing.").small().color(egui::Color32::from_rgb(220, 160, 40)));
                        ui.horizontal(|ui| {
                            ui.label("Homing / Seek Speed:");
                            if ui.add(egui::Slider::new(&mut self.device.settings.homing_speed, 100.0..=1200.0).suffix(" mm/min")).on_hover_text("Gentle seek speed for mechanical switch calibration (default 400 mm/min = 6.6 mm/s)").changed() {
                                let _ = self.device.settings.save();
                            }
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    ui.checkbox(&mut self.device.settings.auto_connect, "Auto-connect to laser port on startup");

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save & Close").clicked() {
                            if let Err(e) = self.device.settings.save() {
                                self.status_msg = format!("Error saving settings: {}", e);
                            } else {
                                self.status_msg = "Settings saved (~/.config/laser_burn/settings.json)".to_string();
                            }
                            close_settings = true;
                        }
                    });
                });

            if close_settings {
                self.show_settings_window = false;
            }
        }

        // -------------------------------------------------------------
        // 4. KAMERA-IKKUNA (CAMERA FEED & ALIGNMENT)
        // -------------------------------------------------------------
        if self.show_camera_window {
            let mut is_open = self.show_camera_window;
            let mut need_stream_restart = false;

            let max_w = (ctx.screen_rect().width() - 40.0).max(450.0);
            let max_h = (ctx.screen_rect().height() - 40.0).max(520.0);

            egui::Window::new("Camera Feed & Alignment")
                .open(&mut is_open)
                .default_size([500.0, 640.0])
                .min_size([400.0, 480.0])
                .max_size([max_w, max_h])
                .constrain(true)
                .resizable(true)
                .show(ctx, |ui| {
                    // Row 1: Device Selection & Scan
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Device:").strong());

                        let current_cam_name = if self.camera_manager.devices.is_empty() {
                            "No cameras detected".to_string()
                        } else if let Some(d) = self.camera_manager.devices.get(self.camera_manager.selected_device_idx) {
                            d.name.clone()
                        } else {
                            "Select camera...".to_string()
                        };

                        let dropdown_w = (ui.available_width() - 120.0).clamp(120.0, 260.0);
                        egui::ComboBox::from_id_salt("camera_device_dropdown")
                            .selected_text(&current_cam_name)
                            .width(dropdown_w)
                            .show_ui(ui, |ui| {
                                for (idx, d) in self.camera_manager.devices.iter().enumerate() {
                                    if ui.selectable_value(&mut self.camera_manager.selected_device_idx, idx, &d.name).clicked() {
                                        if self.camera_manager.is_streaming {
                                            need_stream_restart = true;
                                        }
                                    }
                                }
                            });

                        if ui.button("Scan Cameras").on_hover_text("Scan for connected USB and network cameras").clicked() {
                            self.camera_manager.refresh_devices();
                        }
                    });

                    // Row 2: Stream Controls & Status
                    ui.horizontal(|ui| {
                        if !self.camera_manager.is_streaming {
                            let start_btn = egui::Button::new(egui::RichText::new("Start Stream").color(egui::Color32::from_rgb(0, 220, 0)).strong());
                            if ui.add(start_btn).on_hover_text("Start real-time video stream").clicked() {
                                self.camera_manager.start_stream();
                            }
                        } else {
                            let stop_btn = egui::Button::new(egui::RichText::new("Stop Stream").color(egui::Color32::from_rgb(255, 90, 90)).strong());
                            if ui.add(stop_btn).on_hover_text("Stop video stream").clicked() {
                                self.camera_manager.stop_stream();
                            }
                        }

                        let is_capturing = self.oneshot_snapshot.is_some();
                        let snap_btn = egui::Button::new(
                            egui::RichText::new(if is_capturing { "Capturing..." } else { "Update Overlay" })
                                .strong()
                                .color(if is_capturing { egui::Color32::from_rgb(255, 230, 80) } else { egui::Color32::WHITE })
                        ).fill(if is_capturing { egui::Color32::from_rgb(180, 100, 20) } else { egui::Color32::from_rgb(33, 115, 185) });

                        if ui.add(snap_btn).on_hover_text("Capture fresh snapshot of bed and workpiece (Cmd+U)").clicked() {
                            self.trigger_update_overlay();
                        }

                        ui.separator();

                        let (dot_color, state_txt) = if self.camera_manager.is_streaming {
                            (egui::Color32::from_rgb(0, 230, 0), "LIVE")
                        } else {
                            (egui::Color32::GRAY, "IDLE")
                        };
                        ui.label(egui::RichText::new(format!("{}", state_txt)).color(dot_color).strong());
                    });

                    if need_stream_restart {
                        self.camera_manager.stop_stream();
                        self.camera_manager.start_stream();
                    }

                    ui.separator();

                    // Navigation Tabs
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.camera_window_tab, 0, "4-Point Wizard");
                        ui.selectable_value(&mut self.camera_window_tab, 1, "Manual Sliders");
                        ui.selectable_value(&mut self.camera_window_tab, 2, "Lens & Fisheye");
                        ui.selectable_value(&mut self.camera_window_tab, 3, "Canvas Overlay");
                    });

                    ui.separator();

                    // Fixed Bottom Toolbar docked using TopBottomPanel (No feedback loop)
                    egui::TopBottomPanel::bottom("camera_docked_bottom_panel")
                        .resizable(false)
                        .show_separator_line(true)
                        .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(0, 4)))
                        .show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.camera_calibration.show_on_canvas, "Show on Canvas");

                                if self.camera_calibration.use_homography {
                                    ui.label(egui::RichText::new("Homography: Active").color(egui::Color32::from_rgb(0, 230, 0)).small().strong());
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(egui::RichText::new("Save Calibration").color(egui::Color32::from_rgb(100, 220, 255)).strong()).clicked() {
                                        if let Err(e) = self.camera_calibration.save() {
                                            self.status_msg = format!("Save Error: {}", e);
                                        } else {
                                            self.status_msg = "Camera calibration saved (~/.config/laser_burn/camera_calibration.json)".to_string();
                                        }
                                    }
                                });
                            });
                        });

                    // Scrollable central area fills remaining space without window expansion
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show_inside(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    match self.camera_window_tab {
                                        0 => {
                                            // -------------------------------------------------
                                            // TAB 0: 4-POINT ALIGNMENT WIZARD (HOMOGRAPHY)
                                            // -------------------------------------------------
                                            ui.heading("4-Point Alignment Wizard");
                                            ui.label(egui::RichText::new("Mathematically align camera with 4 scrap papers or cardboard pieces!").small().weak());
                                            ui.add_space(4.0);

                                            // Wizard Step Tabs
                                            ui.horizontal(|ui| {
                                                let step_names = ["1. Place Papers", "2. Burn Marks", "3. Click Targets", "4. Complete"];
                                                for (i, name) in step_names.iter().enumerate() {
                                                    let is_cur = self.wizard_step == i;
                                                    let is_past = self.wizard_step > i;
                                                    let color = if is_cur {
                                                        egui::Color32::from_rgb(100, 210, 255)
                                                    } else if is_past {
                                                        egui::Color32::from_rgb(80, 230, 80)
                                                    } else {
                                                        egui::Color32::GRAY
                                                    };
                                                    if ui.selectable_label(is_cur, egui::RichText::new(*name).color(color).strong()).clicked() {
                                                        self.wizard_step = i;
                                                    }
                                                }
                                            });
                                            ui.separator();
                                            ui.add_space(4.0);

                                            let targets = self.alignment_target_doc_coords();

                                            match self.wizard_step {
                                                0 => {
                                                    // STEP 0: SETUP 4 PAPERS
                                                    ui.label(egui::RichText::new("Step 1: Place 4 small pieces of paper or cardboard").strong());
                                                    ui.label(egui::RichText::new("You do NOT need a large sheet covering the whole bed! Just place 4 small scrap pieces (A5/A6, Post-it notes, tape, or cardboard) under the 4 target locations.").small());
                                                    ui.add_space(6.0);

                                                    ui.horizontal(|ui| {
                                                        ui.label("Margin from Bed Edges (mm):");
                                                        ui.add(egui::DragValue::new(&mut self.camera_calibration.target_inset_mm).speed(1.0).range(15.0..=150.0));
                                                    });
                                                    ui.add(egui::Slider::new(&mut self.camera_calibration.target_inset_mm, 15.0..=150.0).text("Edge Inset"));
                                                    ui.add_space(6.0);

                                                    ui.label(egui::RichText::new("Target Locations on Laser Bed:").strong());
                                                    egui::Frame::group(ui.style())
                                                        .inner_margin(egui::Margin::symmetric(8, 6))
                                                        .show(ui, |ui| {
                                                            let corner_names = ["1. Top-Left", "2. Top-Right", "3. Bottom-Right", "4. Bottom-Left"];
                                                            for i in 0..4 {
                                                                let (x, y) = targets[i];
                                                                ui.horizontal(|ui| {
                                                                    ui.label(egui::RichText::new(corner_names[i]).strong());
                                                                    ui.label(egui::RichText::new(format!("({:.0}, {:.0}) mm", x, y)).monospace());
                                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                        if ui.button("Jog Laser Spot").on_hover_text("Move laser nozzle directly over this spot with weak beam on").clicked() {
                                                                            self.jog_to_alignment_target(i);
                                                                        }
                                                                    });
                                                                });
                                                                if i < 3 {
                                                                    ui.separator();
                                                                }
                                                            }
                                                        });
                                                    ui.add_space(8.0);
                                                    if ui.button(egui::RichText::new("Framing: Visit All 4 Spots").color(egui::Color32::from_rgb(100, 220, 255))).on_hover_text("Laser will visit each of the 4 spots so you can verify paper placement").clicked() {
                                                        for i in 0..4 {
                                                            self.jog_to_alignment_target(i);
                                                        }
                                                    }

                                                    ui.add_space(10.0);
                                                    ui.horizontal(|ui| {
                                                        if ui.button(egui::RichText::new("Next: Proceed to Burn Marks ->").color(egui::Color32::from_rgb(0, 230, 0)).strong()).clicked() {
                                                            self.wizard_step = 1;
                                                        }
                                                        if ui.button("Skip Burning (Marks already placed) ->").clicked() {
                                                            self.wizard_step = 2;
                                                        }
                                                    });
                                                }
                                                1 => {
                                                    // STEP 1: BURN BULLSEYE MARKS
                                                    ui.label(egui::RichText::new("Step 2: Burn 4 High-Contrast Bullseye Marks").strong());
                                                    ui.label(egui::RichText::new("The laser will mark a 10mm crosshair and concentric rings on each of your 4 scrap papers, then park out of the way.").small());
                                                    ui.add_space(6.0);

                                                    ui.horizontal(|ui| {
                                                        ui.label("Laser Power (%):");
                                                        ui.add(egui::Slider::new(&mut self.camera_calibration.burn_power_percent, 1.0..=100.0).custom_formatter(|v, _| format!("{:.0}%", v)));
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Marking Speed (mm/min):");
                                                        ui.add(egui::DragValue::new(&mut self.camera_calibration.burn_speed).speed(50.0).range(200.0..=6000.0));
                                                    });
                                                    ui.add_space(8.0);

                                                    if self.device.is_job_running {
                                                        ui.horizontal(|ui| {
                                                            ui.spinner();
                                                            ui.label(egui::RichText::new(format!("Marking in progress: {:.0}%", self.device.job_progress * 100.0)).color(egui::Color32::YELLOW));
                                                            if ui.button(egui::RichText::new("Stop").color(egui::Color32::RED)).clicked() {
                                                                self.device.stop_job();
                                                            }
                                                        });
                                                    } else {
                                                        ui.horizontal(|ui| {
                                                            if ui.button(egui::RichText::new("Burn 4 Alignment Marks").color(egui::Color32::from_rgb(255, 120, 50)).strong()).on_hover_text("Execute rapid G-code to burn 4 crosshair bullseyes").clicked() {
                                                                self.burn_alignment_targets();
                                                            }
                                                            if ui.button("Marks Already Burned / Skip ->").clicked() {
                                                                self.wizard_step = 2;
                                                            }
                                                        });
                                                    }

                                                    ui.add_space(10.0);
                                                    ui.horizontal(|ui| {
                                                        if ui.button("<- Back to Paper Setup").clicked() {
                                                            self.wizard_step = 0;
                                                        }
                                                        if ui.button(egui::RichText::new("Next: Click Targets on Camera ->").color(egui::Color32::from_rgb(0, 230, 0)).strong()).clicked() {
                                                            self.wizard_step = 2;
                                                        }
                                                    });
                                                }
                                                2 => {
                                                    // STEP 2: CLICK TARGETS WITH ZOOM LOUPE
                                                    ui.label(egui::RichText::new("Step 3: Click the 4 Bullseye Targets on the Camera Image").strong());
                                                    ui.label(egui::RichText::new("Hover mouse over a target to see the 4x Zoom Loupe for pixel-perfect centering!").small().weak());
                                                    ui.add_space(4.0);

                                                    let target_labels = [
                                                        ("Target 1 (Top-Left)", egui::Color32::from_rgb(255, 80, 80)),
                                                        ("Target 2 (Top-Right)", egui::Color32::from_rgb(80, 240, 80)),
                                                        ("Target 3 (Bottom-Right)", egui::Color32::from_rgb(255, 230, 70)),
                                                        ("Target 4 (Bottom-Left)", egui::Color32::from_rgb(80, 210, 255)),
                                                    ];

                                                    ui.horizontal(|ui| {
                                                        for i in 0..4 {
                                                            let is_active = self.wizard_target_idx == i;
                                                            let has_val = self.wizard_points[i].is_some();
                                                            let (name, col) = target_labels[i];
                                                            let text = if has_val { format!("[OK] {}", name) } else { name.to_string() };
                                                            if ui.selectable_label(is_active, egui::RichText::new(text).color(col).strong()).clicked() {
                                                                self.wizard_target_idx = i;
                                                            }
                                                        }
                                                    });

                                                    ui.add_space(6.0);

                                                    // Interactive Camera Canvas with Loupe
                                                    if let Some(ref texture) = self.camera_texture {
                                                        let avail_w = ui.available_width();
                                                        let tex_size = texture.size_vec2();
                                                        if tex_size.x > 0.0 && tex_size.y > 0.0 {
                                                            let aspect = tex_size.y / tex_size.x;
                                                            let target_w = (avail_w - 14.0).clamp(180.0, 480.0);
                                                            let target_h = target_w * aspect;
                                                            let final_size = egui::vec2(target_w, target_h);

                                                            ui.vertical_centered(|ui| {
                                                                let (rect, response) = ui.allocate_exact_size(final_size, egui::Sense::click_and_drag());

                                                                // Draw camera frame
                                                                ui.painter().image(
                                                                    texture.id(),
                                                                    rect,
                                                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                                                    egui::Color32::WHITE,
                                                                );

                                                                let hover_pos = response.hover_pos();
                                                                if response.clicked() {
                                                                    if let Some(pos) = hover_pos {
                                                                        let u = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0) as f64;
                                                                        let v = ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0) as f64;
                                                                        self.wizard_points[self.wizard_target_idx] = Some((u, v));
                                                                        self.wizard_target_idx = (self.wizard_target_idx + 1).min(3);
                                                                    }
                                                                }

                                                                // Draw placed target markers (1, 2, 3, 4)
                                                                for (i, opt_pt) in self.wizard_points.iter().enumerate() {
                                                                    if let Some((u, v)) = *opt_pt {
                                                                        let px = rect.left() + rect.width() * u as f32;
                                                                        let py = rect.top() + rect.height() * v as f32;
                                                                        let marker_pos = egui::pos2(px, py);
                                                                        let (_, col) = target_labels[i];

                                                                        ui.painter().circle_filled(marker_pos, 10.0, col);
                                                                        ui.painter().circle_stroke(marker_pos, 10.0, egui::Stroke::new(1.5_f32, egui::Color32::BLACK));
                                                                        ui.painter().text(
                                                                            marker_pos,
                                                                            egui::Align2::CENTER_CENTER,
                                                                            format!("{}", i + 1),
                                                                            egui::FontId::proportional(13.0),
                                                                            egui::Color32::BLACK,
                                                                        );
                                                                    }
                                                                }

                                                                // Draw Floating Zoom Loupe (Magnifier) near cursor
                                                                if let Some(pos) = hover_pos {
                                                                    if rect.contains(pos) {
                                                                        let u = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                                                                        let v = ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);

                                                                        let loupe_size = 130.0_f32;
                                                                        let zoom = self.wizard_magnifier_zoom.clamp(2.0, 8.0);
                                                                        let half_uv = 0.5 / zoom;

                                                                        let uv_min = egui::pos2((u - half_uv).clamp(0.0, 1.0), (v - half_uv).clamp(0.0, 1.0));
                                                                        let uv_max = egui::pos2((u + half_uv).clamp(0.0, 1.0), (v + half_uv).clamp(0.0, 1.0));

                                                                        // Position loupe away from cursor
                                                                        let loupe_x = if pos.x < rect.center().x {
                                                                            rect.right() - loupe_size - 8.0
                                                                        } else {
                                                                            rect.left() + 8.0
                                                                        };
                                                                        let loupe_y = rect.top() + 8.0;
                                                                        let loupe_rect = egui::Rect::from_min_size(egui::pos2(loupe_x, loupe_y), egui::vec2(loupe_size, loupe_size));

                                                                        ui.painter().rect_filled(loupe_rect, 6.0, egui::Color32::BLACK);
                                                                        ui.painter().image(
                                                                            texture.id(),
                                                                            loupe_rect,
                                                                            egui::Rect::from_min_max(uv_min, uv_max),
                                                                            egui::Color32::WHITE,
                                                                        );
                                                                        ui.painter().rect_stroke(loupe_rect, 6.0, egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(0, 230, 255)), egui::StrokeKind::Outside);

                                                                        let loupe_center = loupe_rect.center();
                                                                        let cross_stroke = egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(255, 30, 30));
                                                                        ui.painter().line_segment([egui::pos2(loupe_center.x - 16.0, loupe_center.y), egui::pos2(loupe_center.x + 16.0, loupe_center.y)], cross_stroke);
                                                                        ui.painter().line_segment([egui::pos2(loupe_center.x, loupe_center.y - 16.0), egui::pos2(loupe_center.x, loupe_center.y + 16.0)], cross_stroke);
                                                                        ui.painter().circle_stroke(loupe_center, 4.0, cross_stroke);
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    } else {
                                                        ui.vertical_centered(|ui| {
                                                            ui.add_space(30.0);
                                                            ui.label("Camera stream is currently IDLE. Click 'Start Stream' above to view feed.");
                                                            ui.add_space(30.0);
                                                        });
                                                    }

                                                    ui.add_space(6.0);
                                                    ui.horizontal(|ui| {
                                                        ui.label("Loupe Zoom:");
                                                        ui.add(egui::Slider::new(&mut self.wizard_magnifier_zoom, 2.0..=8.0).custom_formatter(|v, _| format!("{:.1}x", v)));
                                                        if ui.button("Clear Points").clicked() {
                                                            self.wizard_points = [None; 4];
                                                            self.wizard_target_idx = 0;
                                                        }
                                                    });

                                                    ui.add_space(8.0);
                                                    let all_set = self.wizard_points.iter().all(|p| p.is_some());
                                                    if all_set {
                                                        let apply_btn = egui::Button::new(
                                                            egui::RichText::new("Compute & Apply Homography Alignment")
                                                                .color(egui::Color32::from_rgb(0, 240, 0))
                                                                .strong(),
                                                        );
                                                        if ui.add(apply_btn).clicked() {
                                                            let mut pts = [crate::camera::AlignmentPoint { machine_x: 0.0, machine_y: 0.0, camera_u: 0.0, camera_v: 0.0 }; 4];
                                                            for i in 0..4 {
                                                                let (x, y) = targets[i];
                                                                let (u, v) = self.wizard_points[i].unwrap();
                                                                pts[i] = crate::camera::AlignmentPoint { machine_x: x, machine_y: y, camera_u: u, camera_v: v };
                                                            }
                                                            match self.camera_calibration.update_homography(&pts) {
                                                                Ok(()) => {
                                                                    let _ = self.camera_calibration.save();
                                                                    self.camera_calibration.show_on_canvas = true;
                                                                    self.wizard_step = 3;
                                                                    self.status_msg = "Homography alignment solved & saved! Bed overlay is now active.".to_string();
                                                                }
                                                                Err(e) => {
                                                                    self.status_msg = format!("Alignment calculation error: {}", e);
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        let missing_count = self.wizard_points.iter().filter(|p| p.is_none()).count();
                                                        ui.label(egui::RichText::new(format!("Click {} more target(s) on the camera image above to complete.", missing_count)).color(egui::Color32::YELLOW));
                                                    }

                                                    ui.add_space(8.0);
                                                    ui.horizontal(|ui| {
                                                        if ui.button("<- Back to Burn").clicked() {
                                                            self.wizard_step = 1;
                                                        }
                                                    });
                                                }
                                                3 => {
                                                    // STEP 3: COMPLETED
                                                    ui.vertical_centered(|ui| {
                                                        ui.add_space(10.0);
                                                        ui.heading(egui::RichText::new("4-Point Alignment Complete!").color(egui::Color32::from_rgb(0, 240, 0)));
                                                        ui.label("The camera is now mathematically aligned to your laser bed coordinates using perspective homography.");
                                                        ui.label("The background image under the grid now exactly matches your physical workpiece.");
                                                        ui.add_space(10.0);

                                                        if ui.button(egui::RichText::new("Recalibrate / Start Over").strong()).clicked() {
                                                            self.wizard_step = 0;
                                                        }
                                                    });
                                                }
                                                _ => {}
                                            }
                                        }
                                        1 => {
                                            // -------------------------------------------------
                                            // TAB 1: BED ALIGNMENT (MANUAL SLIDERS)
                                            // -------------------------------------------------
                                            ui.heading("Bed Alignment (Manual Sliders)");
                                            ui.label(egui::RichText::new("Manual affine controls (offset, scale, rotation).").small().weak());
                                            ui.add_space(6.0);

                                            if self.camera_calibration.use_homography {
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("[Active] 4-Point Homography").color(egui::Color32::from_rgb(0, 230, 0)).strong());
                                                    if ui.button("Switch to Manual Mode").clicked() {
                                                        self.camera_calibration.use_homography = false;
                                                    }
                                                });
                                                ui.add_space(6.0);
                                            }

                                            ui.label(egui::RichText::new("Separate Zooms X & Y (Image size on laser bed):").strong());

                                            ui.horizontal(|ui| {
                                                ui.label("Zoom X (Width):");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.scale_x).speed(0.005).range(0.1..=4.0));
                                            });
                                            ui.add(egui::Slider::new(&mut self.camera_calibration.scale_x, 0.2..=3.0)
                                                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                                                .text("X Width"));

                                            ui.horizontal(|ui| {
                                                ui.label("Zoom Y (Height):");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.scale_y).speed(0.005).range(0.1..=4.0));
                                            });
                                            ui.add(egui::Slider::new(&mut self.camera_calibration.scale_y, 0.2..=3.0)
                                                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                                                .text("Y Height"));

                                            ui.horizontal(|ui| {
                                                ui.label("Uniform Zoom (Both):");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.uniform_scale).speed(0.005).range(0.1..=4.0));
                                                if ui.button("Reset 1:1").clicked() {
                                                    self.camera_calibration.scale_x = 1.0;
                                                    self.camera_calibration.scale_y = 1.0;
                                                    self.camera_calibration.uniform_scale = 1.0;
                                                }
                                            });

                                            ui.add_space(8.0);
                                            egui::Grid::new("bed_align_compact_grid")
                                                .spacing([12.0, 10.0])
                                                .show(ui, |ui| {
                                                    ui.label("Offset X (mm):");
                                                    ui.add(egui::DragValue::new(&mut self.camera_calibration.offset_x_mm).speed(0.5).range(-500.0..=500.0));
                                                    ui.end_row();

                                                    ui.label("Offset Y (mm):");
                                                    ui.add(egui::DragValue::new(&mut self.camera_calibration.offset_y_mm).speed(0.5).range(-500.0..=500.0));
                                                    ui.end_row();

                                                    ui.label("Rotation (°):");
                                                    ui.add(egui::DragValue::new(&mut self.camera_calibration.rotation_deg).speed(0.5).range(-180.0..=180.0));
                                                    ui.end_row();
                                                });

                                            ui.add_space(6.0);
                                            ui.horizontal(|ui| {
                                                ui.checkbox(&mut self.camera_calibration.flip_h, "Flip Horizontal");
                                                ui.checkbox(&mut self.camera_calibration.flip_v, "Flip Vertical");
                                            });

                                            ui.add_space(8.0);
                                            ui.horizontal(|ui| {
                                                if ui.button("Fit to Bed").on_hover_text("Fit camera image directly to bed dimensions").clicked() {
                                                    self.camera_calibration.reset_alignment();
                                                }
                                                if ui.button("Reset Alignment").clicked() {
                                                    self.camera_calibration.reset_alignment();
                                                }
                                            });
                                        }
                                        2 => {
                                            // -------------------------------------------------
                                            // TAB 2: FISHEYE LENS & PREVIEW
                                            // -------------------------------------------------
                                            if let Some(ref texture) = self.camera_texture {
                                                let avail_w = ui.available_width();
                                                let tex_size = texture.size_vec2();
                                                if tex_size.x > 0.0 && tex_size.y > 0.0 {
                                                    let aspect = tex_size.y / tex_size.x;
                                                    let target_w = (avail_w - 14.0).clamp(160.0, 460.0);
                                                    let target_h = target_w * aspect;
                                                    let final_size = egui::vec2(target_w, target_h);

                                                    ui.vertical_centered(|ui| {
                                                        let (rect, _response) = ui.allocate_exact_size(final_size, egui::Sense::hover());

                                                        // Draw undistorted live image
                                                        ui.painter().image(
                                                            texture.id(),
                                                            rect,
                                                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                                            egui::Color32::WHITE,
                                                        );

                                                        // Straight alignment guide grid overlay
                                                        if self.camera_calibration.show_guide_grid {
                                                            let guide_color = egui::Color32::from_rgba_unmultiplied(0, 230, 255, 150);
                                                            let stroke = egui::Stroke::new(1.0_f32, guide_color);
                                                            for i in 1..6 {
                                                                let t = i as f32 / 6.0;
                                                                let x = rect.left() + rect.width() * t;
                                                                let y = rect.top() + rect.height() * t;
                                                                ui.painter().line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], stroke);
                                                                ui.painter().line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], stroke);
                                                            }
                                                        }
                                                    });
                                                }
                                            } else {
                                                ui.vertical_centered(|ui| {
                                                    ui.add_space(40.0);
                                                    ui.heading("No Video Stream");
                                                    ui.label("Select a camera above and click 'Start Stream'.");
                                                    ui.add_space(40.0);
                                                });
                                            }

                                            ui.add_space(6.0);
                                            ui.checkbox(&mut self.camera_calibration.show_guide_grid, "Show Alignment Guide Grid");

                                            ui.add_space(6.0);
                                            ui.separator();
                                            ui.label(egui::RichText::new("Fisheye Lens Correction (drag slider left into negative):").strong());

                                            ui.horizontal(|ui| {
                                                ui.label("Distortion (k1):");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.fisheye_k1).speed(0.005).range(-1.0..=1.0));
                                            });
                                            ui.add(egui::Slider::new(&mut self.camera_calibration.fisheye_k1, -1.0..=1.0).show_value(false));

                                            ui.horizontal(|ui| {
                                                ui.label("Fine-tune (k2):");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.fisheye_k2).speed(0.002).range(-0.5..=0.5));
                                            });
                                            ui.add(egui::Slider::new(&mut self.camera_calibration.fisheye_k2, -0.5..=0.5).show_value(false));

                                            ui.horizontal(|ui| {
                                                ui.label("Lens Zoom X:");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.lens_zoom_x).speed(0.005).range(0.2..=4.0));
                                                ui.label("Y:");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.lens_zoom_y).speed(0.005).range(0.2..=4.0));
                                            });

                                            ui.add_space(4.0);
                                            ui.label(egui::RichText::new("Optical Center (extended Y range):").strong());
                                            ui.horizontal(|ui| {
                                                ui.label("Center X:");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.center_x).speed(0.005).range(-2.0..=2.0));
                                                ui.label("Center Y:");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.center_y).speed(0.005).range(-2.0..=2.0));
                                            });
                                            ui.add(egui::Slider::new(&mut self.camera_calibration.center_y, -1.5..=1.5).text("Y Center Slider"));

                                            ui.add_space(6.0);
                                            ui.separator();
                                            ui.label(egui::RichText::new("Pure Zoom (Separate X & Y):").strong());
                                            ui.label(egui::RichText::new("Scale image closer / farther without changing lens distortion:").small().weak());

                                            ui.horizontal(|ui| {
                                                ui.label("Zoom X (Width):");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.scale_x).speed(0.005).range(0.1..=4.0));
                                            });
                                            ui.add(egui::Slider::new(&mut self.camera_calibration.scale_x, 0.2..=3.0)
                                                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                                                .text("X Width"));

                                            ui.horizontal(|ui| {
                                                ui.label("Zoom Y (Height):");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.scale_y).speed(0.005).range(0.1..=4.0));
                                            });
                                            ui.add(egui::Slider::new(&mut self.camera_calibration.scale_y, 0.2..=3.0)
                                                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                                                .text("Y Height"));

                                            ui.horizontal(|ui| {
                                                ui.label("Uniform Zoom (Both):");
                                                ui.add(egui::DragValue::new(&mut self.camera_calibration.uniform_scale).speed(0.005).range(0.1..=4.0));
                                                if ui.button("Reset 1:1").clicked() {
                                                    self.camera_calibration.scale_x = 1.0;
                                                    self.camera_calibration.scale_y = 1.0;
                                                    self.camera_calibration.uniform_scale = 1.0;
                                                }
                                            });

                                            ui.add_space(4.0);
                                            if ui.button("Reset Lens Distortion").clicked() {
                                                self.camera_calibration.reset_lens();
                                            }
                                        }
                                        _ => {
                                            // -------------------------------------------------
                                            // TAB 3: CANVAS OVERLAY SETTINGS
                                            // -------------------------------------------------
                                            ui.heading("Bed Canvas Background");
                                            ui.add_space(6.0);

                                            ui.checkbox(&mut self.camera_calibration.show_on_canvas, "Show camera image as laser bed background overlay");
                                            ui.label(egui::RichText::new("Shortcut: Cmd+Shift+C toggles canvas overlay on/off anytime.").small().weak());

                                            ui.add_space(8.0);
                                            ui.horizontal(|ui| {
                                                ui.label("Opacity:");
                                                ui.add(egui::Slider::new(&mut self.camera_calibration.opacity, 0.05..=1.0)
                                                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)));
                                            });

                                            ui.add_space(10.0);
                                            ui.label(egui::RichText::new("Tip: With the background overlay enabled, you can place your physical material on the laser bed and visually position cut/engrave vectors directly over it with millimeter precision!").small().color(egui::Color32::from_rgb(220, 200, 140)));
                                        }
                                    }
                                });
                        });
                });

            self.show_camera_window = is_open;
        }
    }
}

#[cfg(all(test, feature = "gui"))]
mod tests {
    use super::*;

    #[test]
    fn test_copy_paste_single_and_cascading() {
        let mut app = LaserBurnApp::default();
        app.doc.shapes.clear();
        app.selected_shape_indices.clear();
        app.clipboard_shapes.clear();

        app.doc.add_shape(Shape::Rect {
            layer_index: 0,
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
            corner_radius: 0.0,
        });
        assert_eq!(app.doc.shapes.len(), 1);

        app.selected_shape_indices = vec![0];
        app.copy_selected();
        assert_eq!(app.clipboard_shapes.len(), 1);
        assert_eq!(app.doc.shapes.len(), 1);

        // First paste (+5mm)
        app.paste();
        assert_eq!(app.doc.shapes.len(), 2);
        assert_eq!(app.selected_shape_indices, vec![1]);
        if let Shape::Rect { x, y, width, height, .. } = &app.doc.shapes[1] {
            assert_eq!(*x, 15.0);
            assert_eq!(*y, 15.0);
            assert_eq!(*width, 20.0);
            assert_eq!(*height, 20.0);
        } else {
            panic!("Expected pasted shape to be Rect");
        }

        // Second paste (cascades +5mm more -> +10mm from original)
        app.paste();
        assert_eq!(app.doc.shapes.len(), 3);
        assert_eq!(app.selected_shape_indices, vec![2]);
        if let Shape::Rect { x, y, .. } = &app.doc.shapes[2] {
            assert_eq!(*x, 20.0);
            assert_eq!(*y, 20.0);
        } else {
            panic!("Expected second pasted shape to be Rect at (20, 20)");
        }
    }

    #[test]
    fn test_cut_and_paste() {
        let mut app = LaserBurnApp::default();
        app.doc.shapes.clear();
        app.selected_shape_indices.clear();
        app.clipboard_shapes.clear();

        app.doc.add_shape(Shape::Rect {
            layer_index: 0,
            x: 50.0,
            y: 50.0,
            width: 30.0,
            height: 30.0,
            corner_radius: 0.0,
        });

        app.selected_shape_indices = vec![0];
        app.cut_selected();

        assert_eq!(app.doc.shapes.len(), 0);
        assert!(app.selected_shape_indices.is_empty());
        assert_eq!(app.clipboard_shapes.len(), 1);

        app.paste();
        assert_eq!(app.doc.shapes.len(), 1);
        assert_eq!(app.selected_shape_indices, vec![0]);
        if let Shape::Rect { x, y, .. } = &app.doc.shapes[0] {
            assert_eq!(*x, 55.0);
            assert_eq!(*y, 55.0);
        } else {
            panic!("Expected pasted shape to be Rect");
        }
    }

    #[test]
    fn test_duplicate() {
        let mut app = LaserBurnApp::default();
        app.doc.shapes.clear();
        app.selected_shape_indices.clear();

        app.doc.add_shape(Shape::Circle {
            layer_index: 1,
            center: Point2D::new(100.0, 100.0),
            radius: 20.0,
        });

        app.selected_shape_indices = vec![0];
        app.duplicate_selected();

        assert_eq!(app.doc.shapes.len(), 2);
        assert_eq!(app.selected_shape_indices, vec![1]);
        if let Shape::Circle { center, radius, layer_index } = &app.doc.shapes[1] {
            assert_eq!(center.x, 105.0);
            assert_eq!(center.y, 105.0);
            assert_eq!(*radius, 20.0);
            assert_eq!(*layer_index, 1);
        } else {
            panic!("Expected duplicated shape to be Circle");
        }
    }

    #[test]
    fn test_multi_selection_copy_paste_delete() {
        let mut app = LaserBurnApp::default();
        app.doc.shapes.clear();
        app.selected_shape_indices.clear();

        app.doc.add_shape(Shape::Rect {
            layer_index: 0,
            x: 10.0,
            y: 10.0,
            width: 10.0,
            height: 10.0,
            corner_radius: 0.0,
        });
        app.doc.add_shape(Shape::Circle {
            layer_index: 0,
            center: Point2D::new(40.0, 40.0),
            radius: 5.0,
        });

        app.selected_shape_indices = vec![0, 1];
        app.copy_selected();
        assert_eq!(app.clipboard_shapes.len(), 2);

        app.paste();
        assert_eq!(app.doc.shapes.len(), 4);
        assert_eq!(app.selected_shape_indices, vec![2, 3]);

        app.delete_selected();
        assert_eq!(app.doc.shapes.len(), 2);
        assert!(app.selected_shape_indices.is_empty());
    }

    #[test]
    fn test_locked_shape_protection() {
        let mut app = LaserBurnApp::default();
        app.doc.shapes.clear();
        app.selected_shape_indices.clear();

        // Add a locked group shape
        app.doc.add_shape(Shape::Group {
            layer_index: 0,
            shapes: vec![Shape::Rect {
                layer_index: 0,
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
                corner_radius: 0.0,
            }],
            locked: true,
        });

        // Add an unlocked shape
        app.doc.add_shape(Shape::Circle {
            layer_index: 0,
            center: Point2D::new(50.0, 50.0),
            radius: 10.0,
        });

        // Select both and try to delete
        app.selected_shape_indices = vec![0, 1];
        app.delete_selected();

        // Locked shape (index 0) must remain!
        assert_eq!(app.doc.shapes.len(), 1);
        assert!(app.doc.shapes[0].is_locked());

        // Copy locked shape and paste -> new shape must be unlocked!
        app.selected_shape_indices = vec![0];
        app.copy_selected();
        app.paste();

        assert_eq!(app.doc.shapes.len(), 2);
        assert!(!app.doc.shapes[1].is_locked());
    }

    #[test]
    fn test_rotate_selected_and_flip() {
        let mut app = LaserBurnApp::default();
        app.doc.shapes.clear();
        app.selected_shape_indices.clear();

        app.doc.add_shape(Shape::Rect {
            layer_index: 0,
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
            corner_radius: 0.0,
        });

        app.selected_shape_indices = vec![0];

        // Rotate +1.0 degree
        app.rotate_selected(1.0);
        let b1 = app.doc.shapes[0].bounds();
        let c1 = b1.center();
        assert!((c1.x - 20.0).abs() < 1e-4);
        assert!((c1.y - 20.0).abs() < 1e-4);

        // Rotate back -1.0 degree
        app.rotate_selected(-1.0);
        let b2 = app.doc.shapes[0].bounds();
        let c2 = b2.center();
        assert!((c2.x - 20.0).abs() < 1e-4);
        assert!((c2.y - 20.0).abs() < 1e-4);

        // Flip H and Flip V
        app.flip_selected_h();
        app.flip_selected_v();
        let b3 = app.doc.shapes[0].bounds();
        let c3 = b3.center();
        assert!((c3.x - 20.0).abs() < 1e-4);
        assert!((c3.y - 20.0).abs() < 1e-4);
    }

    #[test]
    fn test_adaptive_grid_steps_for_zoom() {
        // Zoomed out far: 50mm minor, 100mm major, 100mm label
        assert_eq!(LaserBurnApp::grid_steps_for_zoom(0.4), (50.0, 100.0, 100.0));

        // Normal fit / compact zoom (e.g. 0.8 - 1.2): 10mm minor, 50mm major, 50mm label
        assert_eq!(LaserBurnApp::grid_steps_for_zoom(1.0), (10.0, 50.0, 50.0));

        // Slight zoom-in (e.g. 1.5 - 2.5): 5mm minor, 20mm major, 20mm label
        assert_eq!(LaserBurnApp::grid_steps_for_zoom(1.8), (5.0, 20.0, 20.0));

        // Medium close-up (e.g. 3.0 - 5.0): 2mm minor, 10mm major, 10mm label
        assert_eq!(LaserBurnApp::grid_steps_for_zoom(3.5), (2.0, 10.0, 10.0));

        // High detail (e.g. 6.0 - 9.0): 1mm minor, 5mm major, 5mm label
        assert_eq!(LaserBurnApp::grid_steps_for_zoom(7.0), (1.0, 5.0, 5.0));

        // Micro detail (>= 10.0): 0.5mm minor, 2mm major, 2mm label
        assert_eq!(LaserBurnApp::grid_steps_for_zoom(12.0), (0.5, 2.0, 2.0));
    }

    #[test]
    fn test_async_file_dialog_actions() {
        let app = LaserBurnApp::default();
        let tmp_dir = std::env::temp_dir();
        let save_target = tmp_dir.join("test_save_project.lbrn2");

        // Send a SaveProject action through the channel
        app.file_dialog_tx
            .send(FileDialogAction::SaveProject(save_target.clone()))
            .unwrap();

        // Verify channel receives and handles SaveProject safely
        if let Ok(FileDialogAction::SaveProject(path)) = app.file_dialog_rx.try_recv() {
            let save_path = if path.extension().is_none() {
                path.with_extension("lbrn2")
            } else {
                path
            };
            assert!(Lbrn2File::save_to_file(&app.doc, &save_path).is_ok());
            assert!(save_path.exists());
            let _ = std::fs::remove_file(&save_path);
        } else {
            panic!("Expected SaveProject action from file_dialog_rx");
        }
    }

    #[test]
    fn test_alignment_wizard_targets_and_gcode() {
        let mut app = LaserBurnApp::default();
        app.doc.width = 465.0;
        app.doc.height = 440.0;
        app.camera_calibration.target_inset_mm = 50.0;

        let targets = app.alignment_target_doc_coords();
        assert_eq!(targets.len(), 4);
        assert_eq!(targets[0], (50.0, 50.0));
        assert_eq!(targets[1], (415.0, 50.0));
        assert_eq!(targets[2], (415.0, 390.0));
        assert_eq!(targets[3], (50.0, 390.0));

        let gcode = app.generate_alignment_targets_gcode();
        assert!(gcode.contains("G21"));
        assert!(gcode.contains("G90"));
        assert!(gcode.contains("M3"));
        assert!(gcode.contains("M5"));
        assert!(gcode.contains("G1")); // Crosshairs
        assert!(gcode.contains("G2")); // Bullseye circles

        // Test homography solving with these targets
        let pts = [
            crate::camera::AlignmentPoint { machine_x: targets[0].0, machine_y: targets[0].1, camera_u: 0.1, camera_v: 0.1 },
            crate::camera::AlignmentPoint { machine_x: targets[1].0, machine_y: targets[1].1, camera_u: 0.9, camera_v: 0.1 },
            crate::camera::AlignmentPoint { machine_x: targets[2].0, machine_y: targets[2].1, camera_u: 0.9, camera_v: 0.9 },
            crate::camera::AlignmentPoint { machine_x: targets[3].0, machine_y: targets[3].1, camera_u: 0.1, camera_v: 0.9 },
        ];
        assert!(app.camera_calibration.update_homography(&pts).is_ok());
        assert!(app.camera_calibration.use_homography);
        assert!(app.camera_calibration.inv_homography_matrix.is_some());
    }
}
