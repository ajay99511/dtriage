use camino::Utf8Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationRule {
    /// Rule name
    pub name: String,

    /// File extensions to match (e.g., ["pdf", "docx"])
    pub extensions: Vec<String>,

    /// Destination folder (relative to Downloads)
    pub destination: String,

    /// Naming pattern (optional)
    pub naming_pattern: Option<String>,

    /// Priority (higher = evaluated first)
    pub priority: u32,
}

impl CategorizationRule {
    /// Check if file matches this rule
    pub fn matches(&self, file_path: &Utf8Path) -> bool {
        if let Some(ext) = file_path.extension() {
            self.extensions
                .iter()
                .any(|e| e.to_lowercase() == ext.to_lowercase())
        } else {
            false
        }
    }

    /// Get default rules
    pub fn defaults() -> Vec<Self> {
        vec![
            CategorizationRule {
                name: "Documents".to_string(),
                extensions: vec![
                    "pdf".into(),
                    "doc".into(),
                    "docx".into(),
                    "txt".into(),
                    "rtf".into(),
                ],
                destination: "Documents".to_string(),
                naming_pattern: None,
                priority: 10,
            },
            CategorizationRule {
                name: "Images".to_string(),
                extensions: vec![
                    "jpg".into(),
                    "jpeg".into(),
                    "png".into(),
                    "gif".into(),
                    "bmp".into(),
                    "svg".into(),
                    "webp".into(),
                ],
                destination: "Pictures".to_string(),
                naming_pattern: None,
                priority: 10,
            },
            CategorizationRule {
                name: "Videos".to_string(),
                extensions: vec![
                    "mp4".into(),
                    "avi".into(),
                    "mkv".into(),
                    "mov".into(),
                    "wmv".into(),
                    "flv".into(),
                ],
                destination: "Videos".to_string(),
                naming_pattern: None,
                priority: 10,
            },
            CategorizationRule {
                name: "Music".to_string(),
                extensions: vec![
                    "mp3".into(),
                    "wav".into(),
                    "flac".into(),
                    "aac".into(),
                    "ogg".into(),
                    "wma".into(),
                ],
                destination: "Music".to_string(),
                naming_pattern: None,
                priority: 10,
            },
            CategorizationRule {
                name: "Archives".to_string(),
                extensions: vec![
                    "zip".into(),
                    "rar".into(),
                    "7z".into(),
                    "tar".into(),
                    "gz".into(),
                    "bz2".into(),
                ],
                destination: "Archives".to_string(),
                naming_pattern: None,
                priority: 10,
            },
            CategorizationRule {
                name: "Installers".to_string(),
                extensions: vec!["exe".into(), "msi".into(), "dmg".into(), "pkg".into()],
                destination: "Installers".to_string(),
                naming_pattern: None,
                priority: 20, // Higher priority
            },
            CategorizationRule {
                name: "Code".to_string(),
                extensions: vec![
                    "js".into(),
                    "ts".into(),
                    "py".into(),
                    "rs".into(),
                    "go".into(),
                    "java".into(),
                    "cpp".into(),
                    "c".into(),
                    "h".into(),
                    "cs".into(),
                    "php".into(),
                    "rb".into(),
                    "sh".into(),
                    "bash".into(),
                ],
                destination: "Code".to_string(),
                naming_pattern: None,
                priority: 10,
            },
        ]
    }
}
