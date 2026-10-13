// ============================================================
// backup/oauth.rs — OAuth PKCE flows (Google Drive, OneDrive)
// ============================================================

use super::s3::url_encode;
use super::OAuthTokens;

// ─── Tauri commands ─────────────────────────────────────────

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
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("Cannot open browser: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Cannot open browser: {}", e))?;
    }
    let _ = app;
    Ok(())
}
