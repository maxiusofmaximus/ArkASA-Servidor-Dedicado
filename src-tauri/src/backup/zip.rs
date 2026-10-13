// ============================================================
// backup/zip.rs — File collection, ZIP creation and extraction
// ============================================================

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ─── File collection ────────────────────────────────────────

pub fn collect_files(server_dir: &str, map: &str, scope: &str) -> Vec<(PathBuf, String)> {
    let base = PathBuf::from(server_dir);
    let saved_arks = base.join("ShooterGame").join("Saved").join("SavedArks");
    let saved = base.join("ShooterGame").join("Saved");

    let mut files: Vec<(PathBuf, String)> = Vec::new();

    match scope {
        "map" => {
            // Only the world .ark file — path relative to server_dir for consistent extraction
            for ext in &["ark", "ark.bak"] {
                let p = saved_arks.join(format!("{}.{}", map, ext));
                if p.exists() {
                    files.push((p.clone(), format!("ShooterGame/Saved/SavedArks/{}.{}", map, ext)));
                }
            }
        }
        "full" => {
            // Entire Saved/ directory
            if saved.exists() {
                for entry in WalkDir::new(&saved).into_iter().flatten() {
                    if entry.file_type().is_file() {
                        let rel = entry.path()
                            .strip_prefix(&base)
                            .unwrap_or(entry.path())
                            .to_string_lossy()
                            .replace('\\', "/");
                        files.push((entry.path().to_path_buf(), rel));
                    }
                }
            }
        }
        _ => {
            // "map_players_tribes" — everything in SavedArks/
            if saved_arks.exists() {
                for entry in WalkDir::new(&saved_arks).into_iter().flatten() {
                    if entry.file_type().is_file() {
                        let rel = entry.path()
                            .strip_prefix(&base)
                            .unwrap_or(entry.path())
                            .to_string_lossy()
                            .replace('\\', "/");
                        files.push((entry.path().to_path_buf(), rel));
                    }
                }
            }
        }
    }

    files
}

// ─── ZIP creation ───────────────────────────────────────────

pub fn create_zip(files: &[(PathBuf, String)], zip_path: &Path, metadata_json: Option<&str>, config_toml: Option<&str>) -> Result<(), String> {
    let file = std::fs::File::create(zip_path)
        .map_err(|e| format!("Cannot create zip: {}", e))?;

    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    // Write config.toml first so the backup is self-contained and importable
    if let Some(toml) = config_toml {
        zip.start_file("config.toml", options)
            .map_err(|e| format!("Zip config.toml error: {}", e))?;
        zip.write_all(toml.as_bytes())
            .map_err(|e| format!("Zip config.toml write error: {}", e))?;
    }

    // Write metadata so it's easy to inspect
    if let Some(meta) = metadata_json {
        zip.start_file("ark-metadata.json", options)
            .map_err(|e| format!("Zip metadata error: {}", e))?;
        zip.write_all(meta.as_bytes())
            .map_err(|e| format!("Zip metadata write error: {}", e))?;
    }

    let mut buf = Vec::new();
    for (abs_path, rel_path) in files {
        match std::fs::File::open(abs_path) {
            Ok(mut f) => {
                buf.clear();
                if let Err(e) = f.read_to_end(&mut buf) {
                    eprintln!("Skipping {}: {}", rel_path, e);
                    continue;
                }
                zip.start_file(rel_path, options)
                    .map_err(|e| format!("Zip start_file error: {}", e))?;
                zip.write_all(&buf)
                    .map_err(|e| format!("Zip write error: {}", e))?;
            }
            Err(e) => eprintln!("Skipping {}: {}", rel_path, e),
        }
    }

    zip.finish().map_err(|e| format!("Zip finish error: {}", e))?;
    Ok(())
}

// ─── ZIP extraction ───────────────────────────────────────────

/// Extract zip into dest_dir. Zip entries are relative to dest_dir.
/// Returns number of files extracted.
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<usize, String> {
    use std::io::Read as _;

    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("Cannot open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid zip: {}", e))?;

    let mut count = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Zip entry error: {}", e))?;

        // Sanitize path — prevent directory traversal
        let entry_name = entry.name().replace('\\', "/");
        if entry_name.contains("..") { continue; }
        if entry.is_dir() { continue; }

        let dest = dest_dir.join(&entry_name);

        // Create parent directories
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("mkdir error for {}: {}", parent.display(), e))?;
        }

        let mut out = std::fs::File::create(&dest)
            .map_err(|e| format!("Cannot create {}: {}", dest.display(), e))?;

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)
            .map_err(|e| format!("Zip read error: {}", e))?;
        out.write_all(&buf)
            .map_err(|e| format!("Write error: {}", e))?;

        count += 1;
    }

    Ok(count)
}
