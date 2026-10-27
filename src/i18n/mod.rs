//! Internationalization (i18n) localization module for PhotonCAM.
//! Provides multilingual support for Finnish, English, German, French, Italian, and Spanish.

mod de;
mod es;
mod fi;
mod fr;
mod it;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Finnish,
    German,
    French,
    Italian,
    Spanish,
}

impl Language {
    pub const ALL: &'static [Language] = &[
        Language::Finnish,
        Language::English,
        Language::German,
        Language::French,
        Language::Italian,
        Language::Spanish,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Language::Finnish => "Suomi (FI)",
            Language::English => "English (EN)",
            Language::German => "Deutsch (DE)",
            Language::French => "Français (FR)",
            Language::Italian => "Italiano (IT)",
            Language::Spanish => "Español (ES)",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Language::Finnish => "🌐 Suomi",
            Language::English => "🌐 English",
            Language::German => "🌐 Deutsch",
            Language::French => "🌐 Français",
            Language::Italian => "🌐 Italiano",
            Language::Spanish => "🌐 Español",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::Finnish => "fi",
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
            Language::Italian => "it",
            Language::Spanish => "es",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::Finnish
    }
}

/// Translates a given key into the requested language
pub fn t<'a>(key: &'a str, lang: Language) -> &'a str {
    match lang {
        Language::English => key,
        Language::Finnish => fi::translate(key),
        Language::German => de::translate(key),
        Language::French => fr::translate(key),
        Language::Italian => it::translate(key),
        Language::Spanish => es::translate(key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translations_coverage() {
        let keys = ["File", "Edit", "Laser", "Settings", "Undo", "Redo", "Disconnected"];
        for key in keys {
            // Finnish
            assert_ne!(t(key, Language::Finnish), "", "Key: {}", key);
            // English (identity)
            assert_eq!(t(key, Language::English), key);
            // German
            assert_ne!(t(key, Language::German), "", "Key: {}", key);
            // French
            assert_ne!(t(key, Language::French), "", "Key: {}", key);
            // Italian
            assert_ne!(t(key, Language::Italian), "", "Key: {}", key);
            // Spanish
            assert_ne!(t(key, Language::Spanish), "", "Key: {}", key);
        }
    }

    #[test]
    fn test_specific_translations() {
        assert_eq!(t("File", Language::Finnish), "Tiedosto");
        assert_eq!(t("File", Language::German), "Datei");
        assert_eq!(t("File", Language::French), "Fichier");
        assert_eq!(t("File", Language::Italian), "File");
        assert_eq!(t("File", Language::Spanish), "Archivo");

        assert_eq!(t("Undo", Language::Finnish), "Kumoa");
        assert_eq!(t("Undo", Language::German), "Rückgängig");
        assert_eq!(t("Undo", Language::French), "Annuler");
        assert_eq!(t("Undo", Language::Italian), "Annulla");
        assert_eq!(t("Undo", Language::Spanish), "Deshacer");

        // Connection window
        assert_eq!(t("Controller Type:", Language::Finnish), "Ohjaimen tyyppi:");
        assert_eq!(t("Controller Type:", Language::German), "Controller-Typ:");
        assert_eq!(t("Controller Type:", Language::French), "Type de contrôleur :");
        assert_eq!(t("Controller Type:", Language::Italian), "Tipo di controller:");
        assert_eq!(t("Controller Type:", Language::Spanish), "Tipo de controlador:");

        assert_eq!(t("Scan Ports", Language::Finnish), "Etsi portteja");
        assert_eq!(t("Scan Ports", Language::German), "Ports suchen");
        assert_eq!(t("Scan Ports", Language::French), "Scanner les ports");
        assert_eq!(t("Scan Ports", Language::Italian), "Cerca porte");
        assert_eq!(t("Scan Ports", Language::Spanish), "Escanear puertos");

        // Align & Distribute, Preview, Tabs, Auto-Nest
        assert_eq!(t("Align & Distribute", Language::Finnish), "Kohdista ja tasaa");
        assert_eq!(t("Align & Distribute", Language::German), "Ausrichten & Verteilen");
        assert_eq!(t("Center to Bed (P)", Language::Finnish), "Keskitä alustalle (P)");
        assert_eq!(t("Laser Cut Preview & Simulation", Language::French), "Aperçu et simulation de découpe laser");
        assert_eq!(t("Tabs / Bridges:", Language::Spanish), "Puentes de sujeción (Tabs):");
        assert_eq!(t("Auto-Nest (Material Packing)", Language::Finnish), "Automaattinen pesitys (Nesting)");
        assert_eq!(t("Auto-Nest", Language::German), "Verschachteln");
        assert_eq!(t("Bed overlay updated from camera.", Language::Finnish), "Alustan taustakuva päivitetty kamerasta.");
        assert_eq!(t("Bed overlay updated from camera.", Language::German), "Tisch-Hintergrundbild von Kamera aktualisiert.");
    }

    #[test]
    fn test_unknown_key_fallback() {
        let unknown = "Unknown Custom String 12345";
        for &lang in Language::ALL {
            assert_eq!(t(unknown, lang), unknown);
        }
    }
}
