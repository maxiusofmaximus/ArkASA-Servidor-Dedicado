// ============================================================
// backup.rs — Cloud save backup system for ARK SA
// Providers: S3-compatible, Google Drive, OneDrive, iCloud
// ============================================================

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    let mut archive = zip::ZipArchive::new(file)
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

/// Start Google Drive OAuth PKCE flow. Opens browser, waits for callback.
/// Returns {access_token, refresh_token}.
#[tauri::command]
pub async fn start_gdrive_oauth(
    app: tauri::AppHandle,
    client_id: String,
    client_secret: String,
) -> Result<OAuthTokens, String> {
    let port = find_free_port().await?;
    let redirect_uri = format!("http://127.0.0.1:{}", port);
    let verifier = generate_pkce_verifier();
    let challenge = pkce_challenge(&verifier);

    let scope = url_encode("https://www.googleapis.com/auth/drive.file");
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth\
        ?client_id={}&redirect_uri={}&response_type=code\
        &scope={}&code_challenge={}&code_challenge_method=S256\
        &access_type=offline&prompt=consent",
        url_encode(&client_id),
        url_encode(&redirect_uri),
        scope,
        challenge
    );

    open_browser(&app, &auth_url)?;

    let code = wait_for_oauth_callback(port).await?;

    // Exchange code for tokens
    let client = crate::HTTP_CLIENT.clone();
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", verifier.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("Token exchange error: {}", e))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Token parse error: {}", e))?;

    if let Some(err) = json.get("error") {
        return Err(format!("OAuth error: {}", err));
    }

    Ok(OAuthTokens {
        access_token: json["access_token"].as_str().unwrap_or("").to_string(),
        refresh_token: json["refresh_token"].as_str().unwrap_or("").to_string(),
    })
}

/// Start OneDrive OAuth PKCE flow (no client_secret needed for public clients).
#[tauri::command]
pub async fn start_onedrive_oauth(
    app: tauri::AppHandle,
    client_id: String,
) -> Result<OAuthTokens, String> {
    let port = find_free_port().await?;
    let redirect_uri = format!("http://localhost:{}", port);
    let verifier = generate_pkce_verifier();
    let challenge = pkce_challenge(&verifier);

    let scope = url_encode("Files.ReadWrite offline_access");
    let auth_url = format!(
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize\
        ?client_id={}&redirect_uri={}&response_type=code\
        &scope={}&code_challenge={}&code_challenge_method=S256",
        url_encode(&client_id),
        url_encode(&redirect_uri),
        scope,
        challenge
    );

    open_browser(&app, &auth_url)?;

    let code = wait_for_oauth_callback(port).await?;

    let client = crate::HTTP_CLIENT.clone();
    let resp = client
        .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .form(&[
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", verifier.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("Token exchange error: {}", e))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Token parse error: {}", e))?;

    if let Some(err) = json.get("error") {
        return Err(format!("OAuth error: {} — {}", err, json.get("error_description").unwrap_or(&serde_json::Value::Null)));
    }

    Ok(OAuthTokens {
        access_token: json["access_token"].as_str().unwrap_or("").to_string(),
        refresh_token: json["refresh_token"].as_str().unwrap_or("").to_string(),
    })
}

/// Refresh Google Drive access token using refresh_token.
#[tauri::command]
pub async fn refresh_gdrive_token(
    client_id: String,
    client_secret: String,
    refresh_token: String,
) -> Result<String, String> {
    let client = crate::HTTP_CLIENT.clone();
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    json["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("No access_token in response: {:?}", json))
}

/// Refresh OneDrive access token.
#[tauri::command]
pub async fn refresh_onedrive_token(
    client_id: String,
    refresh_token: String,
) -> Result<String, String> {
    let client = crate::HTTP_CLIENT.clone();
    let resp = client
        .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("refresh_token", refresh_token.as_str()),
            ("grant_type", "refresh_token"),
            ("scope", "Files.ReadWrite offline_access"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    json["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("No access_token: {:?}", json))
}

/// Test S3 connection by listing objects with a prefix.
#[tauri::command]
pub async fn test_s3_connection(
    endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
    region: String,
) -> Result<String, String> {
    // List objects — if it doesn't error, connection works
    let url = build_s3_url(&endpoint, &bucket, "");
    let now = Utc::now();
    let payload_hash = sha256_hex(b"");
    let headers = sign_s3_request("GET", &url, &payload_hash, &access_key, &secret_key, &region, &now)?;

    let client = crate::HTTP_CLIENT.clone();
    let mut req = client.get(&format!("{}?max-keys=1", url));
    for (k, v) in &headers {
        req = req.header(k.as_str(), v.as_str());
    }

    let resp = req.send().await.map_err(|e| format!("Connection error: {}", e))?;
    let status = resp.status();
    if status.is_success() || status.as_u16() == 403 {
        // 403 means we reached the server (credentials may be wrong but connection works)
        Ok(format!("Conectado — HTTP {}", status.as_u16()))
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(format!("S3 error {}: {}", status.as_u16(), &body[..body.len().min(200)]))
    }
}

// ─── File collection ────────────────────────────────────────

fn collect_files(server_dir: &str, map: &str, scope: &str) -> Vec<(PathBuf, String)> {
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

fn create_zip(files: &[(PathBuf, String)], zip_path: &Path, metadata_json: Option<&str>, config_toml: Option<&str>) -> Result<(), String> {
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

// ─── S3 upload ──────────────────────────────────────────────

async fn upload_to_s3(
    zip_path: &Path,
    endpoint: &str,
    bucket: &str,
    access_key: &str,
    secret_key: &str,
    region: &str,
    key: &str,
) -> Result<(), String> {
    let body = tokio::fs::read(zip_path)
        .await
        .map_err(|e| format!("Cannot read zip: {}", e))?;

    let url = build_s3_url(endpoint, bucket, key);
    let now = Utc::now();
    let payload_hash = sha256_hex(&body);
    let mut headers = sign_s3_request("PUT", &url, &payload_hash, access_key, secret_key, region, &now)?;
    headers.insert("content-type".to_string(), "application/zip".to_string());
    headers.insert("content-length".to_string(), body.len().to_string());

    let client = crate::HTTP_CLIENT.clone();
    let mut req = client.put(&url).body(body);
    for (k, v) in &headers {
        req = req.header(k.as_str(), v.as_str());
    }

    let resp = req.send().await.map_err(|e| format!("S3 upload error: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("S3 upload failed {}: {}", status, &body[..body.len().min(400)]));
    }
    Ok(())
}

fn build_s3_url(endpoint: &str, bucket: &str, key: &str) -> String {
    let ep = endpoint.trim_end_matches('/');
    // Detect if endpoint already contains the bucket (path-style vs virtual-hosted)
    if ep.contains(bucket) {
        format!("{}/{}", ep, key)
    } else {
        format!("{}/{}/{}", ep, bucket, key)
    }
}

// ─── AWS Signature V4 ───────────────────────────────────────

fn sign_s3_request(
    method: &str,
    url: &str,
    payload_hash: &str,
    access_key: &str,
    secret_key: &str,
    region: &str,
    now: &chrono::DateTime<Utc>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let date = now.format("%Y%m%d").to_string();
    let datetime = now.format("%Y%m%dT%H%M%SZ").to_string();

    // Parse host + path from URL (e.g. "https://s3.example.com/bucket/key")
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let (host, path_and_query) = if let Some(pos) = without_scheme.find('/') {
        (&without_scheme[..pos], &without_scheme[pos..])
    } else {
        (without_scheme, "/")
    };
    let path = path_and_query.split('?').next().unwrap_or("/");

    let canonical_headers = format!(
        "content-type:application/zip\nhost:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        host, payload_hash, datetime
    );
    let signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date";

    let canonical_request = format!(
        "{}\n{}\n\n{}\n{}\n{}",
        method,
        encode_path(path),
        canonical_headers,
        signed_headers,
        payload_hash
    );

    let credential_scope = format!("{}/{}/s3/aws4_request", date, region);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        datetime,
        credential_scope,
        sha256_hex(canonical_request.as_bytes())
    );

    let k_date = hmac_sha256(format!("AWS4{}", secret_key).as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        access_key, credential_scope, signed_headers, signature
    );

    let mut h = std::collections::HashMap::new();
    h.insert("Authorization".to_string(), authorization);
    h.insert("x-amz-content-sha256".to_string(), payload_hash.to_string());
    h.insert("x-amz-date".to_string(), datetime);
    Ok(h)
}

fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|seg| {
            seg.chars().map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            }).collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/")
}

// ─── Google Drive upload ─────────────────────────────────────

async fn upload_to_gdrive(zip_path: &Path, access_token: &str, filename: &str) -> Result<(), String> {
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

// ─── OneDrive upload ─────────────────────────────────────────

async fn upload_to_onedrive(zip_path: &Path, access_token: &str, filename: &str) -> Result<(), String> {
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

// ─── iCloud local copy ───────────────────────────────────────

fn copy_to_icloud(zip_path: &Path, icloud_folder: &str, filename: &str) -> Result<(), String> {
    let dest_dir = PathBuf::from(icloud_folder).join("ArkASA Backups");
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Cannot create iCloud folder: {}", e))?;

    let dest = dest_dir.join(filename);
    std::fs::copy(zip_path, &dest)
        .map_err(|e| format!("iCloud copy failed: {}", e))?;

    Ok(())
}

// ─── OAuth helpers ───────────────────────────────────────────

fn generate_pkce_verifier() -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use rand::RngCore;
    let mut bytes = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn pkce_challenge(verifier: &str) -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

async fn find_free_port() -> Result<u16, String> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Cannot bind port: {}", e))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    drop(listener);
    Ok(port)
}

async fn wait_for_oauth_callback(port: u16) -> Result<String, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|e| format!("Callback server error: {}", e))?;

    // Wait with 3-minute timeout
    let (mut socket, _) = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        listener.accept(),
    )
    .await
    .map_err(|_| "OAuth timeout — el usuario no autorizó en 3 minutos".to_string())?
    .map_err(|e| format!("Accept error: {}", e))?;

    let mut buf = [0u8; 8192];
    let n = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        socket.read(&mut buf),
    )
    .await
    .map_err(|_| "Read timeout".to_string())?
    .map_err(|e| format!("Read error: {}", e))?;

    let request = String::from_utf8_lossy(&buf[..n]);

    // Send success page
    let html = "<html><body style='font-family:sans-serif;text-align:center;margin-top:80px'>\
        <h2>✅ Autorización completada</h2>\
        <p>Puedes cerrar esta pestaña y volver a la app.</p>\
        </body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        html.len(), html
    );
    let _ = socket.write_all(response.as_bytes()).await;
    drop(socket);

    // Parse ?code= from GET /...?code=XXX
    let first_line = request.lines().next().unwrap_or("");
    // "GET /?code=4/abc&... HTTP/1.1"
    let path = first_line.split_whitespace().nth(1).unwrap_or("");
    parse_query_param(path, "code")
        .ok_or_else(|| format!("No 'code' param in callback: {}", path))
}

fn parse_query_param(url_path: &str, param: &str) -> Option<String> {
    let query = url_path.split('?').nth(1)?;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        let k = kv.next()?;
        let v = kv.next()?;
        if k == param {
            return Some(v.replace('+', " ").to_string());
        }
    }
    None
}

fn open_browser(app: &tauri::AppHandle, url: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Cannot open browser: {}", e))?;
    let _ = app; // Keep app handle parameter for future use
    Ok(())
}

// ─── Crypto helpers ──────────────────────────────────────────

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    hex::encode(Sha256::digest(data))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take any key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

// ─── List backups ─────────────────────────────────────────────

/// Entry returned to the frontend for each available backup.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupListEntry {
    pub key: String,       // S3 key / GDrive file ID / OneDrive item ID / local abs path
    pub name: String,      // display filename
    pub size_bytes: u64,
    pub created_at: String, // ISO8601 or human string
}

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

// ─── List helpers (per provider) ─────────────────────────────

async fn list_s3_backups(
    endpoint: &str,
    bucket: &str,
    access_key: &str,
    secret_key: &str,
    region: &str,
) -> Result<Vec<BackupListEntry>, String> {
    let url = format!("{}/{}?list-type=2&prefix=ark_backup_", endpoint.trim_end_matches('/'), bucket);
    let now = Utc::now();
    let payload_hash = sha256_hex(b"");
    let headers = sign_s3_request("GET", &url, &payload_hash, access_key, secret_key, region, &now)?;

    let client = crate::HTTP_CLIENT.clone();
    let mut req = client.get(&url);
    for (k, v) in &headers {
        req = req.header(k.as_str(), v.as_str());
    }
    // Override content-type for GET (sign_s3_request adds it)
    req = req.header("content-type", "application/zip");

    let resp = req.send().await.map_err(|e| format!("S3 list error: {}", e))?;
    let xml = resp.text().await.map_err(|e| format!("S3 list read error: {}", e))?;
    Ok(parse_s3_list_xml(&xml))
}

async fn list_gdrive_backups(token: &str) -> Result<Vec<BackupListEntry>, String> {
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

async fn list_onedrive_backups(token: &str) -> Result<Vec<BackupListEntry>, String> {
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

fn list_icloud_backups(folder: &str) -> Result<Vec<BackupListEntry>, String> {
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

fn list_local_folder_backups(folder: &str) -> Result<Vec<BackupListEntry>, String> {
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

// ─── Download helpers (per provider) ─────────────────────────

async fn download_from_s3(
    endpoint: &str,
    bucket: &str,
    access_key: &str,
    secret_key: &str,
    region: &str,
    key: &str,
) -> Result<Vec<u8>, String> {
    let url = build_s3_url(endpoint, bucket, key);
    let now = Utc::now();
    let payload_hash = sha256_hex(b"");
    let mut headers = sign_s3_request("GET", &url, &payload_hash, access_key, secret_key, region, &now)?;
    // For GET, we don't need content-type in signed headers — use UNSIGNED or empty
    headers.insert("content-type".to_string(), "application/zip".to_string());

    let client = crate::HTTP_CLIENT.clone();
    let mut req = client.get(&url);
    for (k, v) in &headers {
        req = req.header(k.as_str(), v.as_str());
    }
    let resp = req.send().await.map_err(|e| format!("S3 download error: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("S3 download failed {}: {}", status, &body[..body.len().min(200)]));
    }
    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string())
}

async fn download_from_gdrive(file_id: &str, token: &str) -> Result<Vec<u8>, String> {
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

async fn download_from_onedrive(item_id: &str, token: &str) -> Result<Vec<u8>, String> {
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

// ─── ZIP extraction ───────────────────────────────────────────

/// Extract zip into dest_dir. Zip entries are relative to dest_dir.
/// Returns number of files extracted.
fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<usize, String> {
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

// ─── S3 XML list parser ───────────────────────────────────────

fn parse_s3_list_xml(xml: &str) -> Vec<BackupListEntry> {
    let mut entries = Vec::new();
    for block in xml.split("<Contents>").skip(1) {
        let key = match xml_tag(block, "Key") { Some(k) => k, None => continue };
        if !key.ends_with(".zip") { continue; }
        let name = key.split('/').last().unwrap_or(&key).to_string();
        let size = xml_tag(block, "Size").and_then(|s| s.parse().ok()).unwrap_or(0u64);
        let modified = xml_tag(block, "LastModified").unwrap_or_default();
        entries.push(BackupListEntry { key, name, size_bytes: size, created_at: modified });
    }
    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    entries
}

fn xml_tag<'a>(xml: &'a str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].trim().to_string())
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(c),
            ' ' => out.push_str("%20"),
            _ => {
                let encoded = c.to_string();
                for byte in encoded.as_bytes() {
                    out.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    out
}
