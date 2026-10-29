use eframe::egui;
use crate::generator::{
    generate_box_parts, generate_living_hinge, generate_qr_shapes, generate_variable_text_shapes,
    layout_box_shapes, living_hinge_to_shapes, BoxLidType, HingePattern, QrEcLevel, QrStyle,
    VariableTextMode,
};
use super::LaserBurnApp;
use crate::Shape;

impl LaserBurnApp {
    /// Renders all modal generator windows (Box Maker, Living Hinge, QR Code, Variable Text)
    pub fn show_generators_dialog_windows(&mut self, ctx: &egui::Context) {
        self.show_box_maker_dialog(ctx);
        self.show_living_hinge_dialog(ctx);
        self.show_qr_code_dialog(ctx);
        self.show_variable_text_dialog(ctx);
    }

    // -------------------------------------------------------------
    // 1. BOX & ENCLOSURE MAKER DIALOG
    // -------------------------------------------------------------
    fn show_box_maker_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_box_maker_window {
            return;
        }

        let mut is_open = true;
        let mut do_generate = false;

        egui::Window::new("Sormiliitos-laatikot & Kotelot (Box Designer)")
            .id(egui::Id::new("box_maker_dialog_window"))
            .open(&mut is_open)
            .default_size([460.0, 520.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("📦 Sormiliitos-laatikoiden generaattori");
                ui.label(
                    egui::RichText::new("Generoi vaneri- ja akryylilaatikot automaattisella leikkausvälyksen (kerf) korjauksella.")
                        .small()
                        .weak(),
                );
                ui.add_space(8.0);

                egui::Grid::new("box_maker_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        // Ulkomitat
                        ui.label(egui::RichText::new("Ulkomitat (W × D × H):").strong());
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.box_maker_params.width).speed(1.0).range(15.0..=2000.0).suffix(" mm"))
                                .on_hover_text("Leveys X (mm)");
                            ui.label("×");
                            ui.add(egui::DragValue::new(&mut self.box_maker_params.depth).speed(1.0).range(15.0..=2000.0).suffix(" mm"))
                                .on_hover_text("Syvyys Y (mm)");
                            ui.label("×");
                            ui.add(egui::DragValue::new(&mut self.box_maker_params.height).speed(1.0).range(10.0..=2000.0).suffix(" mm"))
                                .on_hover_text("Korkeus Z (mm)");
                        });
                        ui.end_row();

                        // Materiaalin paksuus
                        ui.label(egui::RichText::new("Materiaalin paksuus (T):").strong());
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.box_maker_params.thickness).speed(0.1).range(0.5..=30.0).suffix(" mm"));
                            for &std_t in &[3.0, 4.0, 6.0] {
                                if ui.small_button(format!("{}mm", std_t)).clicked() {
                                    self.box_maker_params.thickness = std_t;
                                }
                            }
                        });
                        ui.end_row();

                        // Sormen / tapin leveys
                        ui.label("Tapin / sormen leveys:");
                        ui.add(egui::DragValue::new(&mut self.box_maker_params.tab_size).speed(0.5).range(3.0..=50.0).suffix(" mm"))
                            .on_hover_text("Tavoiteltu sormiliitoshampaan leveys (mm)");
                        ui.end_row();

                        // Kerf-kompensaatio
                        ui.label("Leikkausvälys (Kerf):");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.box_maker_params.kerf).speed(0.01).range(-0.5..=0.5).suffix(" mm"))
                                .on_hover_text("Tappien laajennus / kolojen kavennus tiukan liitoksen saamiseksi ilman liimaa (tyypillisesti 0.06 - 0.12 mm)");
                            if ui.small_button("0.08mm").clicked() {
                                self.box_maker_params.kerf = 0.08;
                            }
                            if ui.small_button("0.12mm").clicked() {
                                self.box_maker_params.kerf = 0.12;
                            }
                        });
                        ui.end_row();

                        // Kannen tyyppi
                        ui.label("Kannen tyyppi:");
                        egui::ComboBox::from_id_salt("box_lid_type_combo")
                            .selected_text(match self.box_maker_params.lid_type {
                                BoxLidType::Open => "Avoin laatikko (ei kantta)",
                                BoxLidType::FingerJointLid => "Sormiliitoskansi (suljettu kuutio)",
                                BoxLidType::SlideInLid => "Liukukansi",
                                BoxLidType::FlatLid => "Tasainen kansi",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.box_maker_params.lid_type, BoxLidType::Open, "Avoin laatikko (ei kantta)");
                                ui.selectable_value(&mut self.box_maker_params.lid_type, BoxLidType::FingerJointLid, "Sormiliitoskansi (suljettu kuutio)");
                                ui.selectable_value(&mut self.box_maker_params.lid_type, BoxLidType::SlideInLid, "Liukukansi");
                                ui.selectable_value(&mut self.box_maker_params.lid_type, BoxLidType::FlatLid, "Tasainen kansi");
                            });
                        ui.end_row();

                        // Sisäjakajat
                        ui.label("Sisäjakajat (väliseinät):");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut self.box_maker_params.dividers_x).range(0..=10));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut self.box_maker_params.dividers_y).range(0..=10));
                        });
                        ui.end_row();

                        // Palojen välistys kankaalla
                        ui.label("Palojen asetteluväli:");
                        ui.add(egui::DragValue::new(&mut self.box_maker_params.layout_margin).speed(0.5).range(1.0..=50.0).suffix(" mm"));
                        ui.end_row();
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Yhteenveto
                let parts = generate_box_parts(&self.box_maker_params);
                ui.label(egui::RichText::new(format!("Tuotettavia osia yhteensä: {} kpl", parts.len())).strong().color(egui::Color32::from_rgb(0, 220, 130)));
                for (p_idx, p) in parts.iter().enumerate() {
                    ui.label(format!("  {}. {} ({:.0} × {:.0} mm)", p_idx + 1, p.name, p.width, p.height));
                }

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let btn = egui::Button::new(egui::RichText::new("Luo laatikko kankaalle").strong().color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(34, 139, 34));
                    if ui.add_sized([180.0, 32.0], btn).clicked() {
                        do_generate = true;
                    }

                    if ui.button("Sulje").clicked() {
                        self.show_box_maker_window = false;
                    }
                });
            });

        if do_generate {
            self.push_undo();
            let parts = generate_box_parts(&self.box_maker_params);
            let shapes = layout_box_shapes(&parts, 20.0, 20.0, self.box_maker_params.layout_margin, self.selected_layer);

            let start_idx = self.doc.shapes.len();
            let count = shapes.len();
            for s in shapes {
                self.doc.add_shape(s);
            }
            self.selected_shape_indices = (start_idx..start_idx + count).collect();
            self.status_msg = format!("Luotu sormiliitoslaatikko ({} osaa)", count);
            self.show_box_maker_window = false;
        }

        if !is_open {
            self.show_box_maker_window = false;
        }
    }

    // -------------------------------------------------------------
    // 2. LIVING HINGE (JOUSTOSARANA) DIALOG
    // -------------------------------------------------------------
    fn show_living_hinge_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_living_hinge_window {
            return;
        }

        let mut is_open = true;
        let mut do_generate = false;

        egui::Window::new("Living Hinge (Joustosarana / Taivutusleikkaus)")
            .id(egui::Id::new("living_hinge_dialog_window"))
            .open(&mut is_open)
            .default_size([440.0, 480.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("🌀 Joustosarana-leikkaus (Living Hinge)");
                ui.label(
                    egui::RichText::new("Tee vanerista ja akryylistä notkeasti taivutettavaa materiaalia laserleikattavalla viivoituksella.")
                        .small()
                        .weak(),
                );
                ui.add_space(8.0);

                // Option to adapt to selected rectangle on canvas
                let has_rect_selection = self.selected_shape_indices.iter().find_map(|&idx| {
                    if idx < self.doc.shapes.len() {
                        if let Shape::Rect { x, y, width, height, .. } = self.doc.shapes[idx] {
                            return Some((x, y, width, height));
                        }
                    }
                    None
                });

                if let Some((rx, ry, rw, rh)) = has_rect_selection {
                    if ui.button(format!("Käytä valitun suorakulmion mittoja ({:.1} × {:.1} mm)", rw, rh)).clicked() {
                        self.living_hinge_params.x = rx;
                        self.living_hinge_params.y = ry;
                        self.living_hinge_params.width = rw;
                        self.living_hinge_params.height = rh;
                    }
                    ui.add_space(4.0);
                }

                egui::Grid::new("living_hinge_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        // Sijainti & Koko
                        ui.label("Taivutusalueen mitat (W × H):");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.living_hinge_params.width).speed(1.0).range(5.0..=1000.0).suffix(" mm"))
                                .on_hover_text("Taivutusalueen leveys (taivutussuunta)");
                            ui.label("×");
                            ui.add(egui::DragValue::new(&mut self.living_hinge_params.height).speed(1.0).range(5.0..=1000.0).suffix(" mm"))
                                .on_hover_text("Saranan pituus");
                        });
                        ui.end_row();

                        // Sijainti kankaalla
                        ui.label("Sijainti kankaalla (X, Y):");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.living_hinge_params.x).speed(1.0).suffix(" mm"));
                            ui.add(egui::DragValue::new(&mut self.living_hinge_params.y).speed(1.0).suffix(" mm"));
                        });
                        ui.end_row();

                        // Saranakuvion tyyli
                        ui.label("Saranan leikkaustyyli:");
                        egui::ComboBox::from_id_salt("hinge_pattern_combo")
                            .selected_text(match self.living_hinge_params.pattern {
                                HingePattern::StraightLattice => "Suora vuorotteleva viivoitus (Lattice)",
                                HingePattern::WavyLattice => "Aaltoileva pehmeä kuvio (Wavy)",
                                HingePattern::CrossLattice => "Ristikkäinen kuvio (Cross)",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.living_hinge_params.pattern, HingePattern::StraightLattice, "Suora vuorotteleva viivoitus (Lattice)");
                                ui.selectable_value(&mut self.living_hinge_params.pattern, HingePattern::WavyLattice, "Aaltoileva pehmeä kuvio (Wavy)");
                                ui.selectable_value(&mut self.living_hinge_params.pattern, HingePattern::CrossLattice, "Ristikkäinen kuvio (Cross)");
                            });
                        ui.end_row();

                        // Viiltojen väli (pitch)
                        ui.label("Viiltojen etäisyys (Pitch):");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.living_hinge_params.pitch).speed(0.1).range(1.0..=15.0).suffix(" mm"))
                                .on_hover_text("Viiltojen vaakaväli (3mm vanerille suositus 2.5 - 3.5 mm)");
                            if ui.small_button("2.5mm").clicked() { self.living_hinge_params.pitch = 2.5; }
                            if ui.small_button("3.0mm").clicked() { self.living_hinge_params.pitch = 3.0; }
                        });
                        ui.end_row();

                        // Viillon pituus
                        ui.label("Viillon pituus:");
                        ui.add(egui::DragValue::new(&mut self.living_hinge_params.cut_length).speed(0.5).range(5.0..=100.0).suffix(" mm"))
                            .on_hover_text("Yksittäisen laserleikkauksen pituus (tyypillisesti 15 - 25 mm)");
                        ui.end_row();

                        // Kannaksen / sillan leveys
                        ui.label("Sillan / kannaksen pituus:");
                        ui.add(egui::DragValue::new(&mut self.living_hinge_params.gap).speed(0.1).range(0.5..=15.0).suffix(" mm"))
                            .on_hover_text("Leikkaamaton väli viiltojen välissä, joka pitää puun koossa (tyypillisesti 1.5 - 2.5 mm)");
                        ui.end_row();

                        // Reunuksen suojamarginaali
                        ui.label("Reunamarginaali (Border):");
                        ui.add(egui::DragValue::new(&mut self.living_hinge_params.border).speed(0.5).range(0.0..=50.0).suffix(" mm"))
                            .on_hover_text("Eheä leikkaamaton alue ylä- ja alareunassa");
                        ui.end_row();
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                let preview_lines = generate_living_hinge(&self.living_hinge_params);
                ui.label(egui::RichText::new(format!("Tuotettavia leikkausviivoja: {} kpl", preview_lines.len())).strong().color(egui::Color32::from_rgb(0, 220, 130)));

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let btn = egui::Button::new(egui::RichText::new("Luo sarana kankaalle").strong().color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(34, 139, 34));
                    if ui.add_sized([180.0, 32.0], btn).clicked() {
                        do_generate = true;
                    }

                    if ui.button("Sulje").clicked() {
                        self.show_living_hinge_window = false;
                    }
                });
            });

        if do_generate {
            self.push_undo();
            let cuts = generate_living_hinge(&self.living_hinge_params);
            let count = cuts.len();
            let shapes = living_hinge_to_shapes(cuts, self.selected_layer);

            let start_idx = self.doc.shapes.len();
            for s in shapes {
                self.doc.add_shape(s);
            }
            self.selected_shape_indices = (start_idx..start_idx + count).collect();
            self.status_msg = format!("Luotu joustosarana ({} viiltoa)", count);
            self.show_living_hinge_window = false;
        }

        if !is_open {
            self.show_living_hinge_window = false;
        }
    }

    // -------------------------------------------------------------
    // 3. VECTOR QR CODE GENERATOR DIALOG
    // -------------------------------------------------------------
    fn show_qr_code_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_qr_code_window {
            return;
        }

        let mut is_open = true;
        let mut do_generate = false;

        egui::Window::new("Vektori-QR-koodi (Vector QR Code)")
            .id(egui::Id::new("qr_code_dialog_window"))
            .open(&mut is_open)
            .default_size([420.0, 440.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("📱 Vektori-QR-koodin generaattori");
                ui.label(
                    egui::RichText::new("Generoi puhtaat laserleikattavat tai kaiverrettavat QR-koodit kankaalle.")
                        .small()
                        .weak(),
                );
                ui.add_space(8.0);

                egui::Grid::new("qr_code_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        // Sisältö
                        ui.label(egui::RichText::new("Teksti tai URL:").strong());
                        ui.add(egui::TextEdit::singleline(&mut self.qr_code_params.content).desired_width(240.0));
                        ui.end_row();

                        // Koko
                        ui.label("Koko (leveys ja korkeus):");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.qr_code_params.size_mm).speed(1.0).range(10.0..=500.0).suffix(" mm"));
                            for &std_sz in &[30.0, 40.0, 50.0, 80.0] {
                                if ui.small_button(format!("{:.0}", std_sz)).clicked() {
                                    self.qr_code_params.size_mm = std_sz;
                                }
                            }
                        });
                        ui.end_row();

                        // Sijainti
                        ui.label("Sijainti kankaalla (X, Y):");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.qr_code_params.x).speed(1.0).suffix(" mm"));
                            ui.add(egui::DragValue::new(&mut self.qr_code_params.y).speed(1.0).suffix(" mm"));
                        });
                        ui.end_row();

                        // Moduulien tyyli
                        ui.label("Kuvion tyyli:");
                        egui::ComboBox::from_id_salt("qr_style_combo")
                            .selected_text(match self.qr_code_params.style {
                                QrStyle::SharpSquares => "Terävät neliöt (Sharp, optimoitu leikkaukseen)",
                                QrStyle::RoundedSquares => "Pyöristetyt neliöt (Rounded, moderni kyltti)",
                                QrStyle::Dots => "Pyöreät pisteet (Circular Dots)",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.qr_code_params.style, QrStyle::SharpSquares, "Terävät neliöt (Sharp, optimoitu leikkaukseen)");
                                ui.selectable_value(&mut self.qr_code_params.style, QrStyle::RoundedSquares, "Pyöristetyt neliöt (Rounded, moderni kyltti)");
                                ui.selectable_value(&mut self.qr_code_params.style, QrStyle::Dots, "Pyöreät pisteet (Circular Dots)");
                            });
                        ui.end_row();

                        // Virheenkorjaustaso
                        ui.label("Virheenkorjaustaso (ECC):");
                        egui::ComboBox::from_id_salt("qr_ecc_combo")
                            .selected_text(match self.qr_code_params.ec_level {
                                QrEcLevel::Low => "Matala (~7% korjaus)",
                                QrEcLevel::Medium => "Keskitaso (~15% korjaus - suositus)",
                                QrEcLevel::Quartile => "Korkea (~25% korjaus)",
                                QrEcLevel::High => "Maksimaalinen (~30% korjaus)",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.qr_code_params.ec_level, QrEcLevel::Low, "Matala (~7% korjaus)");
                                ui.selectable_value(&mut self.qr_code_params.ec_level, QrEcLevel::Medium, "Keskitaso (~15% korjaus - suositus)");
                                ui.selectable_value(&mut self.qr_code_params.ec_level, QrEcLevel::Quartile, "Korkea (~25% korjaus)");
                                ui.selectable_value(&mut self.qr_code_params.ec_level, QrEcLevel::High, "Maksimaalinen (~30% korjaus)");
                            });
                        ui.end_row();

                        // Marginaali (Quiet zone)
                        ui.label("Hiljainen alue (marginaali):");
                        ui.add(egui::DragValue::new(&mut self.qr_code_params.quiet_zone).range(0..=8).suffix(" moduulia"));
                        ui.end_row();

                        // Käänteiset värit (Invert)
                        ui.label("Käänteinen pohja (Invert):");
                        ui.checkbox(&mut self.qr_code_params.invert, "Luo taustalaatta (negatiivikaiverrus)");
                        ui.end_row();
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    let btn = egui::Button::new(egui::RichText::new("Lisää QR-koodi kankaalle").strong().color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(34, 139, 34));
                    if ui.add_sized([180.0, 32.0], btn).clicked() {
                        do_generate = true;
                    }

                    if ui.button("Sulje").clicked() {
                        self.show_qr_code_window = false;
                    }
                });
            });

        if do_generate {
            match generate_qr_shapes(&self.qr_code_params, self.selected_layer) {
                Ok(shapes) => {
                    self.push_undo();
                    let start_idx = self.doc.shapes.len();
                    let count = shapes.len();
                    for s in shapes {
                        self.doc.add_shape(s);
                    }
                    self.selected_shape_indices = (start_idx..start_idx + count).collect();
                    self.status_msg = format!("Lisätty QR-koodi kankaalle ({:.0}x{:.0} mm)", self.qr_code_params.size_mm, self.qr_code_params.size_mm);
                    self.show_qr_code_window = false;
                }
                Err(err) => {
                    self.status_msg = format!("QR-koodivirhe: {}", err);
                }
            }
        }

        if !is_open {
            self.show_qr_code_window = false;
        }
    }

    // -------------------------------------------------------------
    // 4. VARIABLE TEXT & BATCH PLAQUES DIALOG
    // -------------------------------------------------------------
    fn show_variable_text_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_variable_text_window {
            return;
        }

        let mut is_open = true;
        let mut do_generate = false;

        egui::Window::new("Sarjanumerointi & Nimikyltit (Batch Text)")
            .id(egui::Id::new("variable_text_dialog_window"))
            .open(&mut is_open)
            .default_size([500.0, 560.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("🏷️ Sarjanumerointi & Muuttuva teksti (Batch)");
                ui.label(
                    egui::RichText::new("Tuota automaattisesti sarjanumeroituja tuotelaattoja tai nimilistoja massana laserille.")
                        .small()
                        .weak(),
                );
                ui.add_space(8.0);

                // Tilan valinta
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.variable_text_params.mode, VariableTextMode::Sequential, "1. Numeerinen sarja (SN-001..)");
                    ui.selectable_value(&mut self.variable_text_params.mode, VariableTextMode::CustomList, "2. Vapaa nimilista");
                    ui.selectable_value(&mut self.variable_text_params.mode, VariableTextMode::CsvTemplate, "3. CSV-pohja");
                });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                match self.variable_text_params.mode {
                    VariableTextMode::Sequential => {
                        egui::Grid::new("var_seq_grid")
                            .num_columns(2)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("Etuliite (Prefix):");
                                ui.text_edit_singleline(&mut self.variable_text_params.prefix);
                                ui.end_row();

                                ui.label("Numeroväli (Alku / Loppu / Askel):");
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut self.variable_text_params.start_num));
                                    ui.label("–");
                                    ui.add(egui::DragValue::new(&mut self.variable_text_params.end_num));
                                    ui.label("Askel:");
                                    ui.add(egui::DragValue::new(&mut self.variable_text_params.step).range(1..=100));
                                });
                                ui.end_row();

                                ui.label("Nollatäyttö (Padding):");
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut self.variable_text_params.padding).range(1..=8).suffix(" nroa"));
                                    ui.label("Jälkiliite (Suffix):");
                                    ui.text_edit_singleline(&mut self.variable_text_params.suffix);
                                });
                                ui.end_row();
                            });
                    }

                    VariableTextMode::CustomList => {
                        ui.label(egui::RichText::new("Syötä nimet tai tekstit (yksi per rivi):").strong());
                        ui.add(
                            egui::TextEdit::multiline(&mut self.variable_text_params.raw_data)
                                .desired_rows(6)
                                .desired_width(ui.available_width()),
                        );
                    }

                    VariableTextMode::CsvTemplate => {
                        ui.label("CSV-rivit (erottimena pilkku tai puolipiste):");
                        ui.add(
                            egui::TextEdit::multiline(&mut self.variable_text_params.raw_data)
                                .desired_rows(5)
                                .desired_width(ui.available_width()),
                        );
                        ui.horizontal(|ui| {
                            ui.label("Korvauspohja:");
                            ui.text_edit_singleline(&mut self.variable_text_params.csv_template);
                            ui.label(egui::RichText::new("(Käytä {1}, {2}...)").small().weak());
                        });
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Typografia & Asettelu
                egui::Grid::new("var_layout_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Tekstin koko:");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.variable_text_params.font_size_mm).speed(0.5).range(2.0..=50.0).suffix(" mm"));
                            ui.checkbox(&mut self.variable_text_params.use_single_line_font, "Single-line stroke font (nopea kaiverrus)");
                        });
                        ui.end_row();

                        ui.label("Asettelu (Sarakkeet / Välistys):");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.variable_text_params.columns).range(1..=20).suffix(" saraketta"));
                            ui.label("Väli:");
                            ui.add(egui::DragValue::new(&mut self.variable_text_params.spacing_x).speed(0.5).range(1.0..=50.0).suffix(" mm"));
                        });
                        ui.end_row();

                        // Nimikylttipohja (Badge plaque)
                        ui.label("Nimikylttipohja (Reunaviiva):");
                        ui.checkbox(&mut self.variable_text_params.add_badge, "Luo leikattava kylttipohja tekstin ympärille");
                        ui.end_row();

                        if self.variable_text_params.add_badge {
                            ui.label("Kyltin mitat (W × H):");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.variable_text_params.badge_width).speed(1.0).range(10.0..=300.0).suffix(" mm"));
                                ui.label("×");
                                ui.add(egui::DragValue::new(&mut self.variable_text_params.badge_height).speed(1.0).range(10.0..=300.0).suffix(" mm"));
                                ui.label("Pyöristys:");
                                ui.add(egui::DragValue::new(&mut self.variable_text_params.badge_corner_radius).speed(0.5).range(0.0..=20.0).suffix(" mm"));
                            });
                            ui.end_row();

                            ui.label("Kiinnitysreikä:");
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.variable_text_params.add_mounting_hole, "Lisää ripustusreikä vasemmalle");
                                if self.variable_text_params.add_mounting_hole {
                                    ui.add(egui::DragValue::new(&mut self.variable_text_params.hole_diameter).speed(0.1).range(1.0..=15.0).suffix(" mm"));
                                }
                            });
                            ui.end_row();
                        }
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                let preview_strings = crate::generator::resolve_variable_strings(&self.variable_text_params);
                ui.label(egui::RichText::new(format!("Tuotettavia nimikylttejä yhteensä: {} kpl", preview_strings.len())).strong().color(egui::Color32::from_rgb(0, 220, 130)));

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let btn = egui::Button::new(egui::RichText::new("Generoi sarja kankaalle").strong().color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(34, 139, 34));
                    if ui.add_sized([200.0, 32.0], btn).clicked() {
                        do_generate = true;
                    }

                    if ui.button("Sulje").clicked() {
                        self.show_variable_text_window = false;
                    }
                });
            });

        if do_generate {
            self.push_undo();
            let cut_layer = if self.doc.layers.len() > 1 { 1 } else { 0 };
            let shapes = generate_variable_text_shapes(&self.variable_text_params, self.selected_layer, cut_layer);
            let count = shapes.len();

            let start_idx = self.doc.shapes.len();
            for s in shapes {
                self.doc.add_shape(s);
            }
            self.selected_shape_indices = (start_idx..start_idx + count).collect();
            self.status_msg = format!("Generoitu {} kpl nimikylttejä kankaalle", count);
            self.show_variable_text_window = false;
        }

        if !is_open {
            self.show_variable_text_window = false;
        }
    }
}
