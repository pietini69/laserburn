use eframe::egui;
use super::{ActiveTool, AlignMode, LaserBurnApp};
use crate::Shape;
use crate::geometry::{BooleanOp, Point2D};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    Select,  // Valinta, Solmueditori, Mittaus, Leikepöytä, Käännöt
    Shapes,  // Muodot (Neliö, Ympyrä, Viiva, Teksti, Gear, Trace)
    Camera,  // Kamera (Alustakuva, Tausta ON/OFF, Kalibrointi, Kohdistus laatalle)
    Cam,     // CAM & Booleans (Offset, Ruudukot, Weld, Nesting, Materiaalit)
    Machine, // Laserohjaus (START/STOP, Kehystys, Kohdistusvalo, Koti, Jog)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteIcon {
    Select,
    NodeEdit,
    Rect,
    Circle,
    Line,
    Text,
    Camera,
    Cam,
    LaserAim,
    LaserRun,
    Import,
}

/// Helper function to draw crisp, vector-based tool icons that NEVER render as empty tofu squares
fn draw_vector_icon(painter: &egui::Painter, center: egui::Pos2, icon: PaletteIcon, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.8_f32, color);
    match icon {
        PaletteIcon::Select => {
            // Crisp selection pointer arrow
            let tip = center + egui::vec2(-6.0_f32, -7.0_f32);
            let pts = vec![
                tip,
                tip + egui::vec2(0.0_f32, 15.0_f32),
                tip + egui::vec2(4.0_f32, 11.0_f32),
                tip + egui::vec2(8.5_f32, 15.5_f32),
                tip + egui::vec2(11.0_f32, 13.0_f32),
                tip + egui::vec2(6.5_f32, 8.5_f32),
                tip + egui::vec2(12.0_f32, 6.0_f32),
            ];
            painter.add(egui::Shape::convex_polygon(pts, color, egui::Stroke::NONE));
        }
        PaletteIcon::NodeEdit => {
            // Path node editor: outline box with 4 corner anchor nodes
            let rect = egui::Rect::from_center_size(center, egui::vec2(13.0_f32, 13.0_f32));
            painter.rect_stroke(rect, 0.0_f32, egui::Stroke::new(1.3_f32, color), egui::StrokeKind::Middle);
            for corner in [rect.left_top(), rect.right_top(), rect.left_bottom(), rect.right_bottom()] {
                painter.rect_filled(egui::Rect::from_center_size(corner, egui::vec2(4.0_f32, 4.0_f32)), 0.5_f32, color);
            }
        }
        PaletteIcon::Rect => {
            painter.rect_stroke(egui::Rect::from_center_size(center, egui::vec2(16.0_f32, 12.0_f32)), 1.0_f32, stroke, egui::StrokeKind::Middle);
        }
        PaletteIcon::Circle => {
            painter.circle_stroke(center, 7.5_f32, stroke);
        }
        PaletteIcon::Line => {
            painter.line_segment([center + egui::vec2(-7.5_f32, 7.0_f32), center + egui::vec2(7.5_f32, -7.0_f32)], egui::Stroke::new(2.0_f32, color));
        }
        PaletteIcon::Text => {
            painter.text(center, egui::Align2::CENTER_CENTER, "T", egui::FontId::proportional(21.0_f32), color);
        }
        PaletteIcon::Camera => {
            // Camera body + lens circle + top viewfinder bump
            let body = egui::Rect::from_center_size(center + egui::vec2(0.0_f32, 1.5_f32), egui::vec2(16.0_f32, 11.0_f32));
            painter.rect_stroke(body, 2.0_f32, egui::Stroke::new(1.6_f32, color), egui::StrokeKind::Middle);
            let top_bump = egui::Rect::from_min_size(body.min + egui::vec2(4.0_f32, -3.0_f32), egui::vec2(5.0_f32, 3.5_f32));
            painter.rect_filled(top_bump, 1.0_f32, color);
            painter.circle_stroke(body.center(), 3.5_f32, egui::Stroke::new(1.5_f32, color));
        }
        PaletteIcon::Cam => {
            // 2x2 CAM matrix / grid
            let r = egui::Rect::from_center_size(center, egui::vec2(14.0_f32, 14.0_f32));
            painter.rect_stroke(r, 1.0_f32, egui::Stroke::new(1.4_f32, color), egui::StrokeKind::Middle);
            painter.line_segment([egui::pos2(r.center().x, r.min.y), egui::pos2(r.center().x, r.max.y)], egui::Stroke::new(1.2_f32, color));
            painter.line_segment([egui::pos2(r.min.x, r.center().y), egui::pos2(r.max.x, r.center().y)], egui::Stroke::new(1.2_f32, color));
        }
        PaletteIcon::LaserAim => {
            // Laser crosshairs / position indicator: reticle circle + 4 cross ticks
            painter.circle_stroke(center, 6.5_f32, egui::Stroke::new(1.5_f32, color));
            painter.line_segment([center + egui::vec2(0.0_f32, -9.0_f32), center + egui::vec2(0.0_f32, -4.5_f32)], egui::Stroke::new(1.5_f32, color));
            painter.line_segment([center + egui::vec2(0.0_f32, 4.5_f32), center + egui::vec2(0.0_f32, 9.0_f32)], egui::Stroke::new(1.5_f32, color));
            painter.line_segment([center + egui::vec2(-9.0_f32, 0.0_f32), center + egui::vec2(-4.5_f32, 0.0_f32)], egui::Stroke::new(1.5_f32, color));
            painter.line_segment([center + egui::vec2(4.5_f32, 0.0_f32), center + egui::vec2(9.0_f32, 0.0_f32)], egui::Stroke::new(1.5_f32, color));
            painter.circle_filled(center, 1.6_f32, color);
        }
        PaletteIcon::LaserRun => {
            // Crisp Play / Run triangle
            let p1 = center + egui::vec2(-5.0_f32, -7.5_f32);
            let p2 = center + egui::vec2(7.5_f32, 0.0_f32);
            let p3 = center + egui::vec2(-5.0_f32, 7.5_f32);
            painter.add(egui::Shape::convex_polygon(vec![p1, p2, p3], color, egui::Stroke::NONE));
        }
        PaletteIcon::Import => {
            // Folder icon with top tab
            let body = egui::Rect::from_min_size(center + egui::vec2(-8.0_f32, -3.0_f32), egui::vec2(16.0_f32, 11.0_f32));
            painter.rect_stroke(body, 1.5_f32, egui::Stroke::new(1.5_f32, color), egui::StrokeKind::Middle);
            let tab = egui::Rect::from_min_size(center + egui::vec2(-8.0_f32, -6.5_f32), egui::vec2(7.0_f32, 4.0_f32));
            painter.rect_filled(tab, 1.0_f32, color);
        }
    }
}

impl LaserBurnApp {
    /// Renders the V2 Photoshop-style floating tool palette inside the canvas window
    pub fn render_tool_palette_v2(&mut self, ctx: &egui::Context) {
        if !self.show_tools_palette {
            return;
        }

        let is_dark = self.device.settings.dark_mode;

        // Strictly constrain the floating palette to 44 px width inside the main window
        let window = egui::Window::new("tools_palette_window")
            .id(egui::Id::new("floating_v2_tools_window"))
            .default_pos([14.0, 160.0])
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .fixed_size([44.0, 260.0])
            .frame(
                egui::Frame::window(ctx.style().as_ref())
                    .corner_radius(8.0)
                    .shadow(egui::Shadow {
                        offset: [0, 4],
                        blur: 10,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(if is_dark { 140 } else { 60 }),
                    })
                    .stroke(egui::Stroke::new(1.0_f32, if is_dark { egui::Color32::from_rgb(55, 60, 72) } else { egui::Color32::from_rgb(205, 210, 220) }))
                    .fill(if is_dark { egui::Color32::from_rgb(26, 28, 34) } else { egui::Color32::from_rgb(245, 246, 250) })
                    .inner_margin(egui::Margin::symmetric(4, 6))
            );

        window.show(ctx, |ui| {
            self.render_palette_vertical_buttons(ui, ctx);
        });

        // Render flyout submenu to the side of the active category button
        self.render_palette_flyout_card(ctx);
    }

    /// Renders the vertical button strip (strictly 36 px wide content)
    fn render_palette_vertical_buttons(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.set_width(36.0);
        let is_dark = self.device.settings.dark_mode;

        // Slim drag grip handle at the top (exact width 36 px, clean horizontal grip bars)
        let (grip_rect, grip_resp) = ui.allocate_exact_size(egui::vec2(36.0, 12.0), egui::Sense::hover());
        if grip_resp.hovered() {
            ui.ctx().set_cursor_icon(if ui.ctx().input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
                egui::CursorIcon::Grabbing
            } else {
                egui::CursorIcon::Grab
            });
        }
        let painter = ui.painter();
        let center = grip_rect.center();
        let grip_col = if is_dark { egui::Color32::from_gray(90) } else { egui::Color32::from_gray(160) };

        // 2 subtle horizontal grip lines
        painter.line_segment(
            [egui::pos2(center.x - 10.0, center.y - 2.0), egui::pos2(center.x + 10.0, center.y - 2.0)],
            egui::Stroke::new(1.5_f32, grip_col),
        );
        painter.line_segment(
            [egui::pos2(center.x - 10.0, center.y + 2.5), egui::pos2(center.x + 10.0, center.y + 2.5)],
            egui::Stroke::new(1.5_f32, grip_col),
        );

        ui.add_space(2.0);
        ui.separator();
        ui.add_space(2.0);

        // Category 1: Valinta & Solmut (Select / NodeEdit)
        let is_select_open = self.open_flyout == Some(ToolCategory::Select);
        let select_icon = if self.active_tool == ActiveTool::NodeEdit { PaletteIcon::NodeEdit } else { PaletteIcon::Select };
        if self.draw_palette_tool_btn(ui, select_icon, is_select_open, false, "Valintatyökalut, Muokkaus & Leikepöytä (Klikkaa avataksesi alityökalut)") {
            if self.active_tool != ActiveTool::Select && self.active_tool != ActiveTool::NodeEdit {
                self.active_tool = ActiveTool::Select;
            }
            self.toggle_flyout(ToolCategory::Select);
        }

        ui.add_space(2.0);

        // Category 2: Piirtotyökalut (Shapes: Rect, Circle, Line, Text)
        let is_shape_open = self.open_flyout == Some(ToolCategory::Shapes);
        let shape_icon = match self.active_tool {
            ActiveTool::DrawRect => PaletteIcon::Rect,
            ActiveTool::DrawCircle => PaletteIcon::Circle,
            ActiveTool::DrawLine => PaletteIcon::Line,
            ActiveTool::DrawText => PaletteIcon::Text,
            _ => PaletteIcon::Rect,
        };
        if self.draw_palette_tool_btn(ui, shape_icon, is_shape_open, false, "Muodot & Piirtotyökalut (Neliö, Ympyrä, Viiva, Teksti, Gear)") {
            self.toggle_flyout(ToolCategory::Shapes);
        }

        ui.add_space(2.0);

        // Category 3: Kamera & Kohdistus (Camera & Vision)
        let is_cam_capturing = self.oneshot_snapshot.is_some();
        let is_cam_showing = self.camera_calibration.show_on_canvas;
        let is_cam_open = self.open_flyout == Some(ToolCategory::Camera);
        let cam_icon = PaletteIcon::Camera;
        let cam_btn_tip = if is_cam_capturing {
            "Otetaan kuvaa alustasta... (Klikkaa avataksesi toiminnot)"
        } else if is_cam_showing {
            "Kamera: Taustakuva aktiivinen kankaalla (Klikkaa avataksesi toiminnot)"
        } else {
            "Kameran toiminnot & Alustan kohdistus (Klikkaa avataksesi toiminnot)"
        };
        // has_dot: green LED when camera bed overlay is showing
        if self.draw_palette_tool_btn(ui, cam_icon, is_cam_open, is_cam_showing, cam_btn_tip) {
            self.toggle_flyout(ToolCategory::Camera);
        }

        ui.add_space(2.0);

        // Category 4: Laser CAM & Täyttö (CAM: Hatch, Offset, Array, Materiaali)
        let is_cam_tool_open = self.open_flyout == Some(ToolCategory::Cam);
        if self.draw_palette_tool_btn(ui, PaletteIcon::Cam, is_cam_tool_open, false, "Laser CAM, Booleans & Materiaalit (Weld, Offset, Matriisit, Nesting)") {
            self.toggle_flyout(ToolCategory::Cam);
        }

        ui.add_space(2.0);

        // Category 5: Laserohjaus & Työn ajo (Machine, Jog & Job Run)
        let is_machine_open = self.open_flyout == Some(ToolCategory::Machine);
        let is_beam_on = self.device.is_weak_light_on;
        let is_job_running = self.device.is_job_running;
        let machine_icon = if is_job_running { PaletteIcon::LaserRun } else { PaletteIcon::LaserAim };
        let machine_tip = if is_job_running {
            "Leikkaustyö käynnissä! (Klikkaa avataksesi ohjauspaneeli ja keskeytys)"
        } else if is_beam_on {
            "Kohdistusvalo PÄÄLLÄ (Klikkaa ohjaukseen)"
        } else {
            "Laserin ohjaus, työn käynnistys & asemointi (Klikkaa avataksesi)"
        };
        if self.draw_palette_tool_btn(ui, machine_icon, is_machine_open, is_job_running, machine_tip) {
            self.toggle_flyout(ToolCategory::Machine);
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // File Import Shortcut: crisp vector folder icon
        let (import_rect, import_resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
        let painter = ui.painter();
        let is_hovered = import_resp.hovered();
        if is_hovered {
            let hover_bg = if is_dark {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 22)
            };
            painter.rect_filled(import_rect, 6.0, hover_bg);
        }
        let icon_color = if is_hovered {
            if is_dark { egui::Color32::WHITE } else { egui::Color32::BLACK }
        } else if is_dark {
            egui::Color32::from_rgb(215, 220, 230)
        } else {
            egui::Color32::from_rgb(50, 55, 65)
        };
        draw_vector_icon(&painter, import_rect.center(), PaletteIcon::Import, icon_color);

        if import_resp.on_hover_text("Tuo tiedosto (DXF, PDF, SVG, Kuvat)").clicked() {
            self.prompt_import_image(ctx);
        }
    }

    /// Draws a borderless, flat icon-only tool button (Photoshop style) with crisp vector glyph and flyout indicator
    fn draw_palette_tool_btn(
        &mut self,
        ui: &mut egui::Ui,
        icon: PaletteIcon,
        is_open: bool,
        has_dot: bool,
        tooltip: &str,
    ) -> bool {
        let btn_size = egui::vec2(36.0, 36.0);
        let is_dark = self.device.settings.dark_mode;

        let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
        let painter = ui.painter();
        let is_hovered = response.hovered();

        // Background highlight: flat transparent when idle, soft pill when hovered, solid accent ONLY when open!
        if is_open {
            let active_bg = if is_dark {
                egui::Color32::from_rgb(33, 115, 185)
            } else {
                egui::Color32::from_rgb(50, 130, 205)
            };
            painter.rect_filled(rect, 6.0, active_bg);
        } else if is_hovered {
            let hover_bg = if is_dark {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 22)
            };
            painter.rect_filled(rect, 6.0, hover_bg);
        }

        // Color for vector icon
        let icon_color = if is_open {
            egui::Color32::WHITE
        } else if is_hovered {
            if is_dark { egui::Color32::WHITE } else { egui::Color32::BLACK }
        } else if is_dark {
            egui::Color32::from_rgb(215, 220, 230)
        } else {
            egui::Color32::from_rgb(50, 55, 65)
        };

        // Draw the vector icon
        draw_vector_icon(&painter, rect.center(), icon, icon_color);

        // Status LED dot (e.g. green dot when camera overlay active, red dot when job running)
        if has_dot {
            let dot_col = if self.device.is_job_running && icon == PaletteIcon::LaserRun {
                egui::Color32::from_rgb(255, 60, 60)
            } else {
                egui::Color32::from_rgb(0, 220, 100)
            };
            painter.circle_filled(
                egui::pos2(rect.max.x - 5.0, rect.min.y + 5.0),
                2.5,
                dot_col,
            );
        }

        // Subtle micro-triangle in bottom-right corner for flyout indicator
        let triangle_color = if is_open {
            egui::Color32::from_white_alpha(210)
        } else if is_dark {
            egui::Color32::from_gray(110)
        } else {
            egui::Color32::from_gray(150)
        };

        let p1 = egui::pos2(rect.max.x - 2.5, rect.max.y - 2.5);
        let p2 = egui::pos2(rect.max.x - 6.5, rect.max.y - 2.5);
        let p3 = egui::pos2(rect.max.x - 2.5, rect.max.y - 6.5);
        painter.add(egui::Shape::convex_polygon(
            vec![p1, p2, p3],
            triangle_color,
            egui::Stroke::NONE,
        ));

        let response = response.on_hover_text(tooltip);

        if is_open {
            self.last_flyout_anchor = Some(rect);
        }

        if response.clicked() {
            self.last_flyout_anchor = Some(rect);
            true
        } else {
            false
        }
    }

    /// Toggles or opens flyout menu for a given category
    fn toggle_flyout(&mut self, cat: ToolCategory) {
        if self.open_flyout == Some(cat) {
            self.open_flyout = None;
        } else {
            self.open_flyout = Some(cat);
        }
    }

    /// Renders the floating card flyout menu positioned to the right of the active button
    fn render_palette_flyout_card(&mut self, ctx: &egui::Context) {
        let cat = match self.open_flyout {
            Some(c) => c,
            None => return,
        };

        let is_dark = self.device.settings.dark_mode;
        let anchor = self.last_flyout_anchor.unwrap_or(egui::Rect::from_min_size(egui::pos2(60.0, 160.0), egui::vec2(36.0, 36.0)));

        // Position flyout right next to the toolbar button
        let flyout_pos = egui::pos2(anchor.max.x + 8.0, anchor.min.y - 6.0);

        let window = egui::Window::new("palette_flyout_window")
            .id(egui::Id::new("floating_v2_flyout_window"))
            .fixed_pos(flyout_pos)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .frame(
                egui::Frame::window(ctx.style().as_ref())
                    .corner_radius(8.0)
                    .shadow(egui::Shadow {
                        offset: [2, 6],
                        blur: 16,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(if is_dark { 160 } else { 70 }),
                    })
                    .stroke(egui::Stroke::new(1.0_f32, if is_dark { egui::Color32::from_rgb(60, 66, 80) } else { egui::Color32::from_rgb(200, 205, 215) }))
                    .fill(if is_dark { egui::Color32::from_rgb(30, 33, 40) } else { egui::Color32::from_rgb(250, 251, 254) })
                    .inner_margin(egui::Margin::symmetric(10, 10))
            );

        let mut close_requested = false;

        window.show(ctx, |ui| {
            ui.set_min_width(220.0);
            ui.set_max_width(260.0);

            match cat {
                ToolCategory::Select => self.render_select_flyout_content(ui, ctx, &mut close_requested),
                ToolCategory::Shapes => self.render_shapes_flyout_content(ui, ctx, &mut close_requested),
                ToolCategory::Camera => self.render_camera_flyout_content(ui, ctx, &mut close_requested),
                ToolCategory::Cam => self.render_cam_flyout_content(ui, ctx, &mut close_requested),
                ToolCategory::Machine => self.render_machine_flyout_content(ui, ctx, &mut close_requested),
            }
        });

        if close_requested {
            self.open_flyout = None;
        }

        // Close on ESC
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.open_flyout = None;
        }
    }

    /// Category: Selection & Nodes flyout content
    fn render_select_flyout_content(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Valintatyökalut & Muokkaus").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Työkalutilat
        ui.horizontal(|ui| {
            if ui.selectable_value(&mut self.active_tool, ActiveTool::Select, "Valintatyökalu (S)").clicked() {
                *close_requested = true;
            }
            if ui.selectable_value(&mut self.active_tool, ActiveTool::NodeEdit, "Solmumuokkaus (N)").clicked() {
                *close_requested = true;
            }
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Leikepöytä & Poisto
        let has_sel = !self.selected_shape_indices.is_empty();
        let has_clip = !self.clipboard_shapes.is_empty();

        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 4.0) * 0.5;
            if ui.add_enabled_ui(has_sel, |ui| {
                if ui.add_sized([half_w, 24.0], egui::Button::new("Kopioi (Cmd+C)")).clicked() {
                    self.copy_selected();
                }
            }).response.clicked() {}

            if ui.add_enabled_ui(has_clip, |ui| {
                if ui.add_sized([half_w, 24.0], egui::Button::new("Liitä (Cmd+V)")).clicked() {
                    self.paste();
                }
            }).response.clicked() {}
        });

        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 4.0) * 0.5;
            if ui.add_enabled_ui(has_sel, |ui| {
                if ui.add_sized([half_w, 24.0], egui::Button::new("Kahdenna (Cmd+D)")).clicked() {
                    self.duplicate_selected();
                }
            }).response.clicked() {}

            if ui.add_enabled_ui(has_sel, |ui| {
                if ui.add_sized([half_w, 24.0], egui::Button::new("Poista (Del)")).clicked() {
                    self.delete_selected();
                }
            }).response.clicked() {}
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Ryhmittely & Lukitus
        let multi_sel = self.selected_shape_indices.len() >= 2;
        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 4.0) * 0.5;
            if ui.add_enabled_ui(multi_sel, |ui| {
                if ui.add_sized([half_w, 24.0], egui::Button::new("Ryhmittele (Cmd+G)")).clicked() {
                    self.group_selected();
                }
            }).response.clicked() {}

            if ui.add_enabled_ui(has_sel, |ui| {
                if ui.add_sized([half_w, 24.0], egui::Button::new("Pura ryhmä (Cmd+U)")).clicked() {
                    self.ungroup_selected();
                }
            }).response.clicked() {}
        });

        if has_sel {
            let is_locked = self.selected_shape_indices.iter().any(|&i| i < self.doc.shapes.len() && self.doc.shapes[i].is_locked());
            let lock_lbl = if is_locked { "Avaa lukitus (Cmd+L)" } else { "Lukitse valinta (Cmd+L)" };
            if ui.add_sized([ui.available_width(), 24.0], egui::Button::new(lock_lbl)).clicked() {
                self.toggle_lock_selected();
            }
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Käännöt & Asemointi
        ui.horizontal(|ui| {
            let third_w = (ui.available_width() - 8.0) / 3.0;
            if ui.add_enabled_ui(has_sel, |ui| {
                if ui.add_sized([third_w, 24.0], egui::Button::new("Käännä 90°")).on_hover_text("Käännä 90 astetta myötäpäivään").clicked() {
                    self.rotate_selected(90.0);
                }
            }).response.clicked() {}

            if ui.add_enabled_ui(has_sel, |ui| {
                if ui.add_sized([third_w, 24.0], egui::Button::new("Peilaa X")).on_hover_text("Peilaa vaakasuunnassa").clicked() {
                    self.flip_selected_h();
                }
            }).response.clicked() {}

            if ui.add_enabled_ui(has_sel, |ui| {
                if ui.add_sized([third_w, 24.0], egui::Button::new("Peilaa Y")).on_hover_text("Peilaa pystysuunnassa").clicked() {
                    self.flip_selected_v();
                }
            }).response.clicked() {}
        });

        ui.add_space(2.0);
        if ui.add_enabled(has_sel, egui::Button::new("Keskitä alustalle")).on_hover_text("Siirrä valitut kuviot alustan keskelle").clicked() {
            self.align_selection(AlignMode::CenterToBed);
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Kuviolista (Objektilista)..."))
            .on_hover_text("Avaa erillinen objektilistaus ja kappaleiden hallintaikkuna")
            .clicked()
        {
            self.show_objects_window = true;
            *close_requested = true;
        }
    }

    /// Category: Shapes flyout content
    fn render_shapes_flyout_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Muodot & Teksti").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        if ui.selectable_value(&mut self.active_tool, ActiveTool::DrawRect, "Suorakulmio (R)").clicked() {
            *close_requested = true;
        }
        if ui.selectable_value(&mut self.active_tool, ActiveTool::DrawCircle, "Ympyrä / Ellipsi (C)").clicked() {
            *close_requested = true;
        }
        if ui.selectable_value(&mut self.active_tool, ActiveTool::DrawLine, "Suora viiva (L)").clicked() {
            *close_requested = true;
        }
        if ui.selectable_value(&mut self.active_tool, ActiveTool::DrawText, "Tekstityökalu (T)").clicked() {
            if self.text_pos_x == 50.0 && self.text_pos_y == 50.0 {
                self.text_pos_x = self.doc.width * 0.25;
                self.text_pos_y = self.doc.height * 0.35;
            }
            self.show_text_dialog = true;
            *close_requested = true;
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Pikalisäys
        ui.label(egui::RichText::new("Lisää vakiokuvio:").small().weak());
        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 4.0) * 0.5;
            if ui.add_sized([half_w, 24.0], egui::Button::new("+ Neliö 40mm")).clicked() {
                self.push_undo();
                self.doc.add_shape(Shape::Rect {
                    layer_index: self.selected_layer,
                    x: 80.0,
                    y: 80.0,
                    width: 40.0,
                    height: 40.0,
                    corner_radius: 0.0,
                });
                self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                self.status_msg = "Lisätty neliö (40x40 mm)".to_string();
                *close_requested = true;
            }
            if ui.add_sized([half_w, 24.0], egui::Button::new("+ Ympyrä r=25")).clicked() {
                self.push_undo();
                self.doc.add_shape(Shape::Circle {
                    layer_index: self.selected_layer,
                    center: Point2D::new(120.0, 120.0),
                    radius: 25.0,
                });
                self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                self.status_msg = "Lisätty ympyrä (r=25 mm)".to_string();
                *close_requested = true;
            }
        });

        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 4.0) * 0.5;
            if ui.add_sized([half_w, 24.0], egui::Button::new("+ Hammasratas")).clicked() {
                self.load_sample_gear(180.0, 180.0, 12, 40.0, 50.0);
                self.selected_shape_indices = vec![self.doc.shapes.len() - 1];
                self.status_msg = "Lisätty hammasratas".to_string();
                *close_requested = true;
            }
            if ui.add_sized([half_w, 24.0], egui::Button::new("+ QR-koodi...")).clicked() {
                self.show_qr_code_window = true;
                *close_requested = true;
            }
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Vektorointi & Tuonti
        let is_image_sel = self.selected_shape_indices.iter().any(|&i| i < self.doc.shapes.len() && matches!(self.doc.shapes[i], Shape::Image { .. }));
        if ui.add_enabled(is_image_sel, egui::Button::new("Vektoroi kuva (Trace Image)..."))
            .on_hover_text("Vektoroi valittu bittikarttakuva leikkausvektoreiksi")
            .clicked()
        {
            self.open_trace_dialog();
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Tuo vektori tai kuva..."))
            .on_hover_text("Tuo DXF, PDF, SVG tai bittikarttakuva (PNG, JPG)")
            .clicked()
        {
            self.prompt_import_image(ctx);
            *close_requested = true;
        }
    }

    /// Category: Camera flyout content
    fn render_camera_flyout_content(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Kameran toiminnot").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        let is_capturing = self.oneshot_snapshot.is_some();
        let snap_btn = egui::Button::new(
            egui::RichText::new(if is_capturing { "Haetaan kuvaa..." } else { "Ota kuva alustasta (Snapshot)" })
                .strong()
                .color(egui::Color32::WHITE),
        ).fill(if is_capturing { egui::Color32::from_rgb(180, 100, 20) } else { egui::Color32::from_rgb(0, 130, 200) });

        if ui.add_sized([ui.available_width(), 28.0], snap_btn)
            .on_hover_text("Ottaa uuden kuvan alustasta ja päivittää kankaan taustakuvan")
            .clicked()
        {
            self.trigger_update_overlay();
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Kameran esikatselu & Live-syöte..."))
            .on_hover_text("Avaa live-kameran ikkunan ja säätimet")
            .clicked()
        {
            self.show_camera_window = true;
            *close_requested = true;
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        if ui.checkbox(&mut self.camera_calibration.show_on_canvas, "Näytä taustakuva kankaalla").changed() {
            let _ = self.camera_calibration.save();
        }

        if ui.checkbox(&mut self.camera_calibration.show_guide_grid, "Kohdistusruudukko").changed() {
            let _ = self.camera_calibration.save();
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        let has_sel = !self.selected_shape_indices.is_empty();
        if ui.add_enabled(has_sel, egui::Button::new("Sovita & Skaalaa laatalle (2mm)"))
            .on_hover_text("Sovita valittu kuvio 100x100mm laatalle 2mm reunamarginaalilla")
            .clicked()
        {
            self.auto_fit_to_camera_tile(true, 2.0);
            *close_requested = true;
        }

        if ui.add_enabled(has_sel, egui::Button::new("Keskitä laatalle (ilman skaalausta)"))
            .on_hover_text("Siirrä valitut kuviot laatan keskelle muuttamatta kokoa")
            .clicked()
        {
            self.auto_fit_to_camera_tile(false, 2.0);
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("4-Pisteen kalibrointivelho..."))
            .on_hover_text("Aseta alustan nurkkapisteet kameran kohdistusta varten")
            .clicked()
        {
            self.camera_window_tab = 1;
            self.show_camera_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Vektoroi alustan kappaleet..."))
            .on_hover_text("Tunnista fyysiset kappaleet kamerakuvasta automaattisesti")
            .clicked()
        {
            self.show_trace_window = true;
            *close_requested = true;
        }
    }

    /// Category: CAM & Operations flyout content
    fn render_cam_flyout_content(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Laser CAM & Toiminnot").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        let has_sel = !self.selected_shape_indices.is_empty();
        let multi_sel = self.selected_shape_indices.len() >= 2;

        // SDF Offset
        if ui.add_enabled(has_sel, egui::Button::new("Ääriviivan offset (SDF)..."))
            .on_hover_text("Luo sisä- tai ulkopuolinen ääriviiva valituille kuvioille")
            .clicked()
        {
            self.show_offset_window = true;
            *close_requested = true;
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Booleans
        ui.label(egui::RichText::new("Boolean-yhdistelyt (2+ kuviota):").small().weak());
        ui.horizontal(|ui| {
            if ui.add_enabled(has_sel, egui::Button::new("Yhdistä (Weld / Union)"))
                .on_hover_text("Yhdistä päällekkäiset kuviot yhdeksi ulkokehäksi")
                .clicked()
            {
                self.boolean_selected(BooleanOp::Union);
            }
            if ui.add_enabled(multi_sel, egui::Button::new("Vähennä (Subtract)"))
                .on_hover_text("Vähennä toinen kuvio ensimmäisestä (A - B)")
                .clicked()
            {
                self.boolean_selected(BooleanOp::Subtract);
            }
            if ui.add_enabled(multi_sel, egui::Button::new("Leikkaus (Intersect)"))
                .on_hover_text("Jätä jäljelle vain kuvioiden yhteinen leikkausalue")
                .clicked()
            {
                self.boolean_selected(BooleanOp::Intersect);
            }
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Matriisit & Pesitys
        if ui.add_enabled(has_sel, egui::Button::new("Suorakulmainen ruudukko (Grid Array)..."))
            .on_hover_text("Monista kuvio riveihin ja sarakkeisiin")
            .clicked()
        {
            self.show_grid_array_window = true;
            *close_requested = true;
        }

        if ui.add_enabled(has_sel, egui::Button::new("Ympyrämatriisi (Radial Array)..."))
            .on_hover_text("Monista kuvio kehälle halutulla kappalemäärällä")
            .clicked()
        {
            self.show_radial_array_window = true;
            *close_requested = true;
        }

        if ui.add_enabled(!self.doc.shapes.is_empty(), egui::Button::new("Automaattinen pesitys (Nesting)..."))
            .on_hover_text("Pakkaa kappaleet levylle materiaalinsäästön maksimoimiseksi")
            .clicked()
        {
            self.show_nesting_window = true;
            *close_requested = true;
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Generaattorit & Työkalut
        ui.label(egui::RichText::new("Generaattorit:").small().weak());
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Sormiliitos-laatikot (Box Maker)..."))
            .on_hover_text("Generoi vaneri- ja akryylilaatikot sormiliitoksilla")
            .clicked()
        {
            self.show_box_maker_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Living Hinge (Joustosarana)..."))
            .on_hover_text("Luo taipuisa viivoituskuvio vanerin taivuttamiseen")
            .clicked()
        {
            self.show_living_hinge_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Sarjanumerot & Nimikyltit (Batch)..."))
            .on_hover_text("Massatuota sarjanumeroituja tuotelaattoja tai nimilistoja")
            .clicked()
        {
            self.show_variable_text_window = true;
            *close_requested = true;
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Materiaalit
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Materiaalitestigeneraattori..."))
            .on_hover_text("Luo nopeus × teho -testiruudukko materiaalille")
            .clicked()
        {
            self.show_material_test_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Materiaalikirjasto (Presets)..."))
            .on_hover_text("Avaa leikkaus- ja kaiverrusarvot eri materiaaleille")
            .clicked()
        {
            self.show_material_library = true;
            *close_requested = true;
        }
    }

    /// Category: Machine & Jog flyout content
    fn render_machine_flyout_content(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Laserin ohjaus & Työn ajo").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 1. Työn käynnistys / hallinta (Job Execution)
        if !self.device.is_job_running {
            let start_btn = egui::Button::new(
                egui::RichText::new("ALOITA LEIKKAUS (START)").strong().color(egui::Color32::WHITE),
            ).fill(egui::Color32::from_rgb(34, 139, 34));
            if ui.add_sized([ui.available_width(), 32.0], start_btn)
                .on_hover_text("Optimoi työstöradat ja lähetä G-koodi laserille")
                .clicked()
            {
                self.start_job();
                *close_requested = true;
            }
        } else {
            ui.label(
                egui::RichText::new(format!(
                    "Leikataan: {:.1}% ({}/{})",
                    self.device.job_progress * 100.0,
                    self.device.job_idx,
                    self.device.job_total_lines
                )).strong().color(egui::Color32::from_rgb(255, 140, 0))
            );
            ui.add_space(2.0);

            ui.horizontal(|ui| {
                let half_w = (ui.available_width() - 4.0) * 0.5;
                if self.device.is_job_paused {
                    let resume_btn = egui::Button::new(
                        egui::RichText::new("JATKA").strong().color(egui::Color32::WHITE),
                    ).fill(egui::Color32::from_rgb(0, 140, 255));
                    if ui.add_sized([half_w, 28.0], resume_btn).clicked() {
                        self.device.resume_job();
                    }
                } else {
                    let pause_btn = egui::Button::new(
                        egui::RichText::new("KESKEYTÄ").strong().color(egui::Color32::BLACK),
                    ).fill(egui::Color32::from_rgb(255, 193, 7));
                    if ui.add_sized([half_w, 28.0], pause_btn).clicked() {
                        self.device.pause_job();
                    }
                }

                let stop_btn = egui::Button::new(
                    egui::RichText::new("HÄTÄSEIS").strong().color(egui::Color32::WHITE),
                ).fill(egui::Color32::RED);
                if ui.add_sized([half_w, 28.0], stop_btn).clicked() {
                    self.device.stop_job();
                }
            });
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 2. Kohdistusvalo & Kehystys
        let is_on = self.device.is_weak_light_on;
        let beam_btn = egui::Button::new(
            egui::RichText::new(if is_on { "Kohdistusvalo: PÄÄLLÄ" } else { "Kohdistusvalo: POIS" })
                .strong()
                .color(if is_on { egui::Color32::WHITE } else { egui::Color32::from_rgb(200, 230, 255) })
        ).fill(if is_on { egui::Color32::from_rgb(255, 69, 0) } else { egui::Color32::from_rgb(40, 70, 110) });

        if ui.add_sized([ui.available_width(), 26.0], beam_btn)
            .on_hover_text("Kytkee pienen tehon kohdistuslaserin päälle tai pois")
            .clicked()
        {
            self.device.toggle_weak_light();
        }

        ui.add_space(2.0);

        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 4.0) * 0.5;
            if ui.add_sized([half_w, 24.0], egui::Button::new("Box Frame (Suorakulmio)"))
                .on_hover_text("Ajaa työn suorakulmaisen äärialueen läpi heikolla teholla")
                .clicked()
            {
                self.start_box_frame();
                *close_requested = true;
            }

            if ui.add_sized([half_w, 24.0], egui::Button::new("Contour Frame (Ääriviiva)"))
                .on_hover_text("Ajaa kuvion tarkan leikkausmuodon läpi heikolla teholla")
                .clicked()
            {
                self.start_contour_frame();
                *close_requested = true;
            }
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 3. Asemointi
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Aja laser kotiin ($H)"))
            .on_hover_text("Ajaa koneen rajakytkimille")
            .clicked()
        {
            self.device.home();
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Aseta nollapiste tähän (G92)"))
            .on_hover_text("Asettaa nykyisen laserpään sijainnin käyttäjän nollapisteeksi (G92 X0 Y0)")
            .clicked()
        {
            self.device.set_origin_here();
            *close_requested = true;
        }

        if self.device.settings.enable_limit_switches {
            if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Nollaa rajakytkimiin"))
                .on_hover_text("Palauttaa fyysisen kytkinnollapisteen")
                .clicked()
            {
                self.device.start_mechanical_home_reset();
                *close_requested = true;
            }
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 4. Työkalut & Ikkunat
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Esikatselu / Simulaattori (Alt+P)..."))
            .on_hover_text("Avaa visuaalisen leikkausradan simulaattorin")
            .clicked()
        {
            self.show_preview_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Avaa ohjauspaneeli (Cmd+M)..."))
            .on_hover_text("Avaa täyden liikuttelupaneelin ja D-padin")
            .clicked()
        {
            self.show_move_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("Työn ohjauspaneeli (Job Control)..."))
            .on_hover_text("Avaa leikkauksen reaaliaikainen ohjaus- ja tilastoikkuna")
            .clicked()
        {
            self.show_job_window = true;
            *close_requested = true;
        }

        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 4.0) * 0.5;
            if ui.add_sized([half_w, 24.0], egui::Button::new("Yhteys (Serial)")).clicked() {
                self.show_connection_window = true;
                *close_requested = true;
            }
            if ui.add_sized([half_w, 24.0], egui::Button::new("Sarjaliikenneloki")).clicked() {
                self.show_traffic_panel = !self.show_traffic_panel;
                *close_requested = true;
            }
        });
    }
}
