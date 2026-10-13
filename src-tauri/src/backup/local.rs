// ============================================================
// backup/local.rs — iCloud / local folder + server log reading
// ============================================================

use super::BackupListEntry;
use chrono::Utc;
use std::io::Read;
use std::path::{Path, PathBuf};

// ─── Server log ─────────────────────────────────────────────

/// Read the last N lines of ShooterGame.log.
/// If `map` is provided, tries `{server_dir}/{map}/ShooterGame/Saved/Logs/ShooterGame.log` first
/// (for per-map server dirs in cluster setups), then falls back to the shared log.
#[tauri::command]
pub async fn read_server_log(server_dir: String, map: Option<String>, lines: usize) -> Result<Vec<String>, String> {
    let base = PathBuf::from(&server_dir);
    let shared_log = base.join("ShooterGame").join("Saved").join("Logs").join("ShooterGame.log");

    let log_path = if let Some(ref m) = map {
        let per_map = base.join(m).join("ShooterGame").join("Saved").join("Logs").join("ShooterGame.log");
        if per_map.exists() { per_map } else { shared_log }
    } else {
        shared_log
    };

    if !log_path.exists() {
        return Err(format!("Log no encontrado: {}", log_path.display()));
    }

    tokio::task::spawn_blocking(move || {
        tail_file_lines(&log_path, lines)
    }).await.map_err(|e| format!("log read task panicked: {e}"))?
}

fn tail_file_lines(path: &Path, max_lines: usize) -> Result<Vec<String>, String> {
    use std::io::{BufRead, Seek, SeekFrom};
    let file = std::fs::File::open(path).map_err(|e| format!("Error leyendo log: {}", e))?;
    let file_len = file.metadata().map_err(|e| format!("Error leyendo log: {}", e))?.len();

    if file_len == 0 {
        return Ok(Vec::new());
    }

    let mut reader = std::io::BufReader::new(file);

    let chunk_size: u64 = 8192;
    let mut pos = file_len;
    let mut newline_count = 0u64;
    let mut read_from = file_len;

    while pos > 0 && newline_count < (max_lines as u64 + 1) {
        let seek_back = chunk_size.min(pos);
        pos -= seek_back;
        reader.get_mut().seek(SeekFrom::Start(pos)).map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; seek_back as usize];
        reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
        for b in buf.iter().rev() {
            if *b == b'\n' {
                newline_count += 1;
                if newline_count > max_lines as u64 {
                    break;
                }
            }
        }
        read_from = pos;
    }

    reader.get_mut().seek(SeekFrom::Start(read_from)).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for line in reader.lines() {
        if let Ok(l) = line {
            if result.len() >= max_lines { break; }
            result.push(l);
        }
    }
    Ok(result)
}

// ─── iCloud local copy ───────────────────────────────────────

pub fn copy_to_icloud(zip_path: &Path, icloud_folder: &str, filename: &str) -> Result<(), String> {
    let dest_dir = PathBuf::from(icloud_folder).join("ArkASA Backups");
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Cannot create iCloud folder: {}", e))?;

    let dest = dest_dir.join(filename);
    std::fs::copy(zip_path, &dest)
        .map_err(|e| format!("iCloud copy failed: {}", e))?;

    Ok(())
}

// ─── List helpers ────────────────────────────────────────────

pub fn list_icloud_backups(folder: &str) -> Result<Vec<BackupListEntry>, String> {
    let dir = PathBuf::from(folder).join("ArkASA Backups");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut entries: Vec<BackupListEntry> = std::fs::read_dir(&dir)
        .map_err(|e| format!("iCloud dir error: {}", e))?
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let name = path.file_name()?.to_string_lossy().to_string();
            if !name.ends_with(".zip") { return None; }
            let meta = std::fs::metadata(&path).ok()?;
            let modified = meta.modified().ok()?;
            let dt: chrono::DateTime<Utc> = modified.into();
            Some(BackupListEntry {
                key: path.to_string_lossy().to_string(),
                name,
                size_bytes: meta.len(),
                created_at: dt.to_rfc3339(),
            })
        })
        .collect();
    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(entries)
}

pub fn list_local_folder_backups(folder: &str) -> Result<Vec<BackupListEntry>, String> {
    let dir = PathBuf::from(folder);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut entries: Vec<BackupListEntry> = std::fs::read_dir(&dir)
        .map_err(|e| format!("Local folder dir error: {}", e))?
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let name = path.file_name()?.to_string_lossy().to_string();
            if !name.starts_with("ark_backup_") || !name.ends_with(".zip") { return None; }
            let meta = std::fs::metadata(&path).ok()?;
            let modified = meta.modified().ok()?;
            let dt: chrono::DateTime<Utc> = modified.into();
            Some(BackupListEntry {
                key: path.to_string_lossy().to_string(),
                name,
                size_bytes: meta.len(),
                created_at: dt.to_rfc3339(),
            })
        })
        .collect();
    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(entries)
}
