use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Get the centre directory - checks for local .centre first, then falls back to global ~/.centre
pub fn get_centre_dir() -> Result<PathBuf> {
    // Check for local .centre directory
    let current_dir = env::current_dir().context("Could not determine current directory")?;
    let local_centre = find_local_centre(&current_dir);

    if let Some(local_dir) = local_centre {
        return Ok(local_dir);
    }

    // Fall back to global ~/.centre
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".centre"))
}

/// Find local .centre directory by walking up the directory tree
fn find_local_centre(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir;

    loop {
        let centre_dir = current.join(".centre");
        if centre_dir.exists() && centre_dir.is_dir() {
            return Some(centre_dir);
        }

        // Move up to parent directory
        current = current.parent()?;
    }
}

/// Ensure the centre directory exists
pub fn ensure_centre_dir() -> Result<PathBuf> {
    let dir = get_centre_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }
    Ok(dir)
}

/// Initialize a local .centre directory in the current directory
pub fn init_local_centre() -> Result<PathBuf> {
    let current_dir = env::current_dir().context("Could not determine current directory")?;
    let centre_dir = current_dir.join(".centre");

    if centre_dir.exists() {
        anyhow::bail!("Centre directory already exists: {}", centre_dir.display());
    }

    fs::create_dir_all(&centre_dir)
        .with_context(|| format!("Failed to create directory: {}", centre_dir.display()))?;

    Ok(centre_dir)
}

/// Get path to daily file for a specific date (YYYY-MM-DD.md)
pub fn daily_file(date: chrono::NaiveDate) -> Result<PathBuf> {
    let filename = format!("{}.md", date.format("%Y-%m-%d"));
    Ok(ensure_centre_dir()?.join(filename))
}

/// Get path to today's daily file
pub fn today_file() -> Result<PathBuf> {
    let today = chrono::Local::now().date_naive();
    daily_file(today)
}

/// Get path to tomorrow's daily file
pub fn tomorrow_file() -> Result<PathBuf> {
    let tomorrow = chrono::Local::now().date_naive() + chrono::Duration::days(1);
    daily_file(tomorrow)
}

/// Get path to previous day's file (for copying incomplete tasks)
pub fn previous_day_file() -> Result<PathBuf> {
    let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
    daily_file(yesterday)
}

/// Get all daily files in the centre directory (for browsing history)
pub fn list_daily_files() -> Result<Vec<PathBuf>> {
    let dir = ensure_centre_dir()?;
    let mut files = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check if it matches YYYY-MM-DD.md pattern
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.len() == 13 // "YYYY-MM-DD.md"
                && filename.ends_with(".md")
                && filename.chars().nth(4) == Some('-')
                && filename.chars().nth(7) == Some('-')
            {
                files.push(path);
            }
        }
    }

    // Sort by filename (which naturally sorts by date)
    files.sort();
    Ok(files)
}

/// Legacy file paths (for migration)
pub fn done_log_file() -> Result<PathBuf> {
    Ok(ensure_centre_dir()?.join("done.log.md"))
}

pub fn archive_file() -> Result<PathBuf> {
    Ok(ensure_centre_dir()?.join("archive.md"))
}

/// Get path to journal file for today
pub fn journal_file() -> Result<PathBuf> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    Ok(ensure_centre_dir()?.join(format!("journal-{}.md", today)))
}

/// Get path to meta.json file (stores global mode and other app metadata)
pub fn meta_file() -> Result<PathBuf> {
    Ok(ensure_centre_dir()?.join("meta.json"))
}

/// Atomically write content to a file using temp file + rename
pub fn atomic_write<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();
    let dir = path
        .parent()
        .context("File path has no parent directory")?;

    // Create temp file in the same directory
    let mut temp_file = NamedTempFile::new_in(dir)
        .context("Failed to create temporary file")?;

    // Write content
    temp_file
        .write_all(content.as_bytes())
        .context("Failed to write to temporary file")?;

    // Sync to disk
    temp_file
        .as_file()
        .sync_all()
        .context("Failed to sync temporary file")?;

    // Atomically rename temp file to target
    temp_file
        .persist(path)
        .with_context(|| format!("Failed to persist file: {}", path.display()))?;

    Ok(())
}

/// Read file content, return empty string if file doesn't exist
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))
}

/// Create a backup of a file with timestamp
pub fn backup_file<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(path.to_path_buf());
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = path.with_extension(format!("bak.{}.md", timestamp));

    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to backup file: {}", path.display()))?;

    Ok(backup_path)
}

/// Append content to a file (for done.log.md)
pub fn append_to_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open file for appending: {}", path.display()))?;

    file.write_all(content.as_bytes())
        .context("Failed to append to file")?;

    file.sync_all().context("Failed to sync file")?;

    Ok(())
}

/// Truncate a file (for clearing tomorrow.md after promotion)
pub fn truncate_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        fs::write(path, "")
            .with_context(|| format!("Failed to truncate file: {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_get_centre_dir() {
        let dir = get_centre_dir().unwrap();
        assert!(dir.to_string_lossy().contains(".centre"));
    }

    #[test]
    fn test_atomic_write_and_read() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        let content = "Hello, world!";
        atomic_write(&test_file, content).unwrap();

        let read_content = read_file(&test_file).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("nonexistent.txt");

        let content = read_file(&test_file).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_append_to_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        append_to_file(&test_file, "Line 1\n").unwrap();
        append_to_file(&test_file, "Line 2\n").unwrap();

        let content = read_file(&test_file).unwrap();
        assert_eq!(content, "Line 1\nLine 2\n");
    }

    #[test]
    fn test_truncate_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        atomic_write(&test_file, "Some content").unwrap();
        truncate_file(&test_file).unwrap();

        let content = read_file(&test_file).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_backup_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        atomic_write(&test_file, "Original content").unwrap();
        let backup_path = backup_file(&test_file).unwrap();

        assert!(backup_path.exists());
        let backup_content = read_file(&backup_path).unwrap();
        assert_eq!(backup_content, "Original content");
    }
}
