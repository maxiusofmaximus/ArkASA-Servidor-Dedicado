// ============================================================
// backup/gdrive.rs — Google Drive upload / list / download
// ============================================================

use super::BackupListEntry;
use std::path::Path;

// ─── Google Drive upload ─────────────────────────────────────

pub async fn upload_to_gdrive(zip_path: &Path, access_token: &str, filename: &str) -> Result<(), String> {
    let body = tokio::fs::read(zip_path)
        .await
        .map_err(|e| format!("Cannot read zip: {}", e))?;

    // Step 1: get or create "ArkASA Backups" folder
    let folder_id = gdrive_get_or_create_folder(access_token, "ArkASA Backups").await?;

    // Step 2: initiate resumable upload session
    let meta = serde_json::json!({
        "name": filename,
        "parents": [folder_id],
    });

    let client = crate::HTTP_CLIENT.clone();
    let init_resp = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable")
        .bearer_auth(access_token)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("X-Upload-Content-Type", "application/zip")
        .header("X-Upload-Content-Length", body.len().to_string())
        .json(&meta)
        .send()
        .await
        .map_err(|e| format!("GDrive init error: {}", e))?;

    if !init_resp.status().is_success() {
        let status = init_resp.status().as_u16();
        let text = init_resp.text().await.unwrap_or_default();
        return Err(format!("GDrive session init failed {}: {}", status, &text[..text.len().min(300)]));
    }

    let upload_url = init_resp
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or("GDrive: no Location header in resumable upload response")?;

    // Step 3: upload content
    let resp = client
        .put(&upload_url)
        .header("Content-Type", "application/zip")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("GDrive upload error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("GDrive upload failed {}: {}", status, &text[..text.len().min(300)]));
    }

    Ok(())
}

async fn gdrive_get_or_create_folder(token: &str, name: &str) -> Result<String, String> {
    let client = crate::HTTP_CLIENT.clone();
    let q = format!("name='{}' and mimeType='application/vnd.google-apps.folder' and trashed=false", name);
    let resp = client
        .get("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(token)
        .query(&[("q", q.as_str()), ("fields", "files(id)")])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if let Some(id) = json["files"][0]["id"].as_str() {
        return Ok(id.to_string());
    }

    // Create folder
    let create_resp = client
        .post("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(token)
        .json(&serde_json::json!({
            "name": name,
            "mimeType": "application/vnd.google-apps.folder"
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = create_resp.json().await.map_err(|e| e.to_string())?;
    json["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Cannot create GDrive folder".to_string())
}

// ─── List helpers ────────────────────────────────────────────

pub async fn list_gdrive_backups(token: &str) -> Result<Vec<BackupListEntry>, String> {
    let folder_id = gdrive_get_or_create_folder(token, "ArkASA Backups").await?;
    let client = crate::HTTP_CLIENT.clone();
    let q = format!("'{}' in parents and name contains 'ark_backup' and trashed=false", folder_id);
    let resp = client
        .get("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(token)
        .query(&[
            ("q", q.as_str()),
            ("fields", "files(id,name,size,createdTime)"),
            ("orderBy", "createdTime desc"),
        ])
        .send()
        .await
        .map_err(|e| format!("GDrive list error: {}", e))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let files = json["files"].as_array().cloned().unwrap_or_default();
    Ok(files.iter().map(|f| BackupListEntry {
        key: f["id"].as_str().unwrap_or("").to_string(),
        name: f["name"].as_str().unwrap_or("").to_string(),
        size_bytes: f["size"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0),
        created_at: f["createdTime"].as_str().unwrap_or("").to_string(),
    }).collect())
}

// ─── Download helpers ────────────────────────────────────────

pub async fn download_from_gdrive(file_id: &str, token: &str) -> Result<Vec<u8>, String> {
    let client = crate::HTTP_CLIENT.clone();
    let resp = client
        .get(format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file_id))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("GDrive download error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GDrive download failed {}: {}", status, &body[..body.len().min(200)]));
    }
    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string())
}
