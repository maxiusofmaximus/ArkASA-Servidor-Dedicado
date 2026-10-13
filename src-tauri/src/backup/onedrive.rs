// ============================================================
// backup/onedrive.rs — OneDrive upload / list / download
// ============================================================

use super::BackupListEntry;
use std::path::Path;

// ─── OneDrive upload ─────────────────────────────────────────

pub async fn upload_to_onedrive(zip_path: &Path, access_token: &str, filename: &str) -> Result<(), String> {
    let body = tokio::fs::read(zip_path)
        .await
        .map_err(|e| format!("Cannot read zip: {}", e))?;

    let client = crate::HTTP_CLIENT.clone();

    // Create upload session
    let session_url = format!(
        "https://graph.microsoft.com/v1.0/me/drive/root:/ArkASA Backups/{}:/createUploadSession",
        filename
    );
    let session_resp = client
        .post(&session_url)
        .bearer_auth(access_token)
        .json(&serde_json::json!({
            "item": { "@microsoft.graph.conflictBehavior": "rename" }
        }))
        .send()
        .await
        .map_err(|e| format!("OneDrive session error: {}", e))?;

    if !session_resp.status().is_success() {
        let status = session_resp.status().as_u16();
        let text = session_resp.text().await.unwrap_or_default();
        return Err(format!("OneDrive session failed {}: {}", status, &text[..text.len().min(300)]));
    }

    let session_json: serde_json::Value = session_resp.json().await.map_err(|e| e.to_string())?;
    let upload_url = session_json["uploadUrl"]
        .as_str()
        .ok_or("OneDrive: no uploadUrl in session response")?
        .to_string();

    // Upload in 5MB chunks (OneDrive requires multiples of 320KB, 5MB works)
    let chunk_size = 5 * 1024 * 1024usize;
    let total = body.len();
    let mut offset = 0usize;

    while offset < total {
        let end = (offset + chunk_size).min(total);
        let chunk = &body[offset..end];

        let resp = client
            .put(&upload_url)
            .header("Content-Length", chunk.len().to_string())
            .header("Content-Range", format!("bytes {}-{}/{}", offset, end - 1, total))
            .header("Content-Type", "application/zip")
            .body(chunk.to_vec())
            .send()
            .await
            .map_err(|e| format!("OneDrive chunk upload error: {}", e))?;

        let status = resp.status().as_u16();
        // 200/201 = complete, 202 = in progress
        if status != 200 && status != 201 && status != 202 {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("OneDrive chunk failed {}: {}", status, &text[..text.len().min(300)]));
        }

        offset = end;
    }

    Ok(())
}

// ─── List helpers ────────────────────────────────────────────

pub async fn list_onedrive_backups(token: &str) -> Result<Vec<BackupListEntry>, String> {
    let client = crate::HTTP_CLIENT.clone();
    let resp = client
        .get("https://graph.microsoft.com/v1.0/me/drive/root:/ArkASA Backups:/children")
        .bearer_auth(token)
        .query(&[
            ("$select", "id,name,size,createdDateTime"),
            ("$orderby", "createdDateTime desc"),
        ])
        .send()
        .await
        .map_err(|e| format!("OneDrive list error: {}", e))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let items = json["value"].as_array().cloned().unwrap_or_default();
    Ok(items.iter()
        .filter(|i| i["name"].as_str().map_or(false, |n| n.ends_with(".zip")))
        .map(|i| BackupListEntry {
            key: i["id"].as_str().unwrap_or("").to_string(),
            name: i["name"].as_str().unwrap_or("").to_string(),
            size_bytes: i["size"].as_u64().unwrap_or(0),
            created_at: i["createdDateTime"].as_str().unwrap_or("").to_string(),
        }).collect())
}

// ─── Download helpers ────────────────────────────────────────

pub async fn download_from_onedrive(item_id: &str, token: &str) -> Result<Vec<u8>, String> {
    let client = crate::HTTP_CLIENT.clone();
    let resp = client
        .get(format!("https://graph.microsoft.com/v1.0/me/drive/items/{}/content", item_id))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("OneDrive download error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("OneDrive download failed {}: {}", status, &body[..body.len().min(200)]));
    }
    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string())
}
