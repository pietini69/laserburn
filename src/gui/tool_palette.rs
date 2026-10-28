use eframe::egui;
use super::{ActiveTool, LaserBurnApp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    Select,  // ↖️ Valinta, Solmueditori, Mittaus
    Shapes,  // ✏️ Muodot (Neliö, Ympyrä, Viiva, Teksti)
    Camera,  // 📷 Kamera (Alustakuva, Tausta ON/OFF, Kalibrointi, Läpinäkyvyys, Vektorointi)
    Cam,     // ▦ CAM & Täyttö (Hatch, Offset, Matriisit, Materiaalitesti)
    Machine, // 🎯 Laserohjaus (Kehystys, Kohdistusvalo, Koti, Nollaa, Ohjauspaneeli)
}

impl LaserBurnApp {
    /// Renders the V2 Photoshop-style floating/detachable tool palette and flyouts
    pub fn render_tool_palette_v2(&mut self, ctx: &egui::Context) {
        if !self.show_tools_palette {
            return;
        }

        let is_dark = self.device.settings.dark_mode;

        // Render in detached native OS window if requested, else floating inside main window
        if self.detach_tools_window {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("photoncam_v2_tools_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("PhotonCAM - Työkalut")
                    .with_inner_size([68.0, 340.0])
                    .with_min_inner_size([58.0, 300.0])
                    .with_resizable(true)
                    .with_always_on_top(),
                |ctx, _class| {
                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::default()
                                .fill(if is_dark { egui::Color32::from_rgb(24, 25, 30) } else { egui::Color32::from_rgb(240, 242, 246) })
                                .inner_margin(egui::Margin::symmetric(6, 8))
                        )
                        .show(ctx, |ui| {
                            self.render_palette_vertical_buttons(ui, ctx, true);
                        });
                },
            );
        } else {
            let mut is_open = self.show_tools_palette;
            let window = egui::Window::new("🛠 Työkalut")
                .id(egui::Id::new("floating_v2_tools_window"))
                .default_pos([14.0, 75.0])
                .resizable(false)
                .collapsible(true)
                .title_bar(true)
                .open(&mut is_open);

            window.show(ctx, |ui| {
                self.render_palette_vertical_buttons(ui, ctx, false);
            });

            self.show_tools_palette = is_open;
        }

        // Render flyout submenu to the side of the active category button
        self.render_palette_flyout_card(ctx);
    }

    /// Renders the vertical button strip
    fn render_palette_vertical_buttons(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, is_detached: bool) {
        // Header controls (Detach / Dock toggle)
        ui.horizontal(|ui| {
            let detach_icon = if is_detached { "⤡" } else { "⤢" };
            let detach_tip = if is_detached {
                "Telakoi takaisin pääikkunan sisään"
            } else {
                "Irrota omaksi työpöytäikkunaksi pääikkunan ulkopuolelle (Multi-viewport)"
            };

            if ui.button(egui::RichText::new(detach_icon).strong().size(13.0))
                .on_hover_text(detach_tip)
                .clicked()
            {
                self.detach_tools_window = !self.detach_tools_window;
            }

            ui.label(egui::RichText::new("V2").size(10.0).color(egui::Color32::GRAY));
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Category 1: Valinta & Solmut (Select / NodeEdit)
        let is_select_active = self.active_tool == ActiveTool::Select || self.active_tool == ActiveTool::NodeEdit;
        let is_select_open = self.open_flyout == Some(ToolCategory::Select);
        let select_icon = if self.active_tool == ActiveTool::NodeEdit { "🔲" } else { "↖" };
        if self.draw_palette_tool_btn(ui, select_icon, is_select_active, is_select_open, "Valintatyökalut & Solmumuokkaus (Klikkaa avataksesi alivalikon)") {
            self.toggle_flyout(ToolCategory::Select);
        }

        ui.add_space(4.0);

        // Category 2: Piirtotyökalut (Shapes: Rect, Circle, Line, Text)
        let is_shape_active = matches!(self.active_tool, ActiveTool::DrawRect | ActiveTool::DrawCircle | ActiveTool::DrawLine | ActiveTool::DrawText);
        let is_shape_open = self.open_flyout == Some(ToolCategory::Shapes);
        let shape_icon = match self.active_tool {
            ActiveTool::DrawRect => "⬜",
            ActiveTool::DrawCircle => "⭕",
            ActiveTool::DrawLine => "📏",
            ActiveTool::DrawText => "🔤",
            _ => "✏",
        };
        if self.draw_palette_tool_btn(ui, shape_icon, is_shape_active, is_shape_open, "Muodot & Piirtotyökalut (Neliö, Ympyrä, Viiva, Teksti)") {
            self.toggle_flyout(ToolCategory::Shapes);
        }

        ui.add_space(4.0);

        // Category 3: Kamera & Kohdistus (Camera & Vision)
        let is_cam_capturing = self.oneshot_snapshot.is_some();
        let is_cam_showing = self.camera_calibration.show_on_canvas;
        let is_cam_open = self.open_flyout == Some(ToolCategory::Camera);
        let cam_icon = if is_cam_capturing { "⏳" } else { "📷" };
        let cam_btn_tip = if is_cam_capturing {
            "Otetaan kuvaa alustasta... (Klikkaa avataksesi kameran toiminnot)"
        } else if is_cam_showing {
            "Kamera: Taustakuva aktiivinen (Klikkaa avataksesi toiminnot)"
        } else {
            "Kameran toiminnot & Alustan kohdistus (Klikkaa avataksesi toiminnot)"
        };
        if self.draw_palette_tool_btn(ui, cam_icon, is_cam_showing || is_cam_capturing, is_cam_open, cam_btn_tip) {
            self.toggle_flyout(ToolCategory::Camera);
        }

        ui.add_space(4.0);

        // Category 4: Laser CAM & Täyttö (CAM: Hatch, Offset, Array, Materiaali)
        let is_cam_tool_open = self.open_flyout == Some(ToolCategory::Cam);
        if self.draw_palette_tool_btn(ui, "▦", false, is_cam_tool_open, "Laser CAM & Täyttöoperaatiot (Hatch, Offset, Matriisit)") {
            self.toggle_flyout(ToolCategory::Cam);
        }

        ui.add_space(4.0);

        // Category 5: Laserohjaus & Kohdistus (Machine & Jog)
        let is_machine_open = self.open_flyout == Some(ToolCategory::Machine);
        let is_beam_on = self.device.is_weak_light_on;
        let machine_icon = if is_beam_on { "💡" } else { "🎯" };
        if self.draw_palette_tool_btn(ui, machine_icon, is_beam_on, is_machine_open, "Laserin ohjaus & Kohdistus (Kehystys, Kohdistusvalo, Koti)") {
            self.toggle_flyout(ToolCategory::Machine);
        }

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // File Import Shortcut Button
        let import_btn = egui::Button::new(egui::RichText::new("📂").size(16.0))
            .min_size(egui::vec2(38.0, 32.0));
        if ui.add(import_btn).on_hover_text("Tuo tiedosto (DXF, PDF, SVG, Kuvat)").clicked() {
            self.prompt_import_image(ctx);
        }
    }

    /// Draws a square Photoshop-style toolbar button with a tiny flyout triangle mark in the corner
    fn draw_palette_tool_btn(
        &mut self,
        ui: &mut egui::Ui,
        icon: &str,
        is_active: bool,
        is_open: bool,
        tooltip: &str,
    ) -> bool {
        let btn_size = egui::vec2(40.0, 40.0);
        let is_dark = self.device.settings.dark_mode;

        let fill_color = if is_active || is_open {
            if is_dark {
                egui::Color32::from_rgb(33, 115, 185)
            } else {
                egui::Color32::from_rgb(50, 130, 205)
            }
        } else if is_dark {
            egui::Color32::from_rgb(38, 42, 50)
        } else {
            egui::Color32::from_rgb(228, 232, 240)
        };

        let stroke = if is_active || is_open {
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(100, 180, 255))
        } else {
            egui::Stroke::new(1.0_f32, if is_dark { egui::Color32::from_rgb(55, 60, 72) } else { egui::Color32::from_rgb(205, 210, 220) })
        };

        let button = egui::Button::new(
            egui::RichText::new(icon)
                .size(17.0)
                .color(if is_active || is_open { egui::Color32::WHITE } else if is_dark { egui::Color32::from_rgb(220, 225, 235) } else { egui::Color32::from_rgb(40, 45, 55) })
        )
        .min_size(btn_size)
        .corner_radius(6.0)
        .fill(fill_color)
        .stroke(stroke);

        let response = ui.add(button).on_hover_text(tooltip);

        // Draw Photoshop-style tiny corner triangle in bottom-right corner
        let r = response.rect;
        let painter = ui.painter();
        let triangle_color = if is_active || is_open {
            egui::Color32::WHITE
        } else if is_dark {
            egui::Color32::from_gray(140)
        } else {
            egui::Color32::from_gray(110)
        };

        let p1 = egui::pos2(r.max.x - 3.0, r.max.y - 3.0);
        let p2 = egui::pos2(r.max.x - 7.5, r.max.y - 3.0);
        let p3 = egui::pos2(r.max.x - 3.0, r.max.y - 7.5);
        painter.add(egui::Shape::convex_polygon(
            vec![p1, p2, p3],
            triangle_color,
            egui::Stroke::NONE,
        ));

        if response.clicked() {
            // Save anchor screen position for the flyout card
            self.last_flyout_anchor = Some(r);
            true
        } else {
            false
        }
    }

    /// Toggles or sets the active flyout category
    fn toggle_flyout(&mut self, cat: ToolCategory) {
        if self.open_flyout == Some(cat) {
            self.open_flyout = None;
        } else {
            self.open_flyout = Some(cat);
        }
    }

    /// Renders the flyout popup card to the side of the toolbar
    fn render_palette_flyout_card(&mut self, ctx: &egui::Context) {
        let cat = match self.open_flyout {
            Some(c) => c,
            None => return,
        };

        let is_dark = self.device.settings.dark_mode;
        let anchor = self.last_flyout_anchor.unwrap_or(egui::Rect::from_min_size(
            egui::pos2(70.0, 100.0),
            egui::vec2(40.0, 40.0),
        ));

        // Position flyout immediately to the right of the button
        let flyout_pos = egui::pos2(anchor.max.x + 8.0, (anchor.min.y - 4.0).max(60.0));

        let bg_color = if is_dark {
            egui::Color32::from_rgb(26, 28, 35)
        } else {
            egui::Color32::from_rgb(248, 249, 252)
        };
        let stroke_color = if is_dark {
            egui::Color32::from_rgb(60, 65, 80)
        } else {
            egui::Color32::from_rgb(195, 200, 215)
        };

        let mut close_requested = false;

        egui::Area::new(egui::Id::new("v2_palette_flyout_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(flyout_pos)
            .show(ctx, |ui| {
                egui::Frame::window(ui.style())
                    .corner_radius(8.0)
                    .fill(bg_color)
                    .stroke(egui::Stroke::new(1.0_f32, stroke_color))
                    .shadow(egui::Shadow {
                        offset: [0, 4],
                        blur: 12,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(if is_dark { 160 } else { 70 }),
                    })
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_min_width(230.0);
                        ui.set_max_width(280.0);

                        match cat {
                            ToolCategory::Camera => {
                                self.render_camera_flyout_content(ui, ctx, &mut close_requested);
                            }
                            ToolCategory::Select => {
                                self.render_select_flyout_content(ui, ctx, &mut close_requested);
                            }
                            ToolCategory::Shapes => {
                                self.render_shapes_flyout_content(ui, ctx, &mut close_requested);
                            }
                            ToolCategory::Cam => {
                                self.render_cam_flyout_content(ui, ctx, &mut close_requested);
                            }
                            ToolCategory::Machine => {
                                self.render_machine_flyout_content(ui, ctx, &mut close_requested);
                            }
                        }
                    });
            });

        if close_requested {
            self.open_flyout = None;
        }
    }

    /// Category: Camera flyout content
    fn render_camera_flyout_content(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📷 Kamera & Kohdistus").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 1. Ota uusi taustakuva (1-Click snapshot)
        let is_capturing = self.oneshot_snapshot.is_some();
        let snap_text = if is_capturing {
            "⏳ Otetaan kuvaa alustasta..."
        } else {
            "📸 Ota uusi taustakuva"
        };
        let snap_btn = egui::Button::new(
            egui::RichText::new(snap_text)
                .strong()
                .color(egui::Color32::WHITE)
        ).fill(if is_capturing {
            egui::Color32::from_rgb(180, 100, 20)
        } else {
            egui::Color32::from_rgb(33, 115, 185)
        });

        if ui.add_sized([ui.available_width(), 28.0], snap_btn)
            .on_hover_text("Käynnistää kameran, ottaa tuoreen kuvan työstöalustasta ja asettaa sen canvasin taustalle")
            .clicked()
        {
            self.trigger_update_overlay();
        }

        ui.add_space(4.0);

        // 2. Näytä taustakuva kankaalla ON/OFF
        let show_bg = self.camera_calibration.show_on_canvas;
        let bg_label = if show_bg {
            "👁️ Taustakuva: NÄKYVISSÄ"
        } else {
            "👁️‍🗨️ Taustakuva: PIILOTETTU"
        };
        let bg_btn = egui::Button::new(
            egui::RichText::new(bg_label)
                .strong()
                .color(if show_bg { egui::Color32::from_rgb(0, 230, 118) } else { egui::Color32::GRAY })
        );
        if ui.add_sized([ui.available_width(), 24.0], bg_btn)
            .on_hover_text("Näytä tai piilota kameran taustakuva leikkausalustan alla")
            .clicked()
        {
            self.camera_calibration.show_on_canvas = !show_bg;
            let _ = self.camera_calibration.save();
        }

        ui.add_space(4.0);

        // 3. Taustakuvan läpinäkyvyys
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Läpinäkyvyys:").small());
            let mut opacity_pct = (self.camera_calibration.opacity * 100.0) as i32;
            if ui.add(egui::Slider::new(&mut opacity_pct, 5..=100).suffix("%")).changed() {
                self.camera_calibration.opacity = (opacity_pct as f32 / 100.0).clamp(0.05, 1.0);
                let _ = self.camera_calibration.save();
            }
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 4. Kalibrointivelho (4 pistettä)
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🎯 4-Pisteen kalibrointivelho..."))
            .on_hover_text("Kalibroi kameran linssi ja oikaise perspektiivi alustan kulmien avulla")
            .clicked()
        {
            self.camera_window_tab = 1;
            self.show_camera_window = true;
            *close_requested = true;
        }

        // 5. Kameran hallinta & Live-striimi
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("⚙️ Kameran valinta & Live-kuva..."))
            .on_hover_text("Valitse USB-kamera, resoluutio ja tarkastele reaaliaikaista videokuvaa")
            .clicked()
        {
            self.camera_window_tab = 0;
            self.show_camera_window = true;
            *close_requested = true;
        }

        // 6. Vektoroi kappale alustalta
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🪄 Vektoroi alustan kappaleet..."))
            .on_hover_text("Tunnista alustalla olevan fyysisen kappaleen ääriviivat suoraan leikkausvektoreiksi")
            .clicked()
        {
            self.show_trace_window = true;
            *close_requested = true;
        }
    }

    /// Category: Select flyout content
    fn render_select_flyout_content(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("↖ Valinta & Muokkaus").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        if ui.selectable_value(&mut self.active_tool, ActiveTool::Select, "↖ Vapaa valinta / Siirto (V)").clicked() {
            *close_requested = true;
        }

        if ui.selectable_value(&mut self.active_tool, ActiveTool::NodeEdit, "🔲 Solmumuokkaus (N)").clicked() {
            self.convert_selected_to_path();
            *close_requested = true;
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        let has_sel = !self.selected_shape_indices.is_empty();
        if ui.add_enabled(has_sel, egui::Button::new("🔄 Käännä 90°")).clicked() {
            self.rotate_selected(90.0);
        }
        if ui.add_enabled(has_sel, egui::Button::new("↔ Peilaa vaakasuunnassa (X)")).clicked() {
            self.flip_selected_h();
        }
        if ui.add_enabled(has_sel, egui::Button::new("↕ Peilaa pystysuunnassa (Y)")).clicked() {
            self.flip_selected_v();
        }
    }

    /// Category: Shapes flyout content
    fn render_shapes_flyout_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("✏ Muodot & Teksti").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        if ui.selectable_value(&mut self.active_tool, ActiveTool::DrawRect, "⬜ Suorakulmio (R)").clicked() {
            *close_requested = true;
        }
        if ui.selectable_value(&mut self.active_tool, ActiveTool::DrawCircle, "⭕ Ympyrä / Ellipsi (C)").clicked() {
            *close_requested = true;
        }
        if ui.selectable_value(&mut self.active_tool, ActiveTool::DrawLine, "📏 Suora viiva (L)").clicked() {
            *close_requested = true;
        }
        if ui.selectable_value(&mut self.active_tool, ActiveTool::DrawText, "🔤 Tekstityökalu (T)").clicked() {
            self.show_text_dialog = true;
            *close_requested = true;
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("📂 Tuo vektori tai kuva..."))
            .on_hover_text("Tuo DXF, PDF, SVG tai bittikarttakuva (PNG, JPG)")
            .clicked()
        {
            self.prompt_import_image(ctx);
            *close_requested = true;
        }
    }

    /// Category: CAM & Operations flyout content
    fn render_cam_flyout_content(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("▦ Laser CAM & Täyttö").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🔲 Ääriviivan offset (SDF)..."))
            .on_hover_text("Luo sisä- tai ulkopuolinen ääriviiva valituille kuvioille")
            .clicked()
        {
            self.show_offset_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🔀 Suorakulmainen ruudukko..."))
            .on_hover_text("Monista kuvio riveihin ja sarakkeisiin")
            .clicked()
        {
            self.show_grid_array_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🔄 Ympyrämatriisi (Radial)..."))
            .on_hover_text("Monista kuvio kehälle halutulla kappalemäärällä")
            .clicked()
        {
            self.show_radial_array_window = true;
            *close_requested = true;
        }

        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🧪 Materiaalitestigeneraattori..."))
            .on_hover_text("Luo nopeus × teho -testiruudukko materiaalille")
            .clicked()
        {
            self.show_material_test_window = true;
            *close_requested = true;
        }
    }

    /// Category: Machine & Jog flyout content
    fn render_machine_flyout_content(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, close_requested: &mut bool) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🎯 Laserin ohjaus & Kohdistus").strong().size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Weak Beam Toggle
        let is_on = self.device.is_weak_light_on;
        let beam_btn = egui::Button::new(
            egui::RichText::new(if is_on { "💡 Kohdistusvalo: PÄÄLLÄ" } else { "💡 Kohdistusvalo: POIS" })
                .strong()
                .color(if is_on { egui::Color32::WHITE } else { egui::Color32::from_rgb(200, 230, 255) })
        ).fill(if is_on { egui::Color32::from_rgb(255, 69, 0) } else { egui::Color32::from_rgb(40, 70, 110) });

        if ui.add_sized([ui.available_width(), 26.0], beam_btn)
            .on_hover_text("Kytkee pienen tehon kohdistuslaserin päälle tai pois")
            .clicked()
        {
            self.device.toggle_weak_light();
        }

        ui.add_space(3.0);

        // Frame Test
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🔲 Aja kehystys (Box Frame)"))
            .on_hover_text("Ajaa työn suorakulmaisen äärialueen läpi heikolla teholla")
            .clicked()
        {
            self.start_box_frame();
            *close_requested = true;
        }

        ui.add_space(3.0);

        // Home
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🏠 Aja laser kotiin ($H)"))
            .on_hover_text("Ajaa koneen rajakytkimille")
            .clicked()
        {
            self.device.home();
            *close_requested = true;
        }

        // Set Origin
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🎯 Aseta nollapiste tähän (Zero)"))
            .on_hover_text("Asettaa nykyisen laserpään sijainnin käyttäjän nollapisteeksi (G92 X0 Y0)")
            .clicked()
        {
            self.device.set_origin_here();
            *close_requested = true;
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Full Machine Control Window
        if ui.add_sized([ui.available_width(), 24.0], egui::Button::new("🎛️ Avaa ohjauspaneeli..."))
            .on_hover_text("Avaa täyden liikuttelupaneelin ja D-padin")
            .clicked()
        {
            self.show_move_window = true;
            *close_requested = true;
        }
    }
}
