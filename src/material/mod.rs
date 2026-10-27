//! Material Library and preset management for LaserBurn.
//!
//! Organizes laser speed, power, pass count, and dither settings in a clean
//! 3-tier hierarchy: Material -> Thickness/Variant -> Operation (Cut, Score, Engrave, Photo).
//! Supports loading/saving from `~/Documents/LaserBurn_materials.json`.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::document::{CutMode, CutSetting, DitherMode, Layer};

/// A single cut/engrave operation preset for a specific material and thickness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialOperation {
    pub name: String,                  // e.g. "Cut", "Vector Score", "Dark Engrave", "Ceramic Paint Stack"
    pub cut_setting: CutSetting,       // Complete LaserBurn cut parameters
    #[serde(default)]
    pub notes: String,                 // Optional tips (e.g. "Requires honeycomb and air assist")
}

impl MaterialOperation {
    pub fn new(name: &str, cut_setting: CutSetting, notes: &str) -> Self {
        Self {
            name: name.to_string(),
            cut_setting,
            notes: notes.to_string(),
        }
    }

    /// Apply this operation's cut settings onto an existing document Layer.
    pub fn apply_to_layer(&self, layer: &mut Layer) {
        let mut setting = self.cut_setting.clone();
        // Keep the original layer name if it already has a custom name, or suggest operation
        if setting.name.is_empty() || setting.name == "Default Cut" {
            setting.name = self.name.clone();
        }
        layer.cut_setting = setting;
    }

    /// Create an operation from an active Layer.
    pub fn from_layer(name: &str, layer: &Layer, notes: &str) -> Self {
        Self {
            name: name.to_string(),
            cut_setting: layer.cut_setting.clone(),
            notes: notes.to_string(),
        }
    }
}

/// A specific thickness or variant (e.g. 3.0 mm, 4.0 mm, 100x100mm Tile).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThicknessEntry {
    pub thickness_mm: f64,             // 0.0 if not applicable (e.g. ceramic tile)
    pub label: String,                 // e.g. "3.0 mm", "4.0 mm", "100x100mm Tile"
    pub operations: Vec<MaterialOperation>,
}

impl ThicknessEntry {
    pub fn new(thickness_mm: f64, label: &str, operations: Vec<MaterialOperation>) -> Self {
        Self {
            thickness_mm,
            label: label.to_string(),
            operations,
        }
    }

    pub fn find_operation(&self, op_name: &str) -> Option<&MaterialOperation> {
        self.operations.iter().find(|o| o.name.eq_ignore_ascii_case(op_name))
    }

    pub fn find_operation_mut(&mut self, op_name: &str) -> Option<&mut MaterialOperation> {
        self.operations.iter_mut().find(|o| o.name.eq_ignore_ascii_case(op_name))
    }
}

/// A top-level material type (e.g. "Birch Plywood", "Ceramic Tile", "Cast Acrylic").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialEntry {
    pub name: String,                  // e.g. "Birch Plywood"
    pub category: String,              // e.g. "Wood & Plywood", "Ceramics", "Plastics"
    #[serde(default)]
    pub description: String,           // Optional description
    pub thicknesses: Vec<ThicknessEntry>,
}

impl MaterialEntry {
    pub fn new(name: &str, category: &str, description: &str, thicknesses: Vec<ThicknessEntry>) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            description: description.to_string(),
            thicknesses,
        }
    }

    pub fn find_thickness(&self, thickness_mm: f64) -> Option<&ThicknessEntry> {
        self.thicknesses.iter().find(|t| (t.thickness_mm - thickness_mm).abs() < 0.05)
    }

    pub fn find_thickness_by_label(&self, label: &str) -> Option<&ThicknessEntry> {
        self.thicknesses.iter().find(|t| t.label.eq_ignore_ascii_case(label))
    }

    pub fn find_thickness_mut(&mut self, thickness_mm: f64) -> Option<&mut ThicknessEntry> {
        self.thicknesses.iter_mut().find(|t| (t.thickness_mm - thickness_mm).abs() < 0.05)
    }

    pub fn find_matching_thickness_mut(&mut self, thickness_mm: f64, label: &str) -> Option<&mut ThicknessEntry> {
        if thickness_mm > 0.05 {
            self.thicknesses.iter_mut().find(|t| (t.thickness_mm - thickness_mm).abs() < 0.05)
        } else {
            self.thicknesses.iter_mut().find(|t| t.label.eq_ignore_ascii_case(label) || (t.thickness_mm <= 0.05 && (t.label == "Single / Surface" || label == "Single / Surface")))
        }
    }
}

/// Full Material Library containing curated materials and user presets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialLibrary {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_profile")]
    pub laser_profile: String,         // e.g. "10W Optical Diode", "20W Diode", "50W CO2"
    pub materials: Vec<MaterialEntry>,
}

fn default_version() -> u32 {
    1
}

fn default_profile() -> String {
    "PhotonCAM Oletus (10W-20W)".to_string()
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        Self::default_library()
    }
}

impl MaterialLibrary {
    /// Returns default storage path (`~/Documents/PhotonCAM_materials.json` or fallback to home).
    /// If an old `LaserBurn_materials.json` exists and `PhotonCAM_materials.json` does not, it migrates it.
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let docs = PathBuf::from(&home).join("Documents");
        let dir = if docs.exists() && docs.is_dir() { docs } else { PathBuf::from(home) };
        let new_path = dir.join("PhotonCAM_materials.json");
        let old_path = dir.join("LaserBurn_materials.json");
        if !new_path.exists() && old_path.exists() {
            let _ = fs::copy(&old_path, &new_path);
        }
        new_path
    }

    /// Migrates any legacy material categories ("Wood & Plywood", "Ceramics & Stone", "LaserBurn Oletus...", etc.)
    /// to the unified default library category "PhotonCAM Oletus (10W-20W)".
    pub fn migrate_legacy_categories(&mut self) -> bool {
        let legacy_categories = [
            "Wood & Plywood",
            "Ceramics & Stone",
            "Paper & Packaging",
            "Plastics & Acrylic",
            "Leather & Textiles",
            "LaserBurn Oletus (10W-20W)",
        ];
        let mut changed = false;
        for mat in &mut self.materials {
            if legacy_categories.iter().any(|&c| c.eq_ignore_ascii_case(&mat.category)) {
                mat.category = "PhotonCAM Oletus (10W-20W)".to_string();
                changed = true;
            }
        }
        changed
    }

    /// Load existing library from default path, or return fresh default factory library.
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            match Self::load_from_path(&path) {
                Ok(mut lib) => {
                    if lib.migrate_legacy_categories() {
                        let _ = lib.save_to_path(&path);
                    }
                    return lib;
                }
                Err(e) => {
                    eprintln!("[PhotonCAM] Failed to load {}: {}. Using default library.", path.display(), e);
                }
            }
        }
        let lib = Self::default_library();
        let _ = lib.save_to_path(&path);
        lib
    }

    /// Load library from specific JSON file path.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("JSON parse error: {}", e))
    }

    /// Save library to specific JSON file path.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Write error: {}", e))
    }

    /// Save to default path.
    pub fn save(&self) -> Result<(), String> {
        self.save_to_path(&Self::default_path())
    }

    /// Returns list of all unique categories present in the library, sorted.
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.materials.iter().map(|m| m.category.clone()).collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// Find material by name.
    pub fn find_material(&self, name: &str) -> Option<&MaterialEntry> {
        self.materials.iter().find(|m| m.name.eq_ignore_ascii_case(name))
    }

    /// Find material mutably by name.
    pub fn find_material_mut(&mut self, name: &str) -> Option<&mut MaterialEntry> {
        self.materials.iter_mut().find(|m| m.name.eq_ignore_ascii_case(name))
    }

    /// Find material by category and name.
    pub fn find_material_in_category(&self, category: &str, name: &str) -> Option<&MaterialEntry> {
        self.materials.iter().find(|m| {
            m.name.eq_ignore_ascii_case(name) && m.category.eq_ignore_ascii_case(category)
        })
    }

    /// Find material mutably by category and name.
    pub fn find_material_in_category_mut(&mut self, category: &str, name: &str) -> Option<&mut MaterialEntry> {
        self.materials.iter_mut().find(|m| {
            m.name.eq_ignore_ascii_case(name) && m.category.eq_ignore_ascii_case(category)
        })
    }

    /// Insert or update an operation preset.
    pub fn add_or_update_operation(
        &mut self,
        category: &str,
        material_name: &str,
        thickness_mm: f64,
        thickness_label: &str,
        operation: MaterialOperation,
    ) {
        if let Some(mat) = self.find_material_in_category_mut(category, material_name) {
            if let Some(thick) = mat.find_thickness_mut(thickness_mm) {
                if let Some(op) = thick.find_operation_mut(&operation.name) {
                    *op = operation;
                } else {
                    thick.operations.push(operation);
                }
            } else {
                mat.thicknesses.push(ThicknessEntry::new(
                    thickness_mm,
                    thickness_label,
                    vec![operation],
                ));
            }
        } else {
            let new_thick = ThicknessEntry::new(thickness_mm, thickness_label, vec![operation]);
            let new_mat = MaterialEntry::new(
                material_name,
                category,
                "",
                vec![new_thick],
            );
            self.materials.push(new_mat);
        }
    }

    /// Delete an operation preset within a specific category.
    pub fn delete_operation_in_category(&mut self, category: &str, material_name: &str, thickness_mm: f64, op_name: &str) -> bool {
        if let Some(mat) = self.find_material_in_category_mut(category, material_name) {
            if let Some(thick) = mat.find_thickness_mut(thickness_mm) {
                let initial = thick.operations.len();
                thick.operations.retain(|o| !o.name.eq_ignore_ascii_case(op_name));
                return thick.operations.len() < initial;
            }
        }
        false
    }

    /// Delete an operation preset.
    pub fn delete_operation(&mut self, material_name: &str, thickness_mm: f64, op_name: &str) -> bool {
        if let Some(mat) = self.find_material_mut(material_name) {
            if let Some(thick) = mat.find_thickness_mut(thickness_mm) {
                let initial = thick.operations.len();
                thick.operations.retain(|o| !o.name.eq_ignore_ascii_case(op_name));
                return thick.operations.len() < initial;
            }
        }
        false
    }

    /// Delete an entire thickness entry from a material.
    pub fn delete_thickness(&mut self, material_name: &str, thickness_mm: f64) -> bool {
        if let Some(mat) = self.find_material_mut(material_name) {
            let initial = mat.thicknesses.len();
            mat.thicknesses.retain(|t| (t.thickness_mm - thickness_mm).abs() >= 0.05);
            return mat.thicknesses.len() < initial;
        }
        false
    }

    /// Delete an entire material entry from the library.
    pub fn delete_material(&mut self, material_name: &str) -> bool {
        let initial = self.materials.len();
        self.materials.retain(|m| !m.name.eq_ignore_ascii_case(material_name));
        self.materials.len() < initial
    }

    /// Merge another MaterialLibrary into this one, updating existing matching entries
    /// and appending new materials, thicknesses, and operations.
    pub fn merge(&mut self, other: MaterialLibrary) {
        for other_mat in other.materials {
            if let Some(existing_mat) = self.materials.iter_mut().find(|m| {
                m.name.eq_ignore_ascii_case(&other_mat.name)
                    && (m.category.eq_ignore_ascii_case(&other_mat.category) || other_mat.category.is_empty() || m.category.is_empty())
            }) {
                if existing_mat.category.is_empty() && !other_mat.category.is_empty() {
                    existing_mat.category = other_mat.category.clone();
                }
                for other_thick in other_mat.thicknesses {
                    if let Some(existing_thick) = existing_mat.find_matching_thickness_mut(other_thick.thickness_mm, &other_thick.label) {
                        for other_op in other_thick.operations {
                            if let Some(existing_op) = existing_thick.find_operation_mut(&other_op.name) {
                                *existing_op = other_op;
                            } else {
                                existing_thick.operations.push(other_op);
                            }
                        }
                    } else {
                        existing_mat.thicknesses.push(other_thick);
                    }
                }
            } else {
                self.materials.push(other_mat);
            }
        }
    }

    /// Import presets from a LightBurn `.clb` XML format string.
    pub fn import_from_clb_str(xml_str: &str) -> Result<Self, String> {
        let doc = roxmltree::Document::parse(xml_str).map_err(|e| format!("XML parse error: {}", e))?;
        let root = doc.root_element();

        // Find library display name (e.g. <LightBurnLibrary DisplayName="Oma 10W">)
        let lib_display_name = root
            .attributes()
            .find(|a| a.name().eq_ignore_ascii_case("DisplayName") || a.name().eq_ignore_ascii_case("name"))
            .map(|a| a.value().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "LightBurn Imports".to_string());

        let mut library = MaterialLibrary {
            version: 1,
            laser_profile: lib_display_name.clone(),
            materials: Vec::new(),
        };

        // Find Material elements anywhere in XML (descendants)
        for mat_node in root.descendants().filter(|n| n.tag_name().name().eq_ignore_ascii_case("Material")) {
            let mat_name = mat_node
                .attributes()
                .find(|a| a.name().eq_ignore_ascii_case("name"))
                .map(|a| a.value().trim().to_string())
                .unwrap_or_else(|| "Unnamed Material".to_string());

            if mat_name.is_empty() {
                continue;
            }

            let category = mat_node
                .attributes()
                .find(|a| a.name().eq_ignore_ascii_case("category") || a.name().eq_ignore_ascii_case("group"))
                .map(|a| a.value().trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| lib_display_name.clone());

            // Collect all Entry nodes (both direct children of Material AND inside Thickness tags)
            let mut entries_to_process = Vec::new();

            for child in mat_node.children() {
                let tag = child.tag_name().name();
                if tag.eq_ignore_ascii_case("Entry") || tag.eq_ignore_ascii_case("Operation") {
                    entries_to_process.push((None, child));
                } else if tag.eq_ignore_ascii_case("Thickness") {
                    for sub_entry in child.children().filter(|n| {
                        let stag = n.tag_name().name();
                        stag.eq_ignore_ascii_case("Entry") || stag.eq_ignore_ascii_case("Operation") || stag.eq_ignore_ascii_case("CutSetting")
                    }) {
                        entries_to_process.push((Some(child), sub_entry));
                    }
                }
            }

            // Group entries by thickness: key = ((thickness_mm * 1000.0).round() as i64, thickness_label)
            let mut thickness_order: Vec<((i64, String), f64, String)> = Vec::new();
            let mut thickness_ops: std::collections::HashMap<(i64, String), Vec<MaterialOperation>> = std::collections::HashMap::new();

            for (p_thick, entry_node) in entries_to_process {
                let thick_raw = entry_node
                    .attributes()
                    .find(|a| a.name().eq_ignore_ascii_case("Thickness") || a.name().eq_ignore_ascii_case("thickness"))
                    .or_else(|| p_thick.and_then(|p| p.attributes().find(|a| a.name().eq_ignore_ascii_case("value") || a.name().eq_ignore_ascii_case("val"))))
                    .and_then(|a| a.value().parse::<f64>().ok())
                    .unwrap_or(0.0);

                let thick_val = if thick_raw < 0.0 { 0.0 } else { thick_raw };

                let no_thick_title = entry_node
                    .attributes()
                    .find(|a| a.name().eq_ignore_ascii_case("NoThickTitle") || a.name().eq_ignore_ascii_case("noThickTitle"))
                    .or_else(|| p_thick.and_then(|p| p.attributes().find(|a| a.name().eq_ignore_ascii_case("name") || a.name().eq_ignore_ascii_case("label"))))
                    .map(|a| a.value().trim().to_string());

                let thick_label = if thick_val > 0.0 {
                    if (thick_val * 10.0).fract().abs() > 0.01 {
                        format!("{:.2} mm", thick_val)
                    } else {
                        format!("{:.1} mm", thick_val)
                    }
                } else if let Some(nt) = no_thick_title.filter(|s| !s.is_empty()) {
                    nt
                } else {
                    "Single / Surface".to_string()
                };

                let op_name = entry_node
                    .attributes()
                    .find(|a| a.name().eq_ignore_ascii_case("Desc") || a.name().eq_ignore_ascii_case("desc") || a.name().eq_ignore_ascii_case("name") || a.name().eq_ignore_ascii_case("type"))
                    .map(|a| a.value().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "Cut".to_string());

                let cs_node = if entry_node.tag_name().name().eq_ignore_ascii_case("CutSetting") {
                    Some(entry_node)
                } else {
                    entry_node.children().find(|n| n.tag_name().name().eq_ignore_ascii_case("CutSetting"))
                };

                let cut_setting = if let Some(cs) = cs_node {
                    parse_clb_cut_setting(&cs, &op_name)
                } else {
                    CutSetting::default()
                };

                let notes = entry_node
                    .children()
                    .find(|n| n.tag_name().name().eq_ignore_ascii_case("Notes") || n.tag_name().name().eq_ignore_ascii_case("Description"))
                    .and_then(|n| n.text())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                let key = ((thick_val * 1000.0).round() as i64, thick_label.clone());
                if !thickness_ops.contains_key(&key) {
                    thickness_order.push((key.clone(), thick_val, thick_label.clone()));
                }
                thickness_ops.entry(key).or_default().push(MaterialOperation::new(&op_name, cut_setting, &notes));
            }

            let mut thicknesses = Vec::new();
            for (key, thick_val, thick_label) in thickness_order {
                if let Some(ops) = thickness_ops.remove(&key) {
                    if !ops.is_empty() {
                        thicknesses.push(ThicknessEntry::new(thick_val, &thick_label, ops));
                    }
                }
            }

            if !thicknesses.is_empty() {
                library.materials.push(MaterialEntry::new(&mat_name, &category, "", thicknesses));
            }
        }

        if library.materials.is_empty() {
            Err("No valid materials found in LightBurn XML".to_string())
        } else {
            Ok(library)
        }
    }

    /// Load library from any supported file format (.json or .clb).
    pub fn load_from_any_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
        let trimmed = content.trim_start();
        if trimmed.starts_with('<') || path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("clb")) {
            Self::import_from_clb_str(&content)
        } else {
            serde_json::from_str(&content).map_err(|e| format!("JSON parse error: {}", e))
        }
    }
}

/// Helper function to parse LightBurn XML `<CutSetting>` elements into LaserBurn's native `CutSetting`.
fn parse_clb_cut_setting(cs_node: &roxmltree::Node, op_name: &str) -> CutSetting {
    let mut setting = CutSetting::default();

    let find_child_attr = |tags: &[&str], attr_name: &str| -> Option<String> {
        for child in cs_node.children() {
            let child_tag = child.tag_name().name();
            if tags.iter().any(|&t| child_tag.eq_ignore_ascii_case(t)) {
                if let Some(val) = child.attributes().find(|a| a.name().eq_ignore_ascii_case(attr_name)).map(|a| a.value()) {
                    return Some(val.to_string());
                }
                if let Some(text) = child.text() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
        for attr in cs_node.attributes() {
            if tags.iter().any(|&t| attr.name().eq_ignore_ascii_case(t)) {
                return Some(attr.value().to_string());
            }
        }
        None
    };

    let get_val_f64 = |tags: &[&str], default: f64| -> f64 {
        find_child_attr(tags, "Value")
            .or_else(|| find_child_attr(tags, "val"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(default)
    };

    let get_val_u32 = |tags: &[&str], default: u32| -> u32 {
        find_child_attr(tags, "Value")
            .or_else(|| find_child_attr(tags, "val"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(default)
    };

    let cut_type = cs_node
        .attributes()
        .find(|a| a.name().eq_ignore_ascii_case("type"))
        .map(|a| a.value())
        .unwrap_or("Cut");

    let mut speed = get_val_f64(&["speed", "Speed"], 50.0);
    let is_mm_per_min = find_child_attr(&["speed", "Speed"], "mmPerMin")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if is_mm_per_min && speed > 0.0 {
        speed /= 60.0;
    }

    setting.speed = speed;
    setting.power_min = get_val_f64(&["minPower", "MinPower", "min_power"], 10.0);
    setting.power_max = get_val_f64(&["maxPower", "MaxPower", "power", "Power"], 80.0);
    setting.num_passes = get_val_u32(&["numPasses", "NumPasses", "passes"], 1);
    setting.z_offset = get_val_f64(&["zOffset", "ZOffset"], 0.0);
    setting.air_assist = get_val_u32(&["enableAir", "EnableAir", "airAssist", "AirAssist", "runBlower", "RunBlower"], 0) > 0;
    setting.kerf = get_val_f64(&["kerf", "Kerf"], 0.0);
    setting.line_interval = get_val_f64(&["interval", "Interval", "lineInterval"], 0.08);

    let op_lower = op_name.to_lowercase();
    if cut_type.eq_ignore_ascii_case("Image")
        || op_lower.contains("bitmap")
        || op_lower.contains("image")
        || op_lower.contains("photo")
        || op_lower.contains("kuva")
    {
        setting.mode = CutMode::Image;
    } else if cut_type.eq_ignore_ascii_case("Scan")
        || cut_type.eq_ignore_ascii_case("Scan+Cut")
        || ((op_lower.contains("fill") || op_lower.contains("engrav") || op_lower.contains("kaiver"))
            && !op_lower.contains("vector"))
    {
        setting.mode = CutMode::Fill;
    } else {
        setting.mode = CutMode::Line;
    }

    if let Some(dither_str) = find_child_attr(&["ditherMode", "DitherMode"], "Value") {
        match dither_str.to_lowercase().as_str() {
            "grayscale" => setting.dither_mode = DitherMode::GrayscalePower,
            "halftone" => setting.dither_mode = DitherMode::Halftone,
            "atkinson" => setting.dither_mode = DitherMode::Atkinson,
            "floydsteinberg" | "floyd" => setting.dither_mode = DitherMode::FloydSteinberg,
            "stucki" => setting.dither_mode = DitherMode::Stucki,
            "jarvis" => setting.dither_mode = DitherMode::Jarvis,
            "threshold" => setting.dither_mode = DitherMode::Threshold,
            "ordered" | "bayer" => setting.dither_mode = DitherMode::OrderedBayer,
            _ => {}
        }
    }

    setting
}

impl MaterialLibrary {

    /// Built-in curated library with factory presets for standard 10W-20W diode lasers.
    pub fn default_library() -> Self {
        const DEFAULT_CAT: &str = "PhotonCAM Oletus (10W-20W)";
        let mut materials = Vec::new();

        // 1. Wood & Plywood (Birch Plywood)
        {
            let mut cut_3mm = CutSetting::default();
            cut_3mm.mode = CutMode::Line;
            cut_3mm.speed = 3.0; // mm/s
            cut_3mm.power_min = 100.0;
            cut_3mm.power_max = 100.0;
            cut_3mm.num_passes = 2;
            cut_3mm.air_assist = true;
            cut_3mm.kerf = 0.10;

            let mut score_3mm = CutSetting::default();
            score_3mm.mode = CutMode::Line;
            score_3mm.speed = 35.0; // mm/s
            score_3mm.power_min = 20.0;
            score_3mm.power_max = 25.0;
            score_3mm.num_passes = 1;
            score_3mm.air_assist = false;

            let mut engrave_3mm = CutSetting::default();
            engrave_3mm.mode = CutMode::Fill;
            engrave_3mm.speed = 50.0; // mm/s
            engrave_3mm.power_min = 30.0;
            engrave_3mm.power_max = 50.0;
            engrave_3mm.line_interval = 0.08;

            let mut photo_3mm = CutSetting::default();
            photo_3mm.mode = CutMode::Image;
            photo_3mm.speed = 75.0;
            photo_3mm.power_min = 15.0;
            photo_3mm.power_max = 65.0;
            photo_3mm.dither_mode = DitherMode::Atkinson;
            photo_3mm.overscan = 3.0;

            let thick_3mm = ThicknessEntry::new(
                3.0,
                "3.0 mm",
                vec![
                    MaterialOperation::new("Clean Cut", cut_3mm, "Clean through cut on honeycomb bed with air assist ON"),
                    MaterialOperation::new("Vector Score", score_3mm.clone(), "Crisp surface outline markings"),
                    MaterialOperation::new("Dark Engrave", engrave_3mm.clone(), "Rich dark raster pocket fill"),
                    MaterialOperation::new("Photo Engrave", photo_3mm, "High-contrast portrait dithering (Atkinson)"),
                ],
            );

            let mut cut_4mm = CutSetting::default();
            cut_4mm.mode = CutMode::Line;
            cut_4mm.speed = 2.0;
            cut_4mm.power_min = 100.0;
            cut_4mm.power_max = 100.0;
            cut_4mm.num_passes = 3;
            cut_4mm.air_assist = true;
            cut_4mm.kerf = 0.12;

            let thick_4mm = ThicknessEntry::new(
                4.0,
                "4.0 mm",
                vec![
                    MaterialOperation::new("Clean Cut", cut_4mm, "Deep vector through cut with air assist"),
                    MaterialOperation::new("Vector Score", score_3mm.clone(), "Crisp surface markings"),
                    MaterialOperation::new("Dark Engrave", engrave_3mm.clone(), "Solid dark engraving"),
                ],
            );

            materials.push(MaterialEntry::new(
                "Birch Plywood (Koivuvaneri)",
                DEFAULT_CAT,
                "Finnish birch plywood (AB/B grade). Optimal for furniture, puzzle boxes, and wall art.",
                vec![thick_3mm, thick_4mm],
            ));
        }

        // 2. Ceramic Tiles (Keramiikkalaatat)
        {
            let mut spray_gray = CutSetting::default();
            spray_gray.mode = CutMode::Image;
            spray_gray.speed = 60.0;
            spray_gray.power_min = 10.0;
            spray_gray.power_max = 85.0;
            spray_gray.dither_mode = DitherMode::GrayscalePower;
            spray_gray.invert_image = true;
            spray_gray.overscan = 3.0;

            let mut paint_stack = CutSetting::default();
            paint_stack.mode = CutMode::Image;
            paint_stack.speed = 50.0;
            paint_stack.power_min = 15.0;
            paint_stack.power_max = 95.0;
            paint_stack.dither_mode = DitherMode::GrayscalePower;
            paint_stack.invert_image = false;
            paint_stack.ceramic_preview = true;
            paint_stack.overscan = 3.0;

            let mut norton_white = CutSetting::default();
            norton_white.mode = CutMode::Image;
            norton_white.speed = 40.0;
            norton_white.power_min = 30.0;
            norton_white.power_max = 90.0;
            norton_white.dither_mode = DitherMode::Atkinson;
            norton_white.overscan = 3.0;

            let tile_thick = ThicknessEntry::new(
                0.0,
                "Standard Tile (100x100mm)",
                vec![
                    MaterialOperation::new("Black Spray Paint (Grayscale)", spray_gray, "Single black matte spray coat; laser ablates down to white glaze"),
                    MaterialOperation::new("Multi-Color Paint Stack", paint_stack, "Stacked spray layers (Black -> Red -> Yellow -> White tile base)"),
                    MaterialOperation::new("Norton White Tile (TiO2)", norton_white, "Direct titanium dioxide paint burn creating black ceramic fusion"),
                ],
            );

            materials.push(MaterialEntry::new(
                "Ceramic Tile (Keramiikkalaatta)",
                DEFAULT_CAT,
                "Glazed white bathroom/kitchen tiles with spray paint or TiO2 coating for photo engraving.",
                vec![tile_thick],
            ));
        }

        // 3. Cardboard & Kraft Paper (Pahvi & Paperi)
        {
            let mut cut_card = CutSetting::default();
            cut_card.mode = CutMode::Line;
            cut_card.speed = 20.0;
            cut_card.power_min = 60.0;
            cut_card.power_max = 75.0;
            cut_card.num_passes = 1;
            cut_card.air_assist = true;

            let mut crease_card = CutSetting::default();
            crease_card.mode = CutMode::Line;
            crease_card.speed = 60.0;
            crease_card.power_min = 15.0;
            crease_card.power_max = 20.0;
            crease_card.num_passes = 1;
            crease_card.air_assist = false;

            let thick_2mm_card = ThicknessEntry::new(
                2.0,
                "2.0 mm Corrugated",
                vec![
                    MaterialOperation::new("Fast Cut", cut_card, "High-speed clean cut through corrugated cardboard"),
                    MaterialOperation::new("Fold Crease / Score", crease_card, "Light surface kiss-cut for precise folding"),
                ],
            );

            let mut cut_kraft = CutSetting::default();
            cut_kraft.mode = CutMode::Line;
            cut_kraft.speed = 40.0;
            cut_kraft.power_min = 30.0;
            cut_kraft.power_max = 40.0;
            cut_kraft.num_passes = 1;

            let thick_05mm = ThicknessEntry::new(
                0.5,
                "0.5 mm Cardstock",
                vec![
                    MaterialOperation::new("Paper Cut", cut_kraft, "High-speed paper/kraft card cut"),
                ],
            );

            materials.push(MaterialEntry::new(
                "Cardboard & Paper (Pahvi)",
                DEFAULT_CAT,
                "Corrugated shipping boxes, kraft paper, and chipboard for architectural prototyping.",
                vec![thick_2mm_card, thick_05mm],
            ));
        }

        // 4. Cast Acrylic (Akryyli)
        {
            let mut cut_acryl = CutSetting::default();
            cut_acryl.mode = CutMode::Line;
            cut_acryl.speed = 2.5;
            cut_acryl.power_min = 100.0;
            cut_acryl.power_max = 100.0;
            cut_acryl.num_passes = 2;
            cut_acryl.air_assist = true;
            cut_acryl.kerf = 0.12;

            let mut frost_acryl = CutSetting::default();
            frost_acryl.mode = CutMode::Fill;
            frost_acryl.speed = 70.0;
            frost_acryl.power_min = 20.0;
            frost_acryl.power_max = 35.0;
            frost_acryl.line_interval = 0.06;

            let thick_3mm_acryl = ThicknessEntry::new(
                3.0,
                "3.0 mm Dark / Black",
                vec![
                    MaterialOperation::new("Flame Polished Cut", cut_acryl, "Glossy edge finish on opaque dark acrylic"),
                    MaterialOperation::new("Frost Engrave", frost_acryl, "Smooth satin white frosted surface engraving"),
                ],
            );

            materials.push(MaterialEntry::new(
                "Cast Acrylic (Valuakryyli)",
                DEFAULT_CAT,
                "Opaque black and dark colored acrylic sheet. Note: Diode lasers require opaque/colored acrylic.",
                vec![thick_3mm_acryl],
            ));
        }

        // 5. Slate Stone (Liuskekivi)
        {
            let mut slate_photo = CutSetting::default();
            slate_photo.mode = CutMode::Image;
            slate_photo.speed = 65.0;
            slate_photo.power_min = 20.0;
            slate_photo.power_max = 60.0;
            slate_photo.dither_mode = DitherMode::Atkinson;
            slate_photo.overscan = 2.5;

            let mut slate_fill = CutSetting::default();
            slate_fill.mode = CutMode::Fill;
            slate_fill.speed = 50.0;
            slate_fill.power_min = 30.0;
            slate_fill.power_max = 50.0;
            slate_fill.line_interval = 0.08;

            let slate_thick = ThicknessEntry::new(
                5.0,
                "5.0 mm Coaster",
                vec![
                    MaterialOperation::new("Photo Portrait", slate_photo, "Crisp bright white fracture etching on natural dark slate"),
                    MaterialOperation::new("Vector Logo Engrave", slate_fill, "Solid clean vector text and logo pocketing"),
                ],
            );

            materials.push(MaterialEntry::new(
                "Slate Stone (Liuskekivi)",
                DEFAULT_CAT,
                "Natural grey/black slate drink coasters and signage. Fractures into brilliant white marks.",
                vec![slate_thick],
            ));
        }

        // 6. Leather (Kasviparkittu nahka)
        {
            let mut cut_leather = CutSetting::default();
            cut_leather.mode = CutMode::Line;
            cut_leather.speed = 6.0;
            cut_leather.power_min = 80.0;
            cut_leather.power_max = 90.0;
            cut_leather.num_passes = 1;
            cut_leather.air_assist = true;

            let mut brand_leather = CutSetting::default();
            brand_leather.mode = CutMode::Fill;
            brand_leather.speed = 60.0;
            brand_leather.power_min = 15.0;
            brand_leather.power_max = 30.0;
            brand_leather.line_interval = 0.10;

            let thick_leather = ThicknessEntry::new(
                2.0,
                "2.0 mm Veg-Tan",
                vec![
                    MaterialOperation::new("Clean Cut", cut_leather, "Smooth edge cut with air assist to minimize soot"),
                    MaterialOperation::new("Branded Engrave", brand_leather, "Warm embossed branding effect"),
                ],
            );

            materials.push(MaterialEntry::new(
                "Vegetable Tanned Leather (Nahka)",
                DEFAULT_CAT,
                "Natural tooling leather for wallets, patches, belts, and craft items.",
                vec![thick_leather],
            ));
        }

        Self {
            version: 1,
            laser_profile: default_profile(),
            materials,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_material_library() {
        let lib = MaterialLibrary::default_library();
        assert!(!lib.materials.is_empty());
        assert!(lib.materials.len() >= 6);

        let birch = lib.find_material("Birch Plywood (Koivuvaneri)").expect("Birch Plywood must exist");
        assert_eq!(birch.category, "PhotonCAM Oletus (10W-20W)");
        let thick_3 = birch.find_thickness(3.0).expect("3.0 mm must exist");
        let cut = thick_3.find_operation("Clean Cut").expect("Clean Cut must exist");
        assert_eq!(cut.cut_setting.speed, 3.0);
        assert_eq!(cut.cut_setting.num_passes, 2);
    }

    #[test]
    fn test_apply_to_layer() {
        let lib = MaterialLibrary::default_library();
        let birch = lib.find_material("Birch Plywood (Koivuvaneri)").unwrap();
        let thick_3 = birch.find_thickness(3.0).unwrap();
        let cut_op = thick_3.find_operation("Clean Cut").unwrap();

        let mut layer = Layer::new(0, "C00", "#000000");
        cut_op.apply_to_layer(&mut layer);

        assert_eq!(layer.cut_setting.speed, 3.0);
        assert_eq!(layer.cut_setting.power_max, 100.0);
        assert_eq!(layer.cut_setting.num_passes, 2);
        assert_eq!(layer.cut_setting.air_assist, true);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let lib = MaterialLibrary::default_library();
        let json = serde_json::to_string_pretty(&lib).expect("Must serialize to JSON");
        assert!(json.contains("Birch Plywood"));
        assert!(json.contains("Ceramic Tile"));

        let deserialized: MaterialLibrary = serde_json::from_str(&json).expect("Must deserialize from JSON");
        assert_eq!(lib, deserialized);
    }

    #[test]
    fn test_add_and_delete_operation() {
        let mut lib = MaterialLibrary::default_library();
        let mut custom_setting = CutSetting::default();
        custom_setting.speed = 12.3;

        let op = MaterialOperation::new("Test Op", custom_setting, "Notes");
        lib.add_or_update_operation("Custom Cat", "Custom Mat", 5.0, "5.0 mm", op);

        let mat = lib.find_material("Custom Mat").expect("Should find Custom Mat");
        let thick = mat.find_thickness(5.0).expect("Should find 5.0 mm");
        let found_op = thick.find_operation("Test Op").expect("Should find Test Op");
        assert_eq!(found_op.cut_setting.speed, 12.3);

        // Delete operation
        assert!(lib.delete_operation("Custom Mat", 5.0, "Test Op"));
        let thick_after = lib.find_material("Custom Mat").unwrap().find_thickness(5.0).unwrap();
        assert!(thick_after.find_operation("Test Op").is_none());

        // Delete thickness
        assert!(lib.delete_thickness("Custom Mat", 5.0));
        assert!(lib.find_material("Custom Mat").unwrap().find_thickness(5.0).is_none());

        // Delete material
        assert!(lib.delete_material("Custom Mat"));
        assert!(lib.find_material("Custom Mat").is_none());
    }

    #[test]
    fn test_import_lightburn_clb() {
        let sample_clb = r#"<?xml version="1.0" encoding="UTF-8"?>
<LightBurnLibrary>
    <Material Name="Basswood" Category="Hardwood">
        <Thickness Name="3.0 mm" Value="3.0">
            <Entry Name="Through Cut">
                <CutSetting type="Cut">
                    <Speed Value="12.0"/>
                    <MaxPower Value="85.0"/>
                    <MinPower Value="15.0"/>
                    <NumPasses Value="1"/>
                    <AirAssist Value="1"/>
                    <Kerf Value="0.12"/>
                </CutSetting>
                <Notes>Clean basswood sheet cut</Notes>
            </Entry>
            <Entry Name="Surface Scan">
                <CutSetting type="Scan">
                    <Speed Value="80.0"/>
                    <MaxPower Value="40.0"/>
                    <Interval Value="0.08"/>
                </CutSetting>
            </Entry>
        </Thickness>
    </Material>
</LightBurnLibrary>"#;

        let lib = MaterialLibrary::import_from_clb_str(sample_clb).expect("Must parse LightBurn XML");
        assert_eq!(lib.materials.len(), 1);
        let mat = &lib.materials[0];
        assert_eq!(mat.name, "Basswood");
        assert_eq!(mat.category, "Hardwood");

        let thick = mat.find_thickness(3.0).expect("Must find 3.0mm thickness");
        assert_eq!(thick.operations.len(), 2);

        let cut_op = thick.find_operation("Through Cut").expect("Must find Through Cut");
        assert_eq!(cut_op.cut_setting.speed, 12.0);
        assert_eq!(cut_op.cut_setting.power_max, 85.0);
        assert_eq!(cut_op.cut_setting.air_assist, true);
        assert_eq!(cut_op.cut_setting.mode, CutMode::Line);
        assert_eq!(cut_op.notes, "Clean basswood sheet cut");

        let scan_op = thick.find_operation("Surface Scan").expect("Must find Surface Scan");
        assert_eq!(scan_op.cut_setting.speed, 80.0);
        assert_eq!(scan_op.cut_setting.mode, CutMode::Fill);
    }

    #[test]
    fn test_import_real_lightburn_clb_flat_entries() {
        let sample_clb = r#"<?xml version="1.0" encoding="UTF-8"?>
<LightBurnLibrary DisplayName="Oma 10W">
    <Material name="Balsa">
        <Entry Thickness="2.0000" Desc="Kaiverus">
            <CutSetting type="Cut">
                <maxPower Value="100"/>
                <speed Value="83.3333"/>
                <priority Value="0"/>
            </CutSetting>
        </Entry>
        <Entry Thickness="2.0000" Desc="Leikkaus">
            <CutSetting type="Cut">
                <maxPower Value="50"/>
                <speed Value="3.33333"/>
                <runBlower Value="1"/>
            </CutSetting>
        </Entry>
    </Material>
    <Material name="Titaanioksidi laatta">
        <Entry Thickness="-1.0000" Desc="Kaiverrus" NoThickTitle="B/W kuva">
            <CutSetting type="Image">
                <minPower Value="5"/>
                <maxPower Value="40"/>
                <speed Value="33.3333"/>
                <interval Value="0.0801"/>
                <ditherMode Value="halftone"/>
            </CutSetting>
        </Entry>
    </Material>
</LightBurnLibrary>"#;

        let lib = MaterialLibrary::import_from_clb_str(sample_clb).expect("Must parse real LightBurn XML");
        assert_eq!(lib.laser_profile, "Oma 10W");
        assert_eq!(lib.materials.len(), 2);

        // 1. Verify Balsa (grouped 2mm thickness with 2 operations)
        let balsa = lib.find_material("Balsa").expect("Must find Balsa");
        assert_eq!(balsa.category, "Oma 10W");
        assert_eq!(balsa.thicknesses.len(), 1);

        let thick_2 = balsa.find_thickness(2.0).expect("Must find 2.0mm thickness");
        assert_eq!(thick_2.operations.len(), 2);

        let engrave_op = thick_2.find_operation("Kaiverus").expect("Must find Kaiverus");
        assert_eq!(engrave_op.cut_setting.speed, 83.3333);
        assert_eq!(engrave_op.cut_setting.power_max, 100.0);
        assert_eq!(engrave_op.cut_setting.mode, CutMode::Fill); // Inferred from "Kaiverus"

        let cut_op = thick_2.find_operation("Leikkaus").expect("Must find Leikkaus");
        assert_eq!(cut_op.cut_setting.speed, 3.33333);
        assert_eq!(cut_op.cut_setting.power_max, 50.0);
        assert_eq!(cut_op.cut_setting.air_assist, true); // runBlower Value="1"
        assert_eq!(cut_op.cut_setting.mode, CutMode::Line);

        // 2. Verify Titaanioksidi laatta (Thickness -1.0 with NoThickTitle "B/W kuva")
        let tile = lib.find_material("Titaanioksidi laatta").expect("Must find Titaanioksidi laatta");
        assert_eq!(tile.thicknesses.len(), 1);
        let thick_tile = &tile.thicknesses[0];
        assert_eq!(thick_tile.thickness_mm, 0.0);
        assert_eq!(thick_tile.label, "B/W kuva");

        let photo_op = thick_tile.find_operation("Kaiverrus").expect("Must find Kaiverrus");
        assert_eq!(photo_op.cut_setting.mode, CutMode::Image);
        assert_eq!(photo_op.cut_setting.dither_mode, DitherMode::Halftone);
        assert_eq!(photo_op.cut_setting.power_min, 5.0);
        assert_eq!(photo_op.cut_setting.power_max, 40.0);
    }

    #[test]
    fn test_merge_libraries() {
        let mut base_lib = MaterialLibrary::default_library();
        let initial_count = base_lib.materials.len();

        let mut incoming_lib = MaterialLibrary {
            version: 1,
            laser_profile: "Imported".to_string(),
            materials: Vec::new(),
        };

        let mut custom_setting = CutSetting::default();
        custom_setting.speed = 99.0;
        let op = MaterialOperation::new("Super Cut", custom_setting, "");
        let thick = ThicknessEntry::new(3.0, "3.0 mm", vec![op]);
        let mat = MaterialEntry::new("Birch Plywood (Koivuvaneri)", "PhotonCAM Oletus (10W-20W)", "", vec![thick]);
        incoming_lib.materials.push(mat);

        base_lib.merge(incoming_lib);
        assert_eq!(base_lib.materials.len(), initial_count, "Should not duplicate Birch Plywood");

        let birch = base_lib.find_material("Birch Plywood (Koivuvaneri)").unwrap();
        let thick_3 = birch.find_thickness(3.0).unwrap();
        assert!(thick_3.find_operation("Super Cut").is_some(), "Super Cut must be merged into existing 3mm thickness");
    }

    #[test]
    fn test_migrate_legacy_categories() {
        let mut lib = MaterialLibrary {
            version: 1,
            laser_profile: "Legacy".to_string(),
            materials: vec![
                MaterialEntry::new("Birch Plywood", "Wood & Plywood", "", Vec::new()),
                MaterialEntry::new("Tile", "Ceramics & Stone", "", Vec::new()),
                MaterialEntry::new("Acrylic", "Plastics & Acrylic", "", Vec::new()),
                MaterialEntry::new("Old LaserBurn", "LaserBurn Oletus (10W-20W)", "", Vec::new()),
                MaterialEntry::new("Custom", "My Custom Machine", "", Vec::new()),
            ],
        };

        assert!(lib.migrate_legacy_categories());
        assert_eq!(lib.materials[0].category, "PhotonCAM Oletus (10W-20W)");
        assert_eq!(lib.materials[1].category, "PhotonCAM Oletus (10W-20W)");
        assert_eq!(lib.materials[2].category, "PhotonCAM Oletus (10W-20W)");
        assert_eq!(lib.materials[3].category, "PhotonCAM Oletus (10W-20W)");
        assert_eq!(lib.materials[4].category, "My Custom Machine");
        assert!(!lib.migrate_legacy_categories(), "Second run should make no changes");
    }
}
