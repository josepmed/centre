use crate::domain::GlobalMode;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// App metadata stored in meta.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub global_mode: GlobalMode,
    #[serde(default)]
    pub paused_by_mode_task_ids: Vec<String>, // UUIDs as strings

    // Mode time tracking (in seconds for JSON serialization)
    #[serde(default)]
    pub mode_time_working_secs: i64,
    #[serde(default)]
    pub mode_time_lunch_secs: i64,
    #[serde(default)]
    pub mode_time_gym_secs: i64,
    #[serde(default)]
    pub mode_time_dinner_secs: i64,
    #[serde(default)]
    pub mode_time_personal_secs: i64,
    #[serde(default)]
    pub mode_time_sleep_secs: i64,

    #[serde(default)]
    pub last_mode_change_timestamp: Option<String>, // ISO8601 timestamp
}

impl Default for AppMetadata {
    fn default() -> Self {
        Self {
            global_mode: GlobalMode::Working,
            paused_by_mode_task_ids: Vec::new(),
            mode_time_working_secs: 0,
            mode_time_lunch_secs: 0,
            mode_time_gym_secs: 0,
            mode_time_dinner_secs: 0,
            mode_time_personal_secs: 0,
            mode_time_sleep_secs: 0,
            last_mode_change_timestamp: None,
        }
    }
}

/// Load app metadata from meta.json file
pub fn load_metadata<P: AsRef<Path>>(path: P) -> Result<AppMetadata> {
    let path = path.as_ref();

    if !path.exists() {
        // If file doesn't exist, return default metadata
        return Ok(AppMetadata::default());
    }

    let content = std::fs::read_to_string(path)?;
    let metadata: AppMetadata = serde_json::from_str(&content)?;
    Ok(metadata)
}

/// Save app metadata to meta.json file
pub fn save_metadata<P: AsRef<Path>>(path: P, metadata: &AppMetadata) -> Result<()> {
    let json = serde_json::to_string_pretty(metadata)?;
    crate::persistence::atomic_write(path, &json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_nonexistent_metadata() {
        let temp_dir = tempdir().unwrap();
        let meta_path = temp_dir.path().join("meta.json");

        let metadata = load_metadata(&meta_path).unwrap();
        assert_eq!(metadata.global_mode, GlobalMode::Working);
        assert!(metadata.paused_by_mode_task_ids.is_empty());
    }

    #[test]
    fn test_save_and_load_metadata() {
        let temp_dir = tempdir().unwrap();
        let meta_path = temp_dir.path().join("meta.json");

        let mut metadata = AppMetadata::default();
        metadata.global_mode = GlobalMode::Lunch;
        metadata.paused_by_mode_task_ids = vec!["test-id".to_string()];
        metadata.mode_time_working_secs = 3600; // 1 hour
        metadata.mode_time_lunch_secs = 1800;   // 30 minutes

        save_metadata(&meta_path, &metadata).unwrap();

        let loaded = load_metadata(&meta_path).unwrap();
        assert_eq!(loaded.global_mode, GlobalMode::Lunch);
        assert_eq!(loaded.paused_by_mode_task_ids.len(), 1);
        assert_eq!(loaded.mode_time_working_secs, 3600);
        assert_eq!(loaded.mode_time_lunch_secs, 1800);
    }
}
