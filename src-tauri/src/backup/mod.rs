// ============================================================
// backup/mod.rs — Cloud save backup system for ARK SA
// Providers: S3-compatible, Google Drive, OneDrive, iCloud
// ============================================================
//
// P18 — former god-module `backup.rs` split into focused
// submodules: zip, s3, oauth, gdrive, onedrive, local.
//
// The Tauri commands registered in `lib.rs` as `backup::<fn>`
// keep resolving here thanks to the glob re-exports below,
// which also carry the generated `#[macro_export]` command
// macros (`__cmd__*`, `__tauri_command_name_*`) into this
// module. `lib.rs` is intentionally left untouched.
// ============================================================

mod gdrive;
mod local;
mod oauth;
mod onedrive;
mod s3;
mod zip;

pub use gdrive::*;
pub use local::*;
pub use oauth::*;
pub use onedrive::*;
pub use s3::*;
pub use zip::*;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Public types (serialized to/from frontend) ─────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupEntry {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String, // ISO8601
    pub provider: String,
    pub key: String, // S3 key or Drive file ID or local path
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// Entry returned to the frontend for each available backup.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupListEntry {
    pub key: String,       // S3 key / GDrive file ID / OneDrive item ID / local abs path
    pub name: String,      // display filename
    pub size_bytes: u64,
    pub created_at: String, // ISO8601 or human string
}

// ─── Tauri commands ─────────────────────────────────────────

/// Create a ZIP of ARK saves and upload to the configured provider.
/// Returns the backup filename.
#[tauri::command]
pub async fn backup_saves(
    server_dir: String,
    map: String,
    scope: String, // "map" | "map_players_tribes" | "full"
    provider: String, // "s3" | "gdrive" | "onedrive" | "icloud" | "local_folder"
    // S3
    s3_endpoint: Option<String>,
    s3_bucket: Option<String>,
    s3_access_key: Option<String>,
    s3_secret_key: Option<String>,
    s3_region: Option<String>,
    // Google Drive / OneDrive
    access_token: Option<String>,
    // iCloud
    icloud_path: Option<String>,
    // Local folder
    local_folder_path: Option<String>,
    // Optional JSON blob written as ark-metadata.json inside the zip
    metadata_json: Option<String>,
    // Optional config.toml content — included as config.toml so backups are self-contained
    config_toml: Option<String>,
) -> Result<String, String> {
    // 1. Collect files
    let files = collect_files(&server_dir, &map, &scope);
    if files.is_empty() {
        return Err(format!(
            "No save files found in {}/ShooterGame/Saved/",
            server_dir
        ));
    }

    // 2. Create zip in temp directory (spawn_blocking to not block tokio runtime)
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("ark_backup_{}_{}.zip", map, timestamp);
    let zip_path = std::env::temp_dir().join(&filename);
    let zip_path_clone = zip_path.clone();
    let files_clone = files.clone();
    let meta = metadata_json.clone();
    let cfg = config_toml.clone();
    tokio::task::spawn_blocking(move || {
        create_zip(&files_clone, &zip_path_clone, meta.as_deref(), cfg.as_deref())
    }).await.map_err(|e| format!("zip task panicked: {e}"))??;

    // 3. Upload
    match provider.as_str() {
        "s3" => {
            let endpoint = s3_endpoint.ok_or("Falta s3_endpoint")?;
            let bucket = s3_bucket.ok_or("Falta s3_bucket")?;
            let access_key = s3_access_key.ok_or("Falta s3_access_key")?;
            let secret_key = s3_secret_key.ok_or("Falta s3_secret_key")?;
            let region = s3_region.unwrap_or_else(|| "us-east-1".to_string());
            upload_to_s3(&zip_path, &endpoint, &bucket, &access_key, &secret_key, &region, &filename).await?;
        }
        "gdrive" => {
            let token = access_token.ok_or("Falta access_token de Google Drive")?;
            upload_to_gdrive(&zip_path, &token, &filename).await?;
        }
        "onedrive" => {
            let token = access_token.ok_or("Falta access_token de OneDrive")?;
            upload_to_onedrive(&zip_path, &token, &filename).await?;
        }
        "icloud" => {
            let folder = icloud_path.ok_or("Falta ruta de iCloud")?;
            copy_to_icloud(&zip_path, &folder, &filename)?;
        }
        "local_folder" => {
            let folder = local_folder_path.ok_or("Falta ruta de carpeta local")?;
            let dest_dir = PathBuf::from(&folder);
            std::fs::create_dir_all(&dest_dir)
                .map_err(|e| format!("No se pudo crear el directorio '{}': {}", folder, e))?;
            let dest = dest_dir.join(&filename);
            std::fs::copy(&zip_path, &dest)
                .map_err(|e| format!("No se pudo copiar el backup a '{}': {}", folder, e))?;
        }
        other => return Err(format!("Proveedor desconocido: {}", other)),
    }

    // 4. Clean up temp zip
    let _ = std::fs::remove_file(&zip_path);

    Ok(filename)
}

/// Read ark-metadata.json from inside a local backup zip.
/// Returns None if the zip has no metadata (old backup).
#[tauri::command]
pub fn read_backup_metadata(zip_path: String) -> Result<Option<String>, String> {
    use std::io::Read as _;

    let file = std::fs::File::open(&zip_path)
        .map_err(|e| format!("Cannot open zip: {}", e))?;
    let mut archive = ::zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid zip: {}", e))?;

    let result = match archive.by_name("ark-metadata.json") {
        Ok(mut entry) => {
            let mut buf = String::new();
            entry.read_to_string(&mut buf)
                .map_err(|e| format!("Read error: {}", e))?;
            Some(buf)
        }
        Err(_) => None,
    };
    Ok(result)
}

// ─── List backups ─────────────────────────────────────────────

#[tauri::command]
pub async fn list_cloud_backups(
    provider: String,
    s3_endpoint: Option<String>,
    s3_bucket: Option<String>,
    s3_access_key: Option<String>,
    s3_secret_key: Option<String>,
    s3_region: Option<String>,
    access_token: Option<String>,
    icloud_path: Option<String>,
    local_folder_path: Option<String>,
) -> Result<Vec<BackupListEntry>, String> {
    match provider.as_str() {
        "s3" => {
            let endpoint = s3_endpoint.ok_or("Falta s3_endpoint")?;
            let bucket   = s3_bucket.ok_or("Falta s3_bucket")?;
            let ak       = s3_access_key.ok_or("Falta s3_access_key")?;
            let sk       = s3_secret_key.ok_or("Falta s3_secret_key")?;
            let region   = s3_region.unwrap_or_else(|| "us-east-1".to_string());
            list_s3_backups(&endpoint, &bucket, &ak, &sk, &region).await
        }
        "gdrive" => {
            let token = access_token.ok_or("Falta access_token")?;
            list_gdrive_backups(&token).await
        }
        "onedrive" => {
            let token = access_token.ok_or("Falta access_token")?;
            list_onedrive_backups(&token).await
        }
        "icloud" => {
            let folder = icloud_path.ok_or("Falta icloud_path")?;
            list_icloud_backups(&folder)
        }
        "local_folder" => {
            let folder = local_folder_path.ok_or("Falta local_folder_path")?;
            list_local_folder_backups(&folder)
        }
        _ => Err(format!("Proveedor desconocido: {}", provider)),
    }
}

// ─── Restore from backup ──────────────────────────────────────

/// Download a backup zip and extract it into the server's SavedArks.
/// Renames current SavedArks to SavedArks_preRestore_{timestamp} first (safety).
#[tauri::command]
pub async fn restore_backup_from_cloud(
    server_dir: String,
    backup_key: String,    // S3 key / GDrive ID / OneDrive ID / local path
    backup_name: String,   // display name (for messages)
    provider: String,
    s3_endpoint: Option<String>,
    s3_bucket: Option<String>,
    s3_access_key: Option<String>,
    s3_secret_key: Option<String>,
    s3_region: Option<String>,
    access_token: Option<String>,
    _icloud_path: Option<String>,
) -> Result<String, String> {
    // 1. Download zip to temp
    let zip_bytes = match provider.as_str() {
        "s3" => {
            let endpoint = s3_endpoint.ok_or("Falta s3_endpoint")?;
            let bucket   = s3_bucket.ok_or("Falta s3_bucket")?;
            let ak       = s3_access_key.ok_or("Falta s3_access_key")?;
            let sk       = s3_secret_key.ok_or("Falta s3_secret_key")?;
            let region   = s3_region.unwrap_or_else(|| "us-east-1".to_string());
            download_from_s3(&endpoint, &bucket, &ak, &sk, &region, &backup_key).await?
        }
        "gdrive" => {
            let token = access_token.ok_or("Falta access_token")?;
            download_from_gdrive(&backup_key, &token).await?
        }
        "onedrive" => {
            let token = access_token.ok_or("Falta access_token")?;
            download_from_onedrive(&backup_key, &token).await?
        }
        "icloud" | "local_folder" => {
            tokio::fs::read(&backup_key)
                .await
                .map_err(|e| format!("Error leyendo archivo local: {}", e))?
        }
        _ => return Err(format!("Proveedor desconocido: {}", provider)),
    };

    // 2. Write zip to temp file
    let zip_path = std::env::temp_dir().join(format!("ark_restore_{}.zip", Utc::now().timestamp()));
    tokio::fs::write(&zip_path, &zip_bytes)
        .await
        .map_err(|e| format!("Cannot write temp zip: {}", e))?;

    // 3. Safety snapshot: rename current SavedArks → SavedArks_preRestore_{ts}
    let saved_arks = PathBuf::from(&server_dir)
        .join("ShooterGame").join("Saved").join("SavedArks");
    if saved_arks.exists() {
        let snapshot = saved_arks.with_file_name(
            format!("SavedArks_preRestore_{}", Utc::now().format("%Y%m%d_%H%M%S"))
        );
        std::fs::rename(&saved_arks, &snapshot)
            .map_err(|e| format!("Safety snapshot failed: {}", e))?;
    }

    // 4. Extract zip relative to server_dir
    let extracted = {
        let zp = zip_path.clone();
        let sd = PathBuf::from(&server_dir);
        tokio::task::spawn_blocking(move || extract_zip(&zp, &sd))
            .await.map_err(|e| format!("extract task panicked: {e}"))??
    };

    // 5. Cleanup
    let _ = std::fs::remove_file(&zip_path);

    Ok(format!(
        "Restaurado '{}' — {} archivos extraídos. Snapshot previo guardado como SavedArks_preRestore_*",
        backup_name, extracted
    ))
}
