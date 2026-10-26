//! Internationalization (i18n) localization module for LaserBurn.
//! Provides bilingual support for Finnish (Suomi) and English.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Finnish,
}

impl Language {
    pub fn label(&self) -> &'static str {
        match self {
            Language::English => "English (EN)",
            Language::Finnish => "Suomi (FI)",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Finnish => "fi",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::Finnish
    }
}

/// Translates a given key into the requested language
pub fn t(key: &'static str, lang: Language) -> &'static str {
    match lang {
        Language::English => key,
        Language::Finnish => translate_fi(key),
    }
}

fn translate_fi(key: &'static str) -> &'static str {
    match key {
        // File Menu
        "File" => "Tiedosto",
        "New Project" => "Uusi projekti",
        "Open File / Project..." => "Avaa tiedosto / projekti...",
        "Import / Merge File (DXF, PDF, SVG, Image)..." => "Tuo / yhdistä tiedosto (DXF, PDF, SVG, kuva)...",
        "Save Project (.lbrn2)..." => "Tallenna projekti (.lbrn2)...",
        "Sample Files" => "Esimerkkitiedostot",
        "AutoCAD DXF (Mechanical Bracket)" => "AutoCAD DXF (Mekaaninen kiinnike)",
        "Vector PDF (Laser Test Card)" => "Vektori-PDF (Lasertestikortti)",
        "Export G-Code (GRBL)..." => "Vie G-koodi (GRBL)...",
        "Export Ruida (.rd)..." => "Vie Ruida (.rd)...",
        "Export Settings Profile..." => "Vie asetusprofiili...",
        "Import Settings Profile..." => "Tuo asetusprofiili...",
        "Connection Settings..." => "Yhteysasetukset...",
        "Settings..." => "Asetukset...",

        // Edit Menu
        "Edit" => "Muokkaa",
        "Undo" => "Kumoa",
        "Redo" => "Tee uudelleen",
        "Cut" => "Leikkaa",
        "Copy" => "Kopioi",
        "Paste" => "Liitä",
        "Duplicate" => "Kahdenna",
        "Delete" => "Poista",
        "Offset Shapes (Contour)..." => "Muotojen ääriviivasiirto (Offset)...",
        "Group" => "Ryhmittele",
        "Ungroup" => "Pura ryhmä",
        "Lock" => "Lukitse",
        "Unlock" => "Avaa lukitus",
        "Toggle Lock" => "Lukitse / Avaa lukitus",
        "Boolean Union / Weld" => "Yhdistä (Union / Weld)",
        "Boolean Subtract (A - B)" => "Erotus (A - B)",
        "Boolean Intersect" => "Leikkaus (Intersect)",
        "Grid Array (Matrix)..." => "Ruudukkomonistus (Matrix)...",
        "Radial Array (Circular)..." => "Kehämonistus (Ympyrä)...",
        "Trace Image (Bitmap to Vector)..." => "Bittikartta vektoriksi (Trace)...",
        "Add Text..." => "Lisää tekstiä...",
        "Select All" => "Valitse kaikki",
        "Clear Canvas" => "Tyhjennä työpöytä",
        "Material Test Matrix Generator..." => "Materiaalitestitaulukko...",
        "Reset All Zoom & Pan" => "Palauta zoomaus ja näkymä",

        // Laser Menu
        "Laser" => "Laser",
        "Material Library (Presets)..." => "Materiaalikirjasto...",
        "Material Test Generator (Power vs. Speed)..." => "Materiaalitestigeneraattori...",
        "Laser Control & Jogging" => "Laserin ohjaus ja siirto",
        "Turn Off Weak Beam" => "Sammuta kohdistussäde",
        "Turn On Weak Beam" => "Kytke kohdistussäde päälle",
        "Test Box Frame" => "Rajauksen testaus (Suorakulmio)",
        "Test Cut Contour (Shapes)" => "Rajauksen testaus (Muodot)",
        "Reset Origin to Limit Switches" => "Palauta origo rajakytkimiin",
        "Home Machine ($H)" => "Aja kotiin ($H)",
        "Set Origin Here (G92)" => "Aseta origo tähän (G92)",
        "Hide Data Traffic Log Panel" => "Piilota sarjaliikenneloki",
        "Show Data Traffic Log Panel" => "Näytä sarjaliikenneloki",

        // Camera Menu
        "Camera" => "Kamera",
        "Camera Feed..." => "Kameranäkymä...",
        "Scan Devices" => "Etsi kameroita",
        "Start Stream" => "Käynnistä videolähetys",
        "Stop Stream" => "Pysäytä videolähetys",
        "Update Bed Overlay (Cmd+U)" => "Päivitä taustakuva (Cmd+U)",
        "Auto-Center Selection on Camera Tile" => "Keskitä valinta kameraruutuun",
        "Auto-Fit & Scale Selection to Camera Tile (2mm Margin)" => "Sovita ja skaalaa kameraruutuun (2mm marginaali)",

        // Main Toolbar
        "Select / Move" => "Valitse / Siirrä",
        "Edit Nodes (N)" => "Muokkaa solmuja (N)",
        "Rect" => "Suorakulmio",
        "Circle / Oval" => "Ympyrä / Ovaali",
        "Line" => "Viiva",
        "Text" => "Teksti",
        "Import File" => "Tuo tiedosto",
        "Trace Image" => "Bittikartta vektoriksi",
        "Grid Array" => "Ruudukko",
        "Radial Array" => "Kehä",
        "Offset" => "Ääriviivasiirto",
        "Weld" => "Yhdistä",
        "Subtract" => "Erotus",
        "Intersect" => "Leikkaus",
        "Theme: Dark" => "Teema: Tumma",
        "Theme: Light" => "Teema: Vaalea",
        "↩ Undo" => "↩ Kumoa",
        "↪ Redo" => "↪ Tee uudelleen",

        // Laser Control Panel
        "Laser Control" => "Laserin ohjaus",
        "Laser Status" => "Laserin tila",
        "Disconnected" => "Ei yhteyttä",
        "Connecting..." => "Yhdistetään...",
        "Connected" => "Yhdistetty",
        "Idle" => "Valmiustilassa",
        "Run" => "Työ käynnissä",
        "Hold" => "Keskeytetty",
        "Alarm" => "Hälytystila",
        "Laser Jogging" => "Laserin ohjaus (Jog)",
        "Jog Laser" => "Laserpään siirto (Jog)",
        "Step:" => "Askel:",
        "Speed:" => "Nopeus:",
        "Power:" => "Teho:",
        "Start" => "Aloita",
        "Pause" => "Tauko",
        "Resume" => "Jatka",
        "Stop" => "Pysäytä",
        "Frame" => "Rajaa",
        "Home" => "Koti",
        "Zero" => "Nollaa",
        "Weak Light (Aim)" => "Kohdistusvalo (Aim)",
        "Weak Light Active" => "Kohdistusvalo päällä",
        "Connect" => "Yhdistä",
        "Disconnect" => "Katkaise yhteys",
        "Ports:" => "Portit:",
        "Baud:" => "Nopeus:",
        "Open Jog Window" => "Avaa siirtoikkuna",
        "Keyboard Arrow Jogging" => "Näppäimistön nuolisiirto",
        "Switches:" => "Kytkimet:",
        "Home Speed:" => "Kotinopetus:",

        // Job Execution
        "Job Execution (G-Code)" => "Työn ajo (G-koodi)",
        "Estimated Time:" => "Arvioitu aika:",
        " START JOB (CUT) " => " ALOITA TYÖ (LEIKKAUS) ",
        " RESUME " => " JATKA ",
        " PAUSE " => " TAUKO ",
        " STOP " => " PYSÄYTÄ ",
        "JOB PAUSED" => "TYÖ KESKEYTETTY",
        "Elapsed:" => "Kulunut aika:",
        "Remaining:" => "Jäljellä:",

        // Cuts / Layers Table
        "Cuts / Layers" => "Tasot / Leikkaukset",
        "Cut & Layer Settings" => "Tasot & Leikkausasetukset",
        "Layer" => "Taso",
        "Name" => "Nimi",
        "Mode" => "Tila",
        "Speed (mm/s)" => "Nopeus (mm/s)",
        "Power (%)" => "Teho (%)",
        "Passes" => "Kierrokset",
        "Air Assist" => "Ilma-avustin",
        "Output" => "Leikkaa",
        "Show" => "Näytä",
        "Line (Cut)" => "Viiva (Leikkaus)",
        "Fill (Engrave)" => "Täyttö (Kaiverrus)",
        "Offset Fill" => "Sisäkkäinen täyttö",
        "Image (Dither)" => "Kuva (Rasteri)",
        "Library" => "Kirjasto",
        "Save Palette" => "Tallenna paletti",
        "+ Layer" => "+ Taso",
        "+ Tool" => "+ Aputaso",
        "- Remove" => "- Poista",

        // Shape Properties & Transform Panel
        "Transform & Size" => "Muunnos & Koko",
        "Shape Properties" => "Muodon ominaisuudet",
        "Position" => "Sijainti",
        "Dimensions" => "Mitat",
        "Width" => "Leveys",
        "Height" => "Korkeus",
        "Width (mm):" => "Leveys (mm):",
        "Height (mm):" => "Korkeus (mm):",
        "X (mm):" => "X (mm):",
        "Y (mm):" => "Y (mm):",
        "Rotation" => "Kierto",
        "Scale" => "Skaalaus",
        "Lock Aspect" => "Lukitse kuvasuhde",
        "Lock Aspect Ratio" => "Lukitse kuvasuhde",
        "Nudge (5mm):" => "Siirrä (5mm):",
        "Deselect" => "Poista valinta",
        "Objects List" => "Objektilista",

        // Material Library Panel
        "Material Library" => "Materiaalikirjasto",
        "Apply to Layer" => "Käytä valittuun tasoon",
        "Add New Material" => "Lisää uusi materiaali",
        "Categories" => "Kategoriat",
        "Thickness" => "Paksuus",

        // Dialogs & Windows
        "Trace Raster Image to Vectors" => "Muunna bittikarttakuva vektoreiksi",
        "Threshold (0..255):" => "Kynnysarvo (0..255):",
        "Blur / Smoothness:" => "Sumennus / Pehmennys:",
        "Minimum Area:" => "Minimipinta-ala:",
        "Curve Tolerance:" => "Käyrätoleranssi:",
        "Invert (trace bright pixels instead of dark)" => "Käänteinen (etsi vaaleita pikseleitä tummien sijaan)",
        "Replace original bitmap with vector result" => "Korvaa alkuperäinen kuva vektorituloksella",
        "Live Preview:" => "Reaaliaikainen esikatselu:",
        "Trace & Create Vectors" => "Hyväksy & Luo vektorit",
        "Cancel" => "Peruuta",
        "Apply" => "Käytä",
        "Close" => "Sulje",
        "Save" => "Tallenna",

        // Grid Array Dialog
        "Duplicate Grid Array" => "Ruudukkomonistus",
        "Grid Array (Matrix)" => "Ruudukkomonistus (Matrix)",
        "Rows:" => "Rivejä:",
        "Columns:" => "Sarakkeita:",
        "Spacing X:" => "X-väli:",
        "Spacing Y:" => "Y-väli:",
        "Total Count:" => "Kappalemäärä yhteensä:",

        // Radial Array Dialog
        "Duplicate Radial Array" => "Kehämonistus",
        "Radial Array (Circular)" => "Kehämonistus (Ympyrä)",
        "Copies:" => "Kopioita:",
        "Total Angle:" => "Kokonaiskulma:",
        "Rotate copies" => "Kierrä kopioita kulman mukana",
        "Center Point:" => "Keskipiste:",
        "Auto Center" => "Automaattinen keskipiste",

        // Profiles Window
        "Manage Machine Profiles" => "Hallitse laiteprofiileja",
        "Laser Machine Profiles" => "Laserkoneiden laiteprofiilit",
        "ACTIVE" => "AKTIIVINEN",
        "Activate" => "Ota käyttöön",
        "Create Profile" => "Luo uusi profiili",
        "Bed:" => "Alusta:",

        // Settings Window
        "Device & Bed Settings" => "Laite- ja työpöytäasetukset",
        "LaserBurn Settings" => "LaserBurn Asetukset",
        "Language:" => "Kieli:",
        "Machine Profile:" => "Laiteprofiili:",
        "Manage Profiles..." => "Hallitse profiileja...",
        "Bed Dimensions (mm):" => "Työalueen mitat (mm):",
        "Width:" => "Leveys:",
        "Height:" => "Korkeus:",
        "Origin Corner:" => "Origon nurkka:",
        "Laser & Beam Parameters:" => "Laser- ja sädeparametrit:",
        "Laser Type:" => "Laserin tyyppi:",
        "Max Power ($30 / S-max):" => "Maksimiteho ($30 / S-max):",
        "Weak Beam Power for Alignment:" => "Kohdistussäteen teho:",
        "Beam Kerf (Full Width):" => "Säteen leveys (Kerf):",
        "Speed Limits:" => "Nopeusrajat:",
        "Rapid Speed G0:" => "Pikasiirtonopeus G0:",
        "Jog Speed:" => "Siirtonopeus (Jog):",
        "Framing Test Speed:" => "Rajausnopeus:",
        "Limit Switches:" => "Rajakytkimet:",
        "Enable physical limit switches (Switches installed)" => "Ota fyysiset rajakytkimet käyttöön (kytkimet asennettu)",
        "Homing / Seek Speed:" => "Kotiutusnopeus:",
        "Auto-connect to laser port on startup" => "Yhdistä automaattisesti käynnistettäessä",
        "Save & Close" => "Tallenna ja sulje",
        "Export Profile..." => "Vie profiili...",
        "Import Profile..." => "Tuo profiili...",

        // Material Test Generator
        "Material Test Generator" => "Materiaalitestigeneraattori",
        "Power vs. Speed Test Matrix" => "Teho vs. Nopeus testitaulukko",
        "Min Speed:" => "Min. nopeus:",
        "Max Speed:" => "Maks. nopeus:",
        "Min Power:" => "Min. teho:",
        "Max Power:" => "Maks. teho:",
        "Box Size:" => "Ruudun koko:",
        "Generate Test Grid" => "Luo testitaulukko",

        // Status & Info
        "Ready" => "Valmis",
        "No shape selected." => "Ei muotoa valittuna.",
        "Language" => "Kieli",
        "Machine Settings" => "Koneasetukset",
        "Camera Calibration" => "Kameran kalibrointi",
        "Serial Traffic Log" => "Sarjaliikenneloki",
        "Simulation Stats" => "Simulointitilastot",
        "Text Creation for Laser" => "Tekstin luonti laserille",
        "Create Contour Offset" => "Luo muodon ääriviivasiirto",
        "Serial Port & Controller Connection" => "Sarjaportti- ja ohjainyhteys",
        "Laser Head Positioning & Jog" => "Laserpään asemointi ja siirto",
        "Weak Beam / Alignment Pointer" => "Kohdistussäde / Osoitin",
        "Framing Test with Weak Beam" => "Rajauksen testaus säteellä",
        "Custom Soft Limits (Corners)" => "Mukautetut ohjelmalliset rajat",

        // Fallback: return original key
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_translation() {
        assert_eq!(t("File", Language::English), "File");
        assert_eq!(t("File", Language::Finnish), "Tiedosto");
        assert_eq!(t("Edit", Language::Finnish), "Muokkaa");
        assert_eq!(t("Start", Language::Finnish), "Aloita");
        assert_eq!(t("Stop", Language::Finnish), "Pysäytä");
        assert_eq!(t("Transform & Size", Language::Finnish), "Muunnos & Koko");
        assert_eq!(t("Unknown String XYZ", Language::Finnish), "Unknown String XYZ");
    }

    #[test]
    fn test_language_defaults_and_labels() {
        let lang = Language::default();
        assert_eq!(lang, Language::Finnish);
        assert_eq!(lang.code(), "fi");
        assert_eq!(lang.label(), "Suomi (FI)");
    }
}
