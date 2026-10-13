// ============================================================
// backup/s3.rs — S3-compatible upload / list / download + SigV4
// ============================================================

use super::BackupListEntry;
use chrono::Utc;
use std::path::Path;

// ─── Tauri command ──────────────────────────────────────────

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

// ─── S3 upload ──────────────────────────────────────────────

pub async fn upload_to_s3(
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

// ─── List helpers ────────────────────────────────────────────

pub async fn list_s3_backups(
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

// ─── Download helpers ────────────────────────────────────────

pub async fn download_from_s3(
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

pub fn url_encode(s: &str) -> String {
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
