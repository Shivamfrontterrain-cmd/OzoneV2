use std::collections::HashSet;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use sqlx::{SqlitePool, Row};
use serde_json::json;

use crate::{
    format_pokemon_name, PokedexData, RedeemRewardsConfig,
    ALL_MEGA_STONES, TM_CATALOG, TUTOR_CATALOG,
};

const CANONICAL_GYMS_JSON: &str = include_str!("../assets/canonical_gyms.json");

#[derive(Clone)]
pub struct AdminState {
    pub pool: Arc<SqlitePool>,
    pub pokedex: Arc<PokedexData>,
    pub admin_key: String,
    pub sessions: Arc<RwLock<HashSet<String>>>,
    pub start_time: std::time::Instant,
}

pub async fn start_admin_server(
    pool: Arc<SqlitePool>,
    pokedex: Arc<PokedexData>,
    host: &str,
    port: u16,
    admin_key: String,
) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS redeem_codes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL DEFAULT '',
            rewards_json TEXT NOT NULL DEFAULT '{}',
            max_uses INTEGER DEFAULT NULL,
            current_uses INTEGER NOT NULL DEFAULT 0,
            max_per_user INTEGER NOT NULL DEFAULT 1,
            expires_at TEXT DEFAULT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(&*pool)
    .await;

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_redeemed_codes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            code_id INTEGER NOT NULL,
            code_text TEXT NOT NULL,
            rewards_summary TEXT NOT NULL DEFAULT '',
            redeemed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, code_id)
        );"
    )
    .execute(&*pool)
    .await;

    let state = AdminState {
        pool,
        pokedex,
        admin_key,
        sessions: Arc::new(RwLock::new(HashSet::new())),
        start_time: std::time::Instant::now(),
    };

    let addr = format!("{}:{}", host, port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            tracing::info!("Pure-Rust Gym Web Admin Panel running on http://{}", addr);
            l
        }
        Err(e) => {
            tracing::error!("Failed to bind Admin Panel to {}: {}", addr, e);
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    handle_connection(stream, state_clone).await;
                });
            }
            Err(e) => {
                tracing::warn!("Admin panel listener accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream, state: AdminState) {
    let mut buffer = Vec::with_capacity(8192);
    let mut temp = [0u8; 4096];

    let header_end_pos;
    let mut content_length: usize = 0;

    // Read headers
    loop {
        match stream.read(&mut temp).await {
            Ok(0) => return,
            Ok(n) => {
                buffer.extend_from_slice(&temp[..n]);
                if let Some(pos) = find_subsequence(&buffer, b"\r\n\r\n") {
                    header_end_pos = pos;
                    break;
                }
                if buffer.len() > 65536 {
                    return; // Protect against excessively large headers
                }
            }
            Err(_) => return,
        }
    }

    let header_pos = header_end_pos;

    let header_bytes = &buffer[..header_pos];
    let header_str = String::from_utf8_lossy(header_bytes);
    let mut header_lines = header_str.lines();

    let request_line = match header_lines.next() {
        Some(line) => line,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0].to_uppercase();
    let full_path = parts[1];

    let mut cookie_header = String::new();
    for line in header_lines {
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some((_, val)) = line.split_once(':') {
                content_length = val.trim().parse().unwrap_or(0);
            }
        } else if lower.starts_with("cookie:") {
            if let Some((_, val)) = line.split_once(':') {
                cookie_header = val.trim().to_string();
            }
        }
    }

    let body_start = header_pos + 4;
    let mut body_bytes = buffer[body_start..].to_vec();

    // Read remaining body if needed
    while body_bytes.len() < content_length {
        let to_read = (content_length - body_bytes.len()).min(temp.len());
        match stream.read(&mut temp[..to_read]).await {
            Ok(0) => break,
            Ok(n) => body_bytes.extend_from_slice(&temp[..n]),
            Err(_) => break,
        }
    }

    let (path, query) = match full_path.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (full_path, None),
    };

    // Extract admin session token from Cookie
    let session_token = extract_cookie(&cookie_header, "admin_session");
    let is_auth = if let Some(ref token) = session_token {
        state.sessions.read().await.contains(token)
    } else {
        false
    };

    // Routing
    if path == "/" {
        send_redirect(&mut stream, "/admin").await;
        return;
    }

    if path == "/login" {
        if method == "GET" {
            send_html(&mut stream, 200, &render_login_html(None)).await;
            return;
        } else if method == "POST" {
            let key = parse_form_or_json_key(&body_bytes);
            if key == state.admin_key {
                let token = uuid::Uuid::new_v4().to_string();
                state.sessions.write().await.insert(token.clone());
                let cookie = format!("admin_session={}; Path=/; HttpOnly; SameSite=Lax", token);
                send_redirect_with_cookie(&mut stream, "/admin", &cookie).await;
            } else {
                send_html(&mut stream, 200, &render_login_html(Some("Invalid administrator access key!"))).await;
            }
            return;
        }
    }

    if path == "/logout" {
        if let Some(token) = session_token {
            state.sessions.write().await.remove(&token);
        }
        send_redirect_with_cookie(&mut stream, "/login", "admin_session=; Path=/; Max-Age=0").await;
        return;
    }

    // Require authentication for all subsequent endpoints
    if !is_auth {
        if path.starts_with("/api/") {
            send_json(&mut stream, 401, &json!({"success": false, "error": "Unauthorized"})).await;
        } else {
            send_redirect(&mut stream, "/login").await;
        }
        return;
    }

    if path == "/admin" || path == "/admin/" {
        let html = render_home_html(&state).await;
        send_html(&mut stream, 200, &html).await;
        return;
    }

    if path == "/admin/gyms" || path == "/admin/gyms/" {
        let html = render_dashboard_html(&state).await;
        send_html(&mut stream, 200, &html).await;
        return;
    }

    if path == "/admin/gym" || path == "/admin/gym/" {
        send_redirect(&mut stream, "/admin/gyms").await;
        return;
    }

    if path.starts_with("/admin/gym/") {
        let sub = &path["/admin/gym/".len()..];
        let sub_parts: Vec<&str> = sub.split('/').collect();
        if sub_parts.len() == 2 {
            let region = sub_parts[0];
            let gym_num: usize = sub_parts[1].parse().unwrap_or(1);
            let html = render_editor_html(&state, region, gym_num).await;
            send_html(&mut stream, 200, &html).await;
            return;
        }
    }

    if path == "/api/stats" && method == "GET" {
        let stats = get_admin_stats(&state).await;
        send_json(&mut stream, 200, &json!({
            "success": true,
            "total_users": stats.total_users,
            "total_users_formatted": format_number(stats.total_users),
            "total_pokemon_caught": stats.total_pokemon_caught,
            "total_pokemon_caught_formatted": format_number(stats.total_pokemon_caught),
            "latency_ms": stats.latency_ms,
            "latency_str": stats.latency_str,
            "uptime_seconds": stats.uptime_seconds,
            "uptime_str": stats.uptime_str
        })).await;
        return;
    }

    if path == "/api/gyms" && method == "GET" {
        let res = handle_api_gyms(&state).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if path == "/api/gym/toggle_global" && method == "POST" {
        let res = handle_api_toggle_global(&state, &body_bytes).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if path == "/api/gym/toggle" && method == "POST" {
        let res = handle_api_toggle_gym(&state, &body_bytes).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if path == "/api/gym/save" && method == "POST" {
        let res = handle_api_save_gym(&state, &body_bytes).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if path == "/api/gym/reset" && method == "POST" {
        let res = handle_api_reset_gym(&state, &body_bytes).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if path == "/api/pokemon_moves" && method == "GET" {
        let res = handle_api_pokemon_moves(&state, query).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if (path == "/admin/redeem" || path == "/admin/redeem/") && method == "GET" {
        let html = render_redeem_html(&state).await;
        send_html(&mut stream, 200, &html).await;
        return;
    }

    if path == "/api/redeem/list" && method == "GET" {
        let res = handle_api_redeem_list(&state).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if path == "/api/redeem/create" && method == "POST" {
        let res = handle_api_redeem_create(&state, &body_bytes).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if path == "/api/redeem/toggle" && method == "POST" {
        let res = handle_api_redeem_toggle(&state, &body_bytes).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    if path == "/api/redeem/delete" && method == "POST" {
        let res = handle_api_redeem_delete(&state, &body_bytes).await;
        send_json(&mut stream, 200, &res).await;
        return;
    }

    send_text(&mut stream, 404, "Page Not Found").await;
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn extract_cookie(header: &str, name: &str) -> Option<String> {
    for part in header.split(';') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            if k.trim() == name {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn parse_form_or_json_key(bytes: &[u8]) -> String {
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if let Some(k) = v.get("admin_key").and_then(|x| x.as_str()) {
            return k.to_string();
        }
    }
    let s = String::from_utf8_lossy(bytes);
    for part in s.split('&') {
        if let Some((k, v)) = part.split_once('=') {
            if k == "admin_key" {
                return url_decode(v);
            }
        }
    }
    String::new()
}

fn url_decode(s: &str) -> String {
    let mut res = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            let hex_str = format!("{}{}", h1, h2);
            if let Ok(b) = u8::from_str_radix(&hex_str, 16) {
                res.push(b as char);
            }
        } else if c == '+' {
            res.push(' ');
        } else {
            res.push(c);
        }
    }
    res
}

async fn send_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, &str)],
) {
    let mut header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        status_code, status_text, content_type, body.len()
    );
    for (k, v) in extra_headers {
        header.push_str(&format!("{}: {}\r\n", k, v));
    }
    header.push_str("\r\n");
    let _ = stream.write_all(header.as_bytes()).await;
    let _ = stream.write_all(body).await;
    let _ = stream.flush().await;
}

async fn send_html(stream: &mut TcpStream, status_code: u16, html: &str) {
    send_response(stream, status_code, "OK", "text/html; charset=utf-8", html.as_bytes(), &[]).await;
}

async fn send_json(stream: &mut TcpStream, status_code: u16, val: &serde_json::Value) {
    let s = serde_json::to_string(val).unwrap_or_else(|_| "{}".to_string());
    send_response(stream, status_code, "OK", "application/json; charset=utf-8", s.as_bytes(), &[]).await;
}

async fn send_text(stream: &mut TcpStream, status_code: u16, text: &str) {
    send_response(stream, status_code, "OK", "text/plain; charset=utf-8", text.as_bytes(), &[]).await;
}

async fn send_redirect(stream: &mut TcpStream, location: &str) {
    send_response(stream, 302, "Found", "text/plain", b"", &[("Location", location)]).await;
}

async fn send_redirect_with_cookie(stream: &mut TcpStream, location: &str, cookie: &str) {
    send_response(
        stream,
        302,
        "Found",
        "text/plain",
        b"",
        &[("Location", location), ("Set-Cookie", cookie)],
    )
    .await;
}

// ============================================================================
// API Handlers
// ============================================================================

async fn handle_api_gyms(state: &AdminState) -> serde_json::Value {
    let rows = sqlx::query("SELECT * FROM gym_configs ORDER BY region ASC, gym_num ASC")
        .fetch_all(&*state.pool)
        .await
        .unwrap_or_default();

    let mut gyms = Vec::new();
    for r in rows {
        let team_json: String = r.try_get("team_json").unwrap_or_else(|_| "[]".to_string());
        let team: serde_json::Value = serde_json::from_str(&team_json).unwrap_or_else(|_| json!([]));
        let rules_json: String = r.try_get("rules_json").unwrap_or_else(|_| "{}".to_string());
        let rules: serde_json::Value = serde_json::from_str(&rules_json).unwrap_or_else(|_| json!({}));
        let rewards_json: String = r.try_get("rewards_json").unwrap_or_else(|_| "{}".to_string());
        let rewards: serde_json::Value = serde_json::from_str(&rewards_json).unwrap_or_else(|_| json!({}));
        gyms.push(json!({
            "region": r.try_get::<String, _>("region").unwrap_or_default(),
            "gym_num": r.try_get::<i64, _>("gym_num").unwrap_or(1),
            "is_open": r.try_get::<i64, _>("is_open").unwrap_or(1) != 0,
            "closed_reason": r.try_get::<String, _>("closed_reason").unwrap_or_default(),
            "leader_name": r.try_get::<String, _>("leader_name").unwrap_or_default(),
            "city": r.try_get::<String, _>("city").unwrap_or_default(),
            "gym_name": r.try_get::<String, _>("gym_name").unwrap_or_default(),
            "poke_type": r.try_get::<String, _>("poke_type").unwrap_or_default(),
            "fee": r.try_get::<i64, _>("fee").unwrap_or(200),
            "quote": r.try_get::<String, _>("quote").unwrap_or_default(),
            "badge": r.try_get::<String, _>("badge").unwrap_or_default(),
            "rules_text": r.try_get::<String, _>("rules_text").unwrap_or_default(),
            "rules_json": rules_json,
            "rules": rules,
            "rewards_text": r.try_get::<String, _>("rewards_text").unwrap_or_default(),
            "rewards_json": rewards_json,
            "rewards": rewards,
            "reward_money": r.try_get::<i64, _>("reward_money").unwrap_or(1000),
            "reward_badge": r.try_get::<String, _>("reward_badge").unwrap_or_default(),
            "signature_pokemon_id": r.try_get::<i64, _>("signature_pokemon_id").unwrap_or(0),
            "team": team,
        }));
    }
    json!({"success": true, "gyms": gyms})
}

async fn handle_api_toggle_global(state: &AdminState, body: &[u8]) -> serde_json::Value {
    let payload: serde_json::Value = serde_json::from_slice(body).unwrap_or_else(|_| json!({}));
    let open_val = if let Some(v) = payload.get("open") {
        v.as_bool().unwrap_or(v.as_i64().unwrap_or(1) != 0)
    } else if let Some(v) = payload.get("gyms_open") {
        v.as_bool().unwrap_or(v.as_i64().unwrap_or(1) != 0)
    } else {
        true
    };

    let new_val_int: i64 = if open_val { 1 } else { 0 };
    let reason = payload.get("closed_reason").and_then(|r| r.as_str());

    if let Some(r_str) = reason {
        if !r_str.trim().is_empty() {
            let _ = sqlx::query("UPDATE gym_global_settings SET gyms_open = ?, closed_reason = ? WHERE id = 1")
                .bind(new_val_int)
                .bind(r_str.trim())
                .execute(&*state.pool)
                .await;
            return json!({"success": true, "gyms_open": open_val});
        }
    }

    let _ = sqlx::query("UPDATE gym_global_settings SET gyms_open = ? WHERE id = 1")
        .bind(new_val_int)
        .execute(&*state.pool)
        .await;

    json!({"success": true, "gyms_open": open_val})
}

async fn handle_api_toggle_gym(state: &AdminState, body: &[u8]) -> serde_json::Value {
    let payload: serde_json::Value = serde_json::from_slice(body).unwrap_or_else(|_| json!({}));
    let region = payload.get("region").and_then(|r| r.as_str()).unwrap_or("").to_lowercase();
    let gym_num = payload.get("gym_num").and_then(|n| n.as_i64()).unwrap_or(1);

    let cur_open: Option<i64> = sqlx::query_scalar("SELECT is_open FROM gym_configs WHERE LOWER(region) = ? AND gym_num = ?")
        .bind(&region)
        .bind(gym_num)
        .fetch_optional(&*state.pool)
        .await
        .unwrap_or(None);

    let Some(current) = cur_open else {
        return json!({"success": false, "error": "Gym not found"});
    };

    let new_open = if let Some(v) = payload.get("is_open") {
        v.as_bool().unwrap_or(v.as_i64().unwrap_or(1) != 0)
    } else if let Some(v) = payload.get("open") {
        v.as_bool().unwrap_or(v.as_i64().unwrap_or(1) != 0)
    } else {
        current == 0
    };

    let new_open_int: i64 = if new_open { 1 } else { 0 };
    let reason = payload.get("closed_reason").and_then(|r| r.as_str());

    if let Some(r_str) = reason {
        if !r_str.trim().is_empty() {
            let _ = sqlx::query("UPDATE gym_configs SET is_open = ?, closed_reason = ? WHERE LOWER(region) = ? AND gym_num = ?")
                .bind(new_open_int)
                .bind(r_str.trim())
                .bind(&region)
                .bind(gym_num)
                .execute(&*state.pool)
                .await;
            return json!({"success": true, "is_open": new_open});
        }
    }

    let _ = sqlx::query("UPDATE gym_configs SET is_open = ? WHERE LOWER(region) = ? AND gym_num = ?")
        .bind(new_open_int)
        .bind(&region)
        .bind(gym_num)
        .execute(&*state.pool)
        .await;

    json!({"success": true, "is_open": new_open})
}

async fn handle_api_save_gym(state: &AdminState, body: &[u8]) -> serde_json::Value {
    let payload: serde_json::Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return json!({"success": false, "error": e.to_string()}),
    };

    let region = payload.get("region").and_then(|r| r.as_str()).unwrap_or("").to_lowercase();
    let gym_num = payload.get("gym_num").and_then(|n| n.as_i64()).unwrap_or(1);
    let is_open = if payload.get("is_open").and_then(|b| b.as_bool()).unwrap_or(true) { 1 } else { 0 };
    let closed_reason = payload.get("closed_reason").and_then(|s| s.as_str()).unwrap_or("This Gym is currently closed.");
    let leader_name = payload.get("leader_name").and_then(|s| s.as_str()).unwrap_or("Gym Leader");
    let city = payload.get("city").and_then(|s| s.as_str()).unwrap_or("City");
    let gym_name = payload.get("gym_name").and_then(|s| s.as_str()).unwrap_or("Gym");
    let poke_type = payload.get("poke_type").and_then(|s| s.as_str()).unwrap_or("Normal");
    let fee = payload.get("fee").and_then(|n| n.as_i64()).unwrap_or(200);
    let quote = payload.get("quote").and_then(|s| s.as_str()).unwrap_or("");
    let badge = payload.get("badge").and_then(|s| s.as_str()).unwrap_or("");
    let rules_text = payload.get("rules_text").and_then(|s| s.as_str()).unwrap_or("");
    let rewards_text = payload.get("rewards_text").and_then(|s| s.as_str()).unwrap_or("");
    let reward_money = payload.get("reward_money").and_then(|n| n.as_i64()).unwrap_or(fee * 5);
    let reward_badge = payload.get("reward_badge").and_then(|s| s.as_str()).unwrap_or(badge);
    let signature_pokemon_id = payload.get("signature_pokemon_id").and_then(|n| n.as_i64()).unwrap_or(0);

    let empty_team = json!([]);
    let team = payload.get("team").unwrap_or(&empty_team);
    let team_json = serde_json::to_string(team).unwrap_or_else(|_| "[]".to_string());

    let empty_rules = json!({});
    let rules_val = payload.get("rules").unwrap_or(&empty_rules);
    let rules_json = if let Some(s) = payload.get("rules_json").and_then(|v| v.as_str()) {
        s.to_string()
    } else {
        serde_json::to_string(rules_val).unwrap_or_else(|_| "{}".to_string())
    };

    let empty_rewards = json!({});
    let rewards_val = payload.get("rewards").unwrap_or(&empty_rewards);
    let rewards_json = if let Some(s) = payload.get("rewards_json").and_then(|v| v.as_str()) {
        s.to_string()
    } else {
        serde_json::to_string(rewards_val).unwrap_or_else(|_| "{}".to_string())
    };

    let res = sqlx::query(
        "UPDATE gym_configs
         SET is_open = ?, closed_reason = ?, leader_name = ?, city = ?, gym_name = ?,
             poke_type = ?, fee = ?, quote = ?, badge = ?, rules_text = ?,
             rewards_text = ?, reward_money = ?, reward_badge = ?,
             signature_pokemon_id = ?, team_json = ?, rules_json = ?, rewards_json = ?
         WHERE LOWER(region) = ? AND gym_num = ?"
    )
    .bind(is_open)
    .bind(closed_reason)
    .bind(leader_name)
    .bind(city)
    .bind(gym_name)
    .bind(poke_type)
    .bind(fee)
    .bind(quote)
    .bind(badge)
    .bind(rules_text)
    .bind(rewards_text)
    .bind(reward_money)
    .bind(reward_badge)
    .bind(signature_pokemon_id)
    .bind(&team_json)
    .bind(&rules_json)
    .bind(&rewards_json)
    .bind(&region)
    .bind(gym_num)
    .execute(&*state.pool)
    .await;

    match res {
        Ok(_) => json!({"success": true}),
        Err(e) => json!({"success": false, "error": e.to_string()}),
    }
}

async fn handle_api_reset_gym(state: &AdminState, body: &[u8]) -> serde_json::Value {
    let payload: serde_json::Value = serde_json::from_slice(body).unwrap_or_else(|_| json!({}));
    let region = payload.get("region").and_then(|r| r.as_str()).unwrap_or("").to_lowercase();
    let gym_num = payload.get("gym_num").and_then(|n| n.as_i64()).unwrap_or(1) as usize;

    let canonical: serde_json::Value = match serde_json::from_str(CANONICAL_GYMS_JSON) {
        Ok(v) => v,
        Err(e) => return json!({"success": false, "error": e.to_string()}),
    };

    let gym_arr = match canonical.get(&region).and_then(|arr| arr.as_array()) {
        Some(a) => a,
        None => return json!({"success": false, "error": "Region not found in canonical dataset"}),
    };

    if gym_num == 0 || gym_num > gym_arr.len() {
        return json!({"success": false, "error": "Gym number out of bounds"});
    }

    let leader = &gym_arr[gym_num - 1];
    let leader_name = leader.get("name").and_then(|s| s.as_str()).unwrap_or("Gym Leader");
    let city = leader.get("city").and_then(|s| s.as_str()).unwrap_or("City");
    let gym_name = leader.get("gym_name").and_then(|s| s.as_str()).unwrap_or("Gym");
    let poke_type = leader.get("poke_type").and_then(|s| s.as_str()).unwrap_or("Normal");
    let fee = leader.get("fee").and_then(|n| n.as_i64()).unwrap_or(200);
    let quote = leader.get("quote").and_then(|s| s.as_str()).unwrap_or("");
    let badge = leader.get("badge").and_then(|s| s.as_str()).unwrap_or("");
    let sig_id = leader.get("signature_pokemon_id").and_then(|n| n.as_i64()).unwrap_or(0);

    let team_pids = leader.get("team_pokemon_ids").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let mut team = Vec::new();

    for pid_val in team_pids {
        let pid = pid_val.as_i64().unwrap_or(1) as u32;
        let pkmn_entry = state.pokedex.get_pokemon_by_id(pid);
        let name = pkmn_entry.map(|p| p.name.clone()).unwrap_or_else(|| format!("Pokemon #{}", pid));

        let move_details = state.pokedex.get_level_up_moves(pid, 100);
        let moves_to_take = move_details.into_iter().rev().take(4).collect::<Vec<_>>();
        let mut custom_moves = Vec::new();
        for m in moves_to_take.into_iter().rev() {
            custom_moves.push(json!({
                "name": m.name,
                "type": m.t,
                "category": m.c,
                "power": m.p,
                "accuracy": m.a,
                "pp": m.pp
            }));
        }

        let evs = if pkmn_entry.map(|p| p.stats.attack >= p.stats.special_attack).unwrap_or(true) {
            json!({"hp": 6, "attack": 252, "defense": 0, "sp_attack": 0, "sp_defense": 0, "speed": 252})
        } else {
            json!({"hp": 6, "attack": 0, "defense": 0, "sp_attack": 252, "sp_defense": 0, "speed": 252})
        };

        team.push(json!({
            "pokemon_id": pid,
            "name": name,
            "level": 100,
            "nature": "Adamant",
            "ivs": {"hp": 31, "attack": 31, "defense": 31, "sp_attack": 31, "sp_defense": 31, "speed": 31},
            "evs": evs,
            "moves": custom_moves
        }));
    }

    let rules_text = format!(
        "Gym Rules:\n• 6v6 Pokémon Battle against {}\n• Level 100 Anime Signature Pokémon Team\n• Perfect IVs (186) & Max EVs (510)\n• Defeat all 6 of the Gym Leader's Pokémon to earn the Gym Badge!",
        leader_name
    );
    let rewards_text = format!("Gym Rewards:\n• {}\n• {} PokéDollars", badge, fee * 5);
    let team_json = serde_json::to_string(&team).unwrap_or_else(|_| "[]".to_string());

    let _ = sqlx::query(
        "UPDATE gym_configs
         SET is_open = 1, closed_reason = 'This Gym is currently closed.', leader_name = ?,
             city = ?, gym_name = ?, poke_type = ?, fee = ?, quote = ?, badge = ?,
             rules_text = ?, rewards_text = ?, reward_money = ?, reward_badge = ?,
             signature_pokemon_id = ?, team_json = ?, rules_json = '{}', rewards_json = '{}'
         WHERE LOWER(region) = ? AND gym_num = ?"
    )
    .bind(leader_name)
    .bind(city)
    .bind(gym_name)
    .bind(poke_type)
    .bind(fee)
    .bind(quote)
    .bind(badge)
    .bind(&rules_text)
    .bind(&rewards_text)
    .bind(fee * 5)
    .bind(badge)
    .bind(sig_id)
    .bind(&team_json)
    .bind(&region)
    .bind(gym_num as i64)
    .execute(&*state.pool)
    .await;

    json!({"success": true})
}

async fn handle_api_pokemon_moves(state: &AdminState, query: Option<&str>) -> serde_json::Value {
    let mut pid: u32 = 1;
    if let Some(q) = query {
        for part in q.split('&') {
            if let Some((k, v)) = part.split_once('=') {
                if k == "id" || k == "pokemon_id" {
                    pid = v.parse().unwrap_or(1);
                    break;
                }
            }
        }
    }

    let move_details = state.pokedex.get_level_up_moves(pid, 100);
    let mut arr = Vec::new();
    for m in move_details {
        arr.push(json!({
            "name": m.name,
            "type": m.t,
            "category": m.c,
            "power": m.p,
            "accuracy": m.a,
            "pp": m.pp
        }));
    }

    serde_json::Value::Array(arr)
}

// ============================================================================
// HTML Rendering
// ============================================================================

fn render_login_html(error: Option<&str>) -> String {
    let err_markup = if let Some(e) = error {
        format!("<div class=\"error\">{}</div>", html_escape(e))
    } else {
        String::new()
    };

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Gym League Management - Admin Login</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        body {{ background: #0b0f19; color: #f3f4f6; display: flex; align-items: center; justify-content: center; min-height: 100vh; }}
        .card {{ background: #111827; border: 1px solid #1f2937; border-radius: 12px; padding: 32px; width: 100%; max-width: 420px; box-shadow: 0 10px 25px rgba(0,0,0,0.5); }}
        .title {{ font-size: 22px; font-weight: 700; color: #38bdf8; margin-bottom: 6px; text-align: center; }}
        .sub {{ font-size: 13px; color: #9ca3af; margin-bottom: 24px; text-align: center; }}
        .form-group {{ margin-bottom: 20px; }}
        label {{ display: block; font-size: 13px; font-weight: 500; margin-bottom: 8px; color: #d1d5db; }}
        input[type="password"] {{ width: 100%; padding: 12px 14px; background: #1f2937; border: 1px solid #374151; border-radius: 8px; color: white; font-size: 14px; outline: none; }}
        input[type="password"]:focus {{ border-color: #38bdf8; ring: 2px solid #38bdf8; }}
        button {{ width: 100%; padding: 12px; background: #0284c7; color: white; font-weight: 600; border: none; border-radius: 8px; font-size: 15px; cursor: pointer; transition: background 0.2s; }}
        button:hover {{ background: #0369a1; }}
        .error {{ color: #f87171; font-size: 13px; margin-bottom: 16px; text-align: center; }}
    </style>
</head>
<body>
    <div class="card">
        <div class="title">Pokémon League Admin</div>
        <div class="sub">Gym Management Control Center</div>
        {}
        <form method="POST" action="/login">
            <div class="form-group">
                <label for="admin_key">Admin Key / Secret Password</label>
                <input type="password" id="admin_key" name="admin_key" placeholder="Enter Admin Secret..." required autofocus>
            </div>
            <button type="submit">Access Control Center</button>
        </form>
    </div>
</body>
</html>"#, err_markup)
}

fn format_number(n: i64) -> String {
    let s = n.to_string();
    let is_negative = s.starts_with('-');
    let digits = if is_negative { &s[1..] } else { &s };
    let mut result = String::new();
    let len = digits.len();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    if is_negative {
        format!("-{}", result)
    } else {
        result
    }
}

fn format_uptime(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, mins, s)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, mins, s)
    } else if mins > 0 {
        format!("{}m {}s", mins, s)
    } else {
        format!("{}s", s)
    }
}

pub struct AdminStats {
    pub total_users: i64,
    pub total_pokemon_caught: i64,
    pub latency_ms: f64,
    pub latency_str: String,
    pub uptime_seconds: u64,
    pub uptime_str: String,
}

pub async fn get_admin_stats(state: &AdminState) -> AdminStats {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&*state.pool)
        .await
        .unwrap_or(0);

    let caught_from_users: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(pokemons_caught), 0) FROM users")
        .fetch_one(&*state.pool)
        .await
        .unwrap_or(0);
    let caught_from_pokemons: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pokemons")
        .fetch_one(&*state.pool)
        .await
        .unwrap_or(0);
    let total_pokemon_caught = std::cmp::max(caught_from_users, caught_from_pokemons);

    let t0 = std::time::Instant::now();
    let _ = sqlx::query("SELECT 1").execute(&*state.pool).await;
    let latency_duration = t0.elapsed();
    let latency_ms = latency_duration.as_secs_f64() * 1000.0;
    let latency_str = if latency_ms < 1.0 {
        format!("{:.2} ms", latency_ms)
    } else {
        format!("{:.1} ms", latency_ms)
    };

    let uptime_duration = state.start_time.elapsed();
    let uptime_seconds = uptime_duration.as_secs();
    let uptime_str = format_uptime(uptime_duration);

    AdminStats {
        total_users,
        total_pokemon_caught,
        latency_ms,
        latency_str,
        uptime_seconds,
        uptime_str,
    }
}

fn get_navbar_css() -> &'static str {
    r#"
    .admin-navbar {
        background: #0f172a;
        border-bottom: 1px solid #1e293b;
        padding: 0 28px;
        height: 64px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        position: sticky;
        top: 0;
        z-index: 1000;
    }
    .nav-left {
        display: flex;
        align-items: center;
        gap: 32px;
    }
    .nav-brand {
        font-size: 18px;
        font-weight: 700;
        color: #38bdf8;
        text-decoration: none;
        letter-spacing: -0.02em;
    }
    .nav-brand:hover {
        color: #7dd3fc;
    }
    .nav-links {
        display: flex;
        align-items: center;
        gap: 6px;
    }
    .nav-link {
        color: #94a3b8;
        text-decoration: none;
        font-size: 14px;
        font-weight: 500;
        padding: 8px 16px;
        border-radius: 6px;
        transition: all 0.15s ease;
    }
    .nav-link:hover {
        color: #f1f5f9;
        background: #1e293b;
    }
    .nav-link.active {
        color: #38bdf8;
        background: #1e293b;
        font-weight: 600;
        box-shadow: inset 0 0 0 1px #334155;
    }
    .nav-right {
        display: flex;
        align-items: center;
        gap: 16px;
    }
    .status-pill {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 4px 12px;
        border-radius: 9999px;
        font-size: 12px;
        font-weight: 600;
        background: rgba(16, 185, 129, 0.12);
        color: #34d399;
        border: 1px solid rgba(16, 185, 129, 0.25);
    }
    .status-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: #34d399;
        box-shadow: 0 0 8px #34d399;
    }
    .btn-logout {
        background: #1e293b;
        color: #cbd5e1;
        border: 1px solid #334155;
        padding: 7px 14px;
        border-radius: 6px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        text-decoration: none;
        transition: all 0.15s ease;
    }
    .btn-logout:hover {
        background: #334155;
        color: #ffffff;
        border-color: #475569;
    }
    @media (max-width: 640px) {
        .admin-navbar { padding: 0 14px; height: 56px; }
        .nav-left { gap: 12px; }
        .nav-brand { font-size: 16px; }
        .nav-link { padding: 6px 10px; font-size: 13px; }
        .status-pill { display: none; }
    }
    "#
}

fn render_navbar(active_page: &str) -> String {
    let home_active = if active_page == "home" { "active" } else { "" };
    let gyms_active = if active_page == "gyms" { "active" } else { "" };
    let redeem_active = if active_page == "redeem" { "active" } else { "" };

    format!(r#"
    <header class="admin-navbar">
        <div class="nav-left">
            <a href="/admin" class="nav-brand">Ozone Admin</a>
            <nav class="nav-links">
                <a href="/admin" class="nav-link {}">Home</a>
                <a href="/admin/gyms" class="nav-link {}">Gyms</a>
                <a href="/admin/redeem" class="nav-link {}">Redeem Codes</a>
            </nav>
        </div>
        <div class="nav-right">
            <span class="status-pill"><span class="status-dot"></span> Online</span>
            <a href="/logout" class="btn-logout">Logout</a>
        </div>
    </header>
    "#, home_active, gyms_active, redeem_active)
}

async fn render_home_html(state: &AdminState) -> String {
    let stats = get_admin_stats(state).await;

    let navbar = render_navbar("home");
    let navbar_css = get_navbar_css();

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Admin Dashboard - Ozone</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        body {{ background: #090d16; color: #e2e8f0; min-height: 100vh; display: flex; flex-direction: column; }}
        {}
        main {{ max-width: 1200px; margin: 0 auto; padding: 32px 24px; width: 100%; flex: 1; }}

        .page-header {{ margin-bottom: 28px; }}
        .page-title {{ font-size: 26px; font-weight: 700; color: #f8fafc; letter-spacing: -0.02em; margin-bottom: 6px; }}
        .page-desc {{ font-size: 14px; color: #94a3b8; }}

        .kpi-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 20px; margin-bottom: 32px; }}
        .kpi-card {{ background: #111827; border: 1px solid #1e293b; border-radius: 12px; padding: 22px; display: flex; flex-direction: column; transition: transform 0.15s ease, border-color 0.15s ease; }}
        .kpi-card:hover {{ border-color: #334155; transform: translateY(-2px); }}
        .kpi-head {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }}
        .kpi-title {{ font-size: 13px; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; }}
        .kpi-badge {{ font-size: 11px; font-weight: 600; padding: 3px 8px; border-radius: 9999px; }}
        .badge-blue {{ background: rgba(56, 189, 248, 0.15); color: #38bdf8; border: 1px solid rgba(56, 189, 248, 0.3); }}
        .badge-purple {{ background: rgba(168, 85, 247, 0.15); color: #c084fc; border: 1px solid rgba(168, 85, 247, 0.3); }}
        .badge-green {{ background: rgba(52, 211, 153, 0.15); color: #34d399; border: 1px solid rgba(52, 211, 153, 0.3); }}
        .badge-amber {{ background: rgba(251, 191, 36, 0.15); color: #fbbf24; border: 1px solid rgba(251, 191, 36, 0.3); }}
        .kpi-val {{ font-size: 32px; font-weight: 700; color: #f8fafc; letter-spacing: -0.02em; margin-bottom: 6px; }}
        .kpi-sub {{ font-size: 12px; color: #64748b; }}

        footer {{ text-align: center; padding: 24px; font-size: 12px; color: #475569; border-top: 1px solid #1e293b; margin-top: auto; }}
    </style>
</head>
<body>
    {}
    <main>
        <div class="page-header">
            <h1 class="page-title">System Dashboard</h1>
            <p class="page-desc">Real-time telemetry, bot user base, game statistics, and system health.</p>
        </div>

        <div class="kpi-grid">
            <div class="kpi-card">
                <div class="kpi-head">
                    <span class="kpi-title">Total Users</span>
                    <span class="kpi-badge badge-blue">Trainers</span>
                </div>
                <div class="kpi-val" id="metric-total-users">{}</div>
                <div class="kpi-sub">Registered accounts in database</div>
            </div>

            <div class="kpi-card">
                <div class="kpi-head">
                    <span class="kpi-title">Total Pokémon Caught</span>
                    <span class="kpi-badge badge-purple">Catches</span>
                </div>
                <div class="kpi-val" id="metric-total-caught">{}</div>
                <div class="kpi-sub">Lifetime wild & safari captures</div>
            </div>

            <div class="kpi-card">
                <div class="kpi-head">
                    <span class="kpi-title">Latency</span>
                    <span class="kpi-badge badge-green">Healthy</span>
                </div>
                <div class="kpi-val" id="metric-latency">{}</div>
                <div class="kpi-sub">Database roundtrip ping response</div>
            </div>

            <div class="kpi-card">
                <div class="kpi-head">
                    <span class="kpi-title">Uptime</span>
                    <span class="kpi-badge badge-amber">Online</span>
                </div>
                <div class="kpi-val" id="metric-uptime">{}</div>
                <div class="kpi-sub">Continuous bot service runtime</div>
            </div>
        </div>
    </main>

    <footer>
        Ozone Pokémon Bot &bull; Administrator Control Panel
    </footer>

    <script>
        let uptimeSec = {};
        function formatUptime(secs) {{
            const d = Math.floor(secs / 86400);
            const h = Math.floor((secs % 86400) / 3600);
            const m = Math.floor((secs % 3600) / 60);
            const s = secs % 60;
            if (d > 0) return `${{d}}d ${{h}}h ${{m}}m ${{s}}s`;
            if (h > 0) return `${{h}}h ${{m}}m ${{s}}s`;
            if (m > 0) return `${{m}}m ${{s}}s`;
            return `${{s}}s`;
        }}

        setInterval(() => {{
            uptimeSec++;
            const el = document.getElementById('metric-uptime');
            if (el) el.textContent = formatUptime(uptimeSec);
        }}, 1000);

        async function updateStats() {{
            try {{
                const res = await fetch('/api/stats');
                if (res.ok) {{
                    const d = await res.json();
                    if (d.success) {{
                        const elUsers = document.getElementById('metric-total-users');
                        if (elUsers) elUsers.textContent = d.total_users_formatted;
                        const elCaught = document.getElementById('metric-total-caught');
                        if (elCaught) elCaught.textContent = d.total_pokemon_caught_formatted;
                        const elLat = document.getElementById('metric-latency');
                        if (elLat) elLat.textContent = d.latency_str;
                        uptimeSec = d.uptime_seconds;
                        const elUp = document.getElementById('metric-uptime');
                        if (elUp) elUp.textContent = d.uptime_str;
                    }}
                }}
            }} catch (e) {{}}
        }}
        setInterval(updateStats, 5000);
    </script>
</body>
</html>"#,
        navbar_css,
        navbar,
        format_number(stats.total_users),
        format_number(stats.total_pokemon_caught),
        stats.latency_str,
        stats.uptime_str,
        stats.uptime_seconds
    )
}

async fn render_dashboard_html(state: &AdminState) -> String {
    let global_row = sqlx::query("SELECT gyms_open, closed_reason FROM gym_global_settings WHERE id = 1")
        .fetch_optional(&*state.pool)
        .await
        .ok()
        .flatten();

    let (global_open, global_reason) = if let Some(r) = global_row {
        let o: i64 = r.try_get("gyms_open").unwrap_or(1);
        let s: String = r.try_get("closed_reason").unwrap_or_else(|_| "Gyms are temporarily closed by the Pokémon League.".to_string());
        (o != 0, s)
    } else {
        (true, "Gyms are temporarily closed by the Pokémon League.".to_string())
    };

    let rows = sqlx::query("SELECT * FROM gym_configs ORDER BY region ASC, gym_num ASC")
        .fetch_all(&*state.pool)
        .await
        .unwrap_or_default();

    let total_gyms = rows.len();
    let mut open_gyms = 0;
    let mut kanto_cards = String::new();
    let mut johto_cards = String::new();
    let mut hoenn_cards = String::new();
    let mut sinnoh_cards = String::new();

    for r in &rows {
        let is_open = r.try_get::<i64, _>("is_open").unwrap_or(1) != 0;
        if is_open {
            open_gyms += 1;
        }

        let region: String = r.try_get("region").unwrap_or_default();
        let gym_num: i64 = r.try_get("gym_num").unwrap_or(1);
        let leader_name: String = r.try_get("leader_name").unwrap_or_default();
        let city: String = r.try_get("city").unwrap_or_default();
        let gym_name: String = r.try_get("gym_name").unwrap_or_default();
        let poke_type: String = r.try_get("poke_type").unwrap_or_default();
        let fee: i64 = r.try_get("fee").unwrap_or(200);
        let reward_money: i64 = r.try_get("reward_money").unwrap_or(1000);
        let badge: String = r.try_get("badge").unwrap_or_default();
        let sig_id: i64 = r.try_get("signature_pokemon_id").unwrap_or(0);
        let team_json: String = r.try_get("team_json").unwrap_or_else(|_| "[]".to_string());

        let team_count = serde_json::from_str::<serde_json::Value>(&team_json)
            .ok()
            .and_then(|v| v.as_array().map(|a| a.len()))
            .unwrap_or(6);

        let status_badge = if is_open {
            "<span class=\"badge-open\">OPEN</span>"
        } else {
            "<span class=\"badge-closed\">CLOSED</span>"
        };

        let toggle_btn = if is_open {
            format!("<button class=\"btn-sm btn-close\" onclick=\"toggleGym('{}', {}, false)\">Close</button>", region, gym_num)
        } else {
            format!("<button class=\"btn-sm btn-open\" onclick=\"toggleGym('{}', {}, true)\">Open</button>", region, gym_num)
        };

        let card_html = format!(r#"
        <div class="gym-card">
            <div class="gym-header">
                <div>
                    <span class="gym-num">Gym #{}</span>
                    <div class="gym-name">{}</div>
                    <div class="gym-city">{}</div>
                </div>
                <div style="text-align: right;">
                    <span class="type-tag">{}</span>
                    <div style="margin-top: 6px;">{}</div>
                </div>
            </div>
            <div class="gym-body">
                <div class="info-row"><span class="info-label">Leader:</span><span class="info-val">{}</span></div>
                <div class="info-row"><span class="info-label">Badge:</span><span class="info-val">{}</span></div>
                <div class="info-row"><span class="info-label">Fee:</span><span class="info-val">{} PokéDollars</span></div>
                <div class="info-row"><span class="info-label">Reward:</span><span class="info-val">{} PokéDollars</span></div>
                <div class="info-row"><span class="info-label">Team:</span><span class="info-val">{} Pokémon (Lv. 100)</span></div>
                <div class="info-row"><span class="info-label">Signature ID:</span><span class="info-val">#{}</span></div>
            </div>
            <div class="gym-actions">
                <a class="btn-sm btn-edit" href="/admin/gym/{}/{}">Edit Gym</a>
                {}
            </div>
        </div>
        "#, gym_num, html_escape(&gym_name), html_escape(&city), html_escape(&poke_type), status_badge,
            html_escape(&leader_name), html_escape(&badge), fee, reward_money, team_count, sig_id,
            region, gym_num, toggle_btn);

        match region.to_lowercase().as_str() {
            "kanto" => kanto_cards.push_str(&card_html),
            "johto" => johto_cards.push_str(&card_html),
            "hoenn" => hoenn_cards.push_str(&card_html),
            "sinnoh" => sinnoh_cards.push_str(&card_html),
            _ => {}
        }
    }

    let global_status_badge = if global_open {
        "<span class=\"badge-open\">SYSTEM: ALL GYMS OPEN</span>"
    } else {
        "<span class=\"badge-closed\">SYSTEM: ALL GYMS CLOSED</span>"
    };

    let global_toggle_btn = if global_open {
        "<button class=\"global-btn btn-close-all\" onclick=\"toggleGlobal(false)\">Close All Gyms</button>"
    } else {
        "<button class=\"global-btn btn-open-all\" onclick=\"toggleGlobal(true)\">Open All Gyms</button>"
    };

    let navbar = render_navbar("gyms");
    let navbar_css = get_navbar_css();

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Gym League Management - Admin Dashboard</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        body {{ background: #090d16; color: #e2e8f0; min-height: 100vh; display: flex; flex-direction: column; }}
        {}
        main {{ max-width: 1300px; margin: 0 auto; padding: 24px; width: 100%; flex: 1; }}

        .global-card {{ background: #131d2e; border: 1px solid #1e293b; border-radius: 12px; padding: 20px 24px; margin-bottom: 24px; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 16px; }}
        .global-info h2 {{ font-size: 18px; font-weight: 700; margin-bottom: 4px; display: flex; align-items: center; gap: 10px; }}
        .global-info p {{ font-size: 13px; color: #94a3b8; }}
        .badge-open {{ background: #065f46; color: #6ee7b7; padding: 4px 10px; border-radius: 9999px; font-size: 12px; font-weight: 600; }}
        .badge-closed {{ background: #7f1d1d; color: #fca5a5; padding: 4px 10px; border-radius: 9999px; font-size: 12px; font-weight: 600; }}
        .global-btn {{ padding: 10px 18px; border-radius: 8px; font-weight: 600; font-size: 14px; border: none; cursor: pointer; }}
        .btn-close-all {{ background: #dc2626; color: white; }}
        .btn-close-all:hover {{ background: #b91c1c; }}
        .btn-open-all {{ background: #059669; color: white; }}
        .btn-open-all:hover {{ background: #047857; }}

        .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 24px; }}
        .stat-card {{ background: #111827; border: 1px solid #1f2937; border-radius: 10px; padding: 16px; }}
        .stat-label {{ font-size: 12px; color: #9ca3af; text-transform: uppercase; font-weight: 600; margin-bottom: 6px; }}
        .stat-val {{ font-size: 26px; font-weight: 700; color: #f3f4f6; }}

        .tabs {{ display: flex; gap: 8px; border-bottom: 1px solid #1f2937; margin-bottom: 20px; overflow-x: auto; }}
        .tab-btn {{ background: transparent; border: none; padding: 10px 20px; font-size: 15px; font-weight: 600; color: #9ca3af; cursor: pointer; border-radius: 8px 8px 0 0; border-bottom: 2px solid transparent; }}
        .tab-btn:hover {{ color: #f3f4f6; }}
        .tab-btn.active {{ color: #38bdf8; border-bottom: 2px solid #38bdf8; background: #111827; }}

        .gym-grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(290px, 1fr)); gap: 20px; }}
        .gym-card {{ background: #111827; border: 1px solid #1f2937; border-radius: 12px; padding: 20px; display: flex; flex-direction: column; }}
        .gym-card:hover {{ border-color: #374151; }}
        .gym-header {{ display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px; }}
        .gym-num {{ font-size: 12px; color: #9ca3af; font-weight: 700; text-transform: uppercase; }}
        .gym-name {{ font-size: 17px; font-weight: 700; color: #f9fafb; margin-top: 2px; }}
        .gym-city {{ font-size: 13px; color: #94a3b8; }}
        .type-tag {{ font-size: 12px; font-weight: 600; padding: 2px 8px; border-radius: 4px; background: #1f2937; color: #e5e7eb; border: 1px solid #374151; }}

        .gym-body {{ margin-bottom: 16px; flex: 1; }}
        .info-row {{ display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 6px; }}
        .info-label {{ color: #94a3b8; }}
        .info-val {{ font-weight: 600; color: #e2e8f0; }}

        .gym-actions {{ display: flex; gap: 8px; border-top: 1px solid #1f2937; padding-top: 14px; margin-top: auto; }}
        .btn-sm {{ flex: 1; text-align: center; padding: 8px 12px; border-radius: 6px; font-size: 13px; font-weight: 600; text-decoration: none; border: none; cursor: pointer; }}
        .btn-edit {{ background: #0284c7; color: white; }}
        .btn-edit:hover {{ background: #0369a1; }}
        .btn-close {{ background: #ef4444; color: white; }}
        .btn-close:hover {{ background: #dc2626; }}
        .btn-open {{ background: #10b981; color: white; }}
        .btn-open:hover {{ background: #059669; }}
    </style>
</head>
<body>
    {}

    <main>
        <div class="global-card">
            <div class="global-info">
                <h2>Gym Operations {}</h2>
                <p>Status: {}</p>
            </div>
            <div>
                {}
            </div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-label">Total Gyms</div>
                <div class="stat-val">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Open Gyms</div>
                <div class="stat-val" style="color: #6ee7b7;">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Closed Gyms</div>
                <div class="stat-val" style="color: #fca5a5;">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Regions Managed</div>
                <div class="stat-val" style="color: #38bdf8;">4</div>
            </div>
        </div>

        <div class="tabs">
            <button class="tab-btn active" onclick="switchTab('kanto')">Kanto</button>
            <button class="tab-btn" onclick="switchTab('johto')">Johto</button>
            <button class="tab-btn" onclick="switchTab('hoenn')">Hoenn</button>
            <button class="tab-btn" onclick="switchTab('sinnoh')">Sinnoh</button>
        </div>

        <div id="tab-kanto" class="tab-content">
            <div class="gym-grid">{}</div>
        </div>
        <div id="tab-johto" class="tab-content" style="display: none;">
            <div class="gym-grid">{}</div>
        </div>
        <div id="tab-hoenn" class="tab-content" style="display: none;">
            <div class="gym-grid">{}</div>
        </div>
        <div id="tab-sinnoh" class="tab-content" style="display: none;">
            <div class="gym-grid">{}</div>
        </div>
    </main>

    <script>
        function switchTab(region) {{
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.style.display = 'none');
            event.target.classList.add('active');
            document.getElementById('tab-' + region).style.display = 'block';
        }}

        async function toggleGlobal(open) {{
            const reason = open ? "" : prompt("Enter closed reason notice for players:", "Gyms are temporarily closed for League tournament!");
            if (!open && reason === null) return;
            try {{
                const res = await fetch('/api/gym/toggle_global', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ open: open, closed_reason: reason || '' }})
                }});
                const data = await res.json();
                if (data.success) location.reload();
                else alert("Error: " + (data.error || 'Failed'));
            }} catch (err) {{
                alert("Network error: " + err);
            }}
        }}

        async function toggleGym(region, gymNum, open) {{
            const reason = open ? "" : prompt("Enter closed reason notice for this Gym:", "This Gym is currently closed for maintenance.");
            if (!open && reason === null) return;
            try {{
                const res = await fetch('/api/gym/toggle', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ region: region, gym_num: gymNum, is_open: open, closed_reason: reason || '' }})
                }});
                const data = await res.json();
                if (data.success) location.reload();
                else alert("Error: " + (data.error || 'Failed'));
            }} catch (err) {{
                alert("Network error: " + err);
            }}
        }}
    </script>
</body>
</html>"#,
        navbar_css,
        navbar,
        global_status_badge,
        html_escape(&global_reason),
        global_toggle_btn,
        total_gyms,
        open_gyms,
        total_gyms - open_gyms,
        kanto_cards,
        johto_cards,
        hoenn_cards,
        sinnoh_cards
    )
}

async fn render_editor_html(state: &AdminState, region: &str, gym_num: usize) -> String {
    let row = sqlx::query("SELECT * FROM gym_configs WHERE LOWER(region) = LOWER(?) AND gym_num = ?")
        .bind(region)
        .bind(gym_num as i64)
        .fetch_optional(&*state.pool)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return "<h1>Gym not found</h1><p><a href=\"/admin\">Back to Dashboard</a></p>".to_string();
    };

    let leader_name: String = row.try_get("leader_name").unwrap_or_default();
    let city: String = row.try_get("city").unwrap_or_default();
    let gym_name: String = row.try_get("gym_name").unwrap_or_default();
    let poke_type: String = row.try_get("poke_type").unwrap_or_default();
    let fee: i64 = row.try_get("fee").unwrap_or(200);
    let quote: String = row.try_get("quote").unwrap_or_default();
    let badge: String = row.try_get("badge").unwrap_or_default();
    let rules_text: String = row.try_get("rules_text").unwrap_or_default();
    let rewards_text: String = row.try_get("rewards_text").unwrap_or_default();
    let reward_money: i64 = row.try_get("reward_money").unwrap_or(1000);
    let _reward_badge: String = row.try_get("reward_badge").unwrap_or_else(|_| badge.clone());
    let sig_id: i64 = row.try_get("signature_pokemon_id").unwrap_or(0);
    let is_open: bool = row.try_get::<i64, _>("is_open").unwrap_or(1) != 0;
    let closed_reason: String = row.try_get("closed_reason").unwrap_or_default();
    let team_json: String = row.try_get("team_json").unwrap_or_else(|_| "[]".to_string());
    let rules_json: String = row.try_get("rules_json").unwrap_or_else(|_| "{}".to_string());
    let rules_config: crate::GymRulesConfig = serde_json::from_str(&rules_json).unwrap_or_default();
    let rewards_json: String = row.try_get("rewards_json").unwrap_or_else(|_| "{}".to_string());
    let rewards_config: crate::GymRewardsConfig = serde_json::from_str(&rewards_json).unwrap_or_default();
    let reward_money = rewards_config.pokedollars.unwrap_or(reward_money);

    let effective_rules_text = {
        let bullets = rules_config.to_bullet_rules();
        if !bullets.is_empty() {
            let mut all_bullets = bullets;
            all_bullets.push(format!("• 6v6 Pokémon Battle against {}", leader_name));
            all_bullets.push("• Level 100 Anime Signature Pokémon Team".to_string());
            all_bullets.push("• Perfect IVs (186) & Max EVs (510)".to_string());
            all_bullets.push("• Defeat all 6 of the Gym Leader's Pokémon to earn the Gym Badge!".to_string());
            format!("Gym Rules:\n{}", all_bullets.join("\n"))
        } else if !rules_text.trim().is_empty() {
            rules_text.clone()
        } else {
            format!(
                "Gym Rules:\n• 6v6 Pokémon Battle against {}\n• Level 100 Anime Signature Pokémon Team\n• Perfect IVs (186) & Max EVs (510)\n• Defeat all 6 of the Gym Leader's Pokémon to earn the Gym Badge!",
                leader_name
            )
        }
    };

    let effective_rewards_text = {
        let bullets = rewards_config.to_bullet_rewards_with_pokedex(&badge, reward_money, &state.pokedex);
        if !bullets.is_empty() {
            format!("Gym Rewards:\n{}", bullets.join("\n"))
        } else if !rewards_text.trim().is_empty() {
            rewards_text.clone()
        } else {
            format!("Gym Rewards:\n• {}\n• {} PokéDollars 💵", badge, reward_money)
        }
    };

    const ALL_TYPES: &[&str] = &[
        "Normal", "Fire", "Water", "Grass", "Electric", "Ice",
        "Fighting", "Poison", "Ground", "Flying", "Psychic", "Bug",
        "Rock", "Ghost", "Dragon", "Steel", "Dark", "Fairy",
    ];

    let mut type_checkboxes = String::new();
    for t in ALL_TYPES {
        let checked = if rules_config.banned_types.iter().any(|bt| bt.eq_ignore_ascii_case(t)) {
            "checked"
        } else {
            ""
        };
        type_checkboxes.push_str(&format!(
            r#"<label class="type-pill">
                <input type="checkbox" name="banned_type" value="{}" {}>
                <span>{}</span>
            </label>"#,
            t, checked, t
        ));
    }

    let ban_leg_checked = if rules_config.ban_legendaries { "checked" } else { "" };
    let ban_myth_checked = if rules_config.ban_mythicals { "checked" } else { "" };
    let ban_ub_checked = if rules_config.ban_ultra_beasts { "checked" } else { "" };
    let ban_paradox_checked = if rules_config.ban_paradox { "checked" } else { "" };
    let no_dupes_checked = if rules_config.no_duplicates { "checked" } else { "" };
    let req_conscious_checked = if rules_config.require_conscious { "checked" } else { "" };
    let max_lvl_val = rules_config.max_level.map(|l| l.to_string()).unwrap_or_default();

    let mut pokemon_options = String::new();
    let mut sorted_pokemon: Vec<_> = state.pokedex.pokemon.values().collect();
    sorted_pokemon.sort_by_key(|p| p.id);
    for p in &sorted_pokemon {
        pokemon_options.push_str(&format!(
            "<option value=\"{}\">#{} {}</option>",
            p.id, p.id, html_escape(&p.name)
        ));
    }

    let current_ms = rewards_config.mega_stone.as_deref().unwrap_or("");
    let mut mega_stone_options = format!(
        "<option value=\"\" {}>None</option>\n<option value=\"random\" {}>Random Mega Stone</option>",
        if current_ms.is_empty() { "selected" } else { "" },
        if current_ms == "random" { "selected" } else { "" }
    );
    for &stone in crate::ALL_MEGA_STONES {
        let sel = if current_ms == stone { "selected" } else { "" };
        mega_stone_options.push_str(&format!(
            "\n<option value=\"{}\" {}>{}</option>",
            html_escape(stone), sel, html_escape(stone)
        ));
    }

    let current_tm = rewards_config.tm.as_deref().unwrap_or("");
    let mut tm_options = format!(
        "<option value=\"\" {}>None</option>\n<option value=\"random\" {}>Random TM</option>",
        if current_tm.is_empty() { "selected" } else { "" },
        if current_tm == "random" { "selected" } else { "" }
    );
    for tm in crate::TM_CATALOG {
        let num_str = tm.number.to_string();
        let sel = if current_tm == num_str || current_tm.eq_ignore_ascii_case(tm.name) { "selected" } else { "" };
        tm_options.push_str(&format!(
            "\n<option value=\"{}\" {}>TM #{:03} - {} ({})</option>",
            tm.number, sel, tm.number, html_escape(tm.name), html_escape(tm.move_type)
        ));
    }

    let current_egg = rewards_config.egg.as_deref().unwrap_or("");
    let mut egg_options = format!(
        "<option value=\"\" {}>None</option>\n<option value=\"random\" {}>Random Pokémon Egg</option>",
        if current_egg.is_empty() { "selected" } else { "" },
        if current_egg == "random" { "selected" } else { "" }
    );
    for p in &sorted_pokemon {
        let id_str = p.id.to_string();
        let sel = if current_egg == id_str || current_egg.eq_ignore_ascii_case(&p.name) { "selected" } else { "" };
        egg_options.push_str(&format!(
            "\n<option value=\"{}\" {}>#{} {} Egg</option>",
            p.id, sel, p.id, html_escape(&p.name)
        ));
    }

    let navbar = render_navbar("gyms");
    let navbar_css = get_navbar_css();

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Edit Gym - {}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        body {{ background: #090d16; color: #e2e8f0; min-height: 100vh; padding-bottom: 60px; }}
        {}
        .editor-subnav {{ background: #111827; border-bottom: 1px solid #1f2937; padding: 14px 24px; display: flex; justify-content: space-between; align-items: center; position: sticky; top: 64px; z-index: 90; }}
        .brand {{ font-size: 18px; font-weight: 700; color: #38bdf8; display: flex; align-items: center; gap: 8px; }}
        .header-actions {{ display: flex; align-items: center; gap: 12px; }}
        .btn {{ padding: 8px 16px; border-radius: 6px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; text-decoration: none; }}
        .btn-back {{ background: #374151; color: #e5e7eb; }}
        .btn-save {{ background: #0284c7; color: white; }}
        .btn-save:hover {{ background: #0369a1; }}
        .btn-reset {{ background: #b91c1c; color: white; }}
        .btn-reset:hover {{ background: #991b1b; }}
        
        main {{ max-width: 1100px; margin: 24px auto; padding: 0 20px; }}
        .card {{ background: #111827; border: 1px solid #1f2937; border-radius: 12px; padding: 24px; margin-bottom: 24px; }}
        .card-title {{ font-size: 18px; font-weight: 700; margin-bottom: 16px; color: #38bdf8; border-bottom: 1px solid #1f2937; padding-bottom: 8px; }}
        
        .grid-2 {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }}
        .grid-3 {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; }}
        .grid-6 {{ display: grid; grid-template-columns: repeat(6, 1fr); gap: 8px; }}
        
        .form-group {{ margin-bottom: 14px; }}
        label {{ display: block; font-size: 12px; font-weight: 600; color: #9ca3af; margin-bottom: 4px; text-transform: uppercase; }}
        input:not([type="checkbox"]), select, textarea {{ width: 100%; padding: 10px 12px; background: #1f2937; border: 1px solid #374151; border-radius: 6px; color: white; font-size: 14px; outline: none; }}
        input:not([type="checkbox"]):focus, select:focus, textarea:focus {{ border-color: #38bdf8; }}
        textarea {{ resize: vertical; min-height: 80px; font-family: monospace; }}

        input[type="checkbox"] {{
            width: 16px !important;
            height: 16px !important;
            min-width: 16px !important;
            min-height: 16px !important;
            margin: 0 !important;
            padding: 0 !important;
            cursor: pointer;
            accent-color: #38bdf8;
            flex-shrink: 0 !important;
            display: inline-block;
            vertical-align: middle;
        }}

        .rule-check-item {{
            display: flex !important;
            align-items: center !important;
            gap: 10px !important;
            cursor: pointer !important;
            text-transform: none !important;
            color: #e2e8f0 !important;
            font-size: 13px !important;
            font-weight: 500 !important;
            margin-bottom: 0 !important;
            user-select: none;
            padding: 8px 12px;
            border-radius: 6px;
            background: #182234;
            border: 1px solid #283548;
            transition: all 0.15s ease;
        }}
        .rule-check-item:hover {{
            background: #1e2d42;
            border-color: #38bdf8;
        }}
        .rule-check-item span {{
            flex: 1;
            line-height: 1.3;
        }}

        .type-pill {{
            display: inline-flex !important;
            align-items: center !important;
            gap: 8px !important;
            cursor: pointer !important;
            text-transform: none !important;
            color: #f1f5f9 !important;
            font-size: 13px !important;
            font-weight: 500 !important;
            margin-bottom: 0 !important;
            padding: 6px 10px;
            border-radius: 6px;
            background: #182234;
            border: 1px solid #283548;
            user-select: none;
            transition: all 0.15s ease;
        }}
        .type-pill:hover {{
            background: #1e2d42;
            border-color: #38bdf8;
        }}
        .type-pill span {{
            line-height: 1;
        }}

        .pkmn-tabs {{ display: flex; gap: 8px; border-bottom: 1px solid #374151; margin-bottom: 16px; overflow-x: auto; }}
        .pkmn-tab-btn {{ background: transparent; border: none; padding: 8px 16px; color: #9ca3af; font-size: 14px; font-weight: 600; cursor: pointer; border-radius: 6px 6px 0 0; }}
        .pkmn-tab-btn.active {{ color: #38bdf8; background: #1f2937; border-bottom: 2px solid #38bdf8; }}

        .sub-box {{ background: #182234; border: 1px solid #283548; border-radius: 8px; padding: 16px; margin-bottom: 14px; }}
        .sub-box h4 {{ font-size: 14px; margin-bottom: 10px; color: #60a5fa; display: flex; justify-content: space-between; align-items: center; }}
        .preset-btn {{ background: #374151; color: #d1d5db; border: none; padding: 4px 8px; border-radius: 4px; font-size: 11px; cursor: pointer; }}
        .preset-btn:hover {{ background: #4b5563; color: white; }}
        .counter {{ font-size: 12px; font-weight: bold; color: #94a3b8; }}
    </style>
</head>
<body>
    {}
    <div class="editor-subnav">
        <div class="brand">Edit Gym: {} - {}</div>
        <div class="header-actions">
            <a href="/admin/gyms" class="btn btn-back">Back to Gyms</a>
            <button class="btn btn-reset" onclick="resetDefaults()">Reset Canonical Anime Team</button>
            <button class="btn btn-save" onclick="saveGym()">Save Changes</button>
        </div>
    </div>

    <main>
        <div class="card">
            <div class="card-title">Gym Basic Information</div>
            <div class="grid-2">
                <div class="form-group">
                    <label>Leader Name</label>
                    <input type="text" id="leader_name" value="{}">
                </div>
                <div class="form-group">
                    <label>Gym Name</label>
                    <input type="text" id="gym_name" value="{}">
                </div>
                <div class="form-group">
                    <label>City</label>
                    <input type="text" id="city" value="{}">
                </div>
                <div class="form-group">
                    <label>Gym Type</label>
                    <input type="text" id="poke_type" value="{}">
                </div>
                <div class="form-group">
                    <label>Challenge Fee (PokéDollars)</label>
                    <input type="number" id="fee" value="{}">
                </div>
                <div class="form-group">
                    <label>Badge Name</label>
                    <input type="text" id="badge" value="{}">
                </div>
                <div class="form-group">
                    <label>Signature Pokémon ID</label>
                    <input type="number" id="signature_pokemon_id" value="{}">
                </div>
            </div>
            <div class="form-group">
                <label>Leader Quote</label>
                <input type="text" id="quote" value="{}">
            </div>
            <div class="grid-2">
                <div class="form-group">
                    <label>Gym Status</label>
                    <select id="is_open">
                        <option value="1" {}>OPEN</option>
                        <option value="0" {}>CLOSED</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Closed Reason Notice</label>
                    <input type="text" id="closed_reason" value="{}">
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-title">Enforceable Gym Battle Rules</div>
            <div style="font-size: 13px; color: #9ca3af; margin-bottom: 14px;">
                Configure battle restrictions enforced by the bot. Players who violate these rules cannot battle this Gym.
            </div>
            
            <div style="margin-bottom: 16px;">
                <label style="margin-bottom: 8px;">Banned Pokémon Types (Team members cannot have any of these types):</label>
                <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 8px; background: #182234; padding: 12px; border-radius: 8px; border: 1px solid #283548;">
                    {}
                </div>
            </div>

            <div class="grid-2" style="margin-bottom: 16px;">
                <div style="background: #182234; padding: 16px; border-radius: 8px; border: 1px solid #283548; display: flex; flex-direction: column; gap: 8px;">
                    <div style="font-size: 12px; font-weight: 700; color: #38bdf8; text-transform: uppercase; margin-bottom: 4px;">Special Category Bans</div>
                    <label class="rule-check-item">
                        <input type="checkbox" id="rule_ban_legendaries" {}>
                        <span>Ban Legendary Pokémon</span>
                    </label>
                    <label class="rule-check-item">
                        <input type="checkbox" id="rule_ban_mythicals" {}>
                        <span>Ban Mythical Pokémon</span>
                    </label>
                    <label class="rule-check-item">
                        <input type="checkbox" id="rule_ban_ultra_beasts" {}>
                        <span>Ban Ultra Beasts</span>
                    </label>
                    <label class="rule-check-item">
                        <input type="checkbox" id="rule_ban_paradox" {}>
                        <span>Ban Paradox Pokémon</span>
                    </label>
                </div>

                <div style="background: #182234; padding: 16px; border-radius: 8px; border: 1px solid #283548; display: flex; flex-direction: column; gap: 8px;">
                    <div style="font-size: 12px; font-weight: 700; color: #38bdf8; text-transform: uppercase; margin-bottom: 4px;">Team & Battle Constraints</div>
                    <label class="rule-check-item">
                        <input type="checkbox" id="rule_no_duplicates" {}>
                        <span>No Duplicate Pokémon (Species clause)</span>
                    </label>
                    <label class="rule-check-item">
                        <input type="checkbox" id="rule_require_conscious" {}>
                        <span>Require Conscious (Must have conscious Pokémon)</span>
                    </label>
                    <div class="form-group" style="margin-top: 6px; margin-bottom: 0;">
                        <label style="margin-bottom: 4px;">Max Pokémon Level Cap (Blank for no limit):</label>
                        <input type="number" id="rule_max_level" min="1" max="100" placeholder="e.g. 50 or 100" value="{}">
                    </div>
                </div>
            </div>

            <button type="button" class="btn" style="background: #374151; color: #38bdf8; border: 1px solid #4b5563; font-size: 12px; padding: 8px 14px;" onclick="syncRulesToText()">Auto-generate Rules Notice from Settings</button>
        </div>

        <div class="card">
            <div class="card-title">Enforceable Gym Rewards Configuration</div>
            <div style="font-size: 13px; color: #9ca3af; margin-bottom: 14px;">
                Configure rewards automatically deposited into the player&#39;s account upon defeating this Gym Leader.
            </div>

            <div class="grid-2">
                <div class="form-group">
                    <label>PokéDollars Reward</label>
                    <input type="number" id="reward_money" value="{}" placeholder="e.g. 1000">
                </div>
                <div class="form-group">
                    <label>Mega Stone Reward (Deposited into Key Items)</label>
                    <select id="reward_mega_stone">
                        {}
                    </select>
                </div>
                <div class="form-group">
                    <label>Technical Machine (TM) Reward</label>
                    <select id="reward_tm">
                        {}
                    </select>
                </div>
                <div class="form-group">
                    <label>Pokémon Egg Reward (Deposited into Eggs)</label>
                    <select id="reward_egg">
                        {}
                    </select>
                </div>
            </div>

            <button type="button" class="btn" style="background: #374151; color: #38bdf8; border: 1px solid #4b5563; font-size: 12px; padding: 8px 14px; margin-top: 6px;" onclick="syncRewardsToText()">Auto-generate Rewards Notice from Settings</button>
        </div>

        <div class="card">
            <div class="card-title">Rules & Rewards Details</div>
            <div class="grid-2">
                <div class="form-group">
                    <label>Custom Rules Text (Displayed on Telegram Rules Alert)</label>
                    <textarea id="rules_text">{}</textarea>
                </div>
                <div class="form-group">
                    <label>Custom Rewards Text (Displayed on Telegram Rewards Alert)</label>
                    <textarea id="rewards_text">{}</textarea>
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-title">Gym Leader Pokémon Team (1-6 Pokémon)</div>
            <div class="pkmn-tabs">
                <button class="pkmn-tab-btn active" onclick="switchPkmnTab(0)" id="tab-btn-0">Slot 1</button>
                <button class="pkmn-tab-btn" onclick="switchPkmnTab(1)" id="tab-btn-1">Slot 2</button>
                <button class="pkmn-tab-btn" onclick="switchPkmnTab(2)" id="tab-btn-2">Slot 3</button>
                <button class="pkmn-tab-btn" onclick="switchPkmnTab(3)" id="tab-btn-3">Slot 4</button>
                <button class="pkmn-tab-btn" onclick="switchPkmnTab(4)" id="tab-btn-4">Slot 5</button>
                <button class="pkmn-tab-btn" onclick="switchPkmnTab(5)" id="tab-btn-5">Slot 6</button>
            </div>

            <div id="pkmn-editor-container"></div>
        </div>
    </main>

    <script>
        const REGION = "{}";
        const GYM_NUM = {};
        const POKEMON_OPTIONS_HTML = `{}`;
        let teamData = {};

        function initEditor() {{
            while (teamData.length < 6) {{
                teamData.push({{
                    pokemon_id: 1,
                    name: "Bulbasaur",
                    level: 100,
                    nature: "Adamant",
                    ivs: {{ hp: 31, attack: 31, defense: 31, sp_attack: 31, sp_defense: 31, speed: 31 }},
                    evs: {{ hp: 6, attack: 252, defense: 0, sp_attack: 0, sp_defense: 0, speed: 252 }},
                    moves: []
                }});
            }}
            renderSlot(0);

            // Real-time synchronization: when settings change, update Rules and Rewards text
            document.querySelectorAll('input[name="banned_type"]').forEach(cb => cb.addEventListener('change', syncRulesToText));
            ['rule_ban_legendaries', 'rule_ban_mythicals', 'rule_ban_ultra_beasts', 'rule_ban_paradox', 'rule_no_duplicates', 'rule_require_conscious'].forEach(id => {{
                const el = document.getElementById(id);
                if (el) el.addEventListener('change', syncRulesToText);
            }});
            ['rule_max_level', 'leader_name'].forEach(id => {{
                const el = document.getElementById(id);
                if (el) el.addEventListener('input', syncRulesToText);
            }});

            ['reward_mega_stone', 'reward_tm', 'reward_egg'].forEach(id => {{
                const el = document.getElementById(id);
                if (el) el.addEventListener('change', syncRewardsToText);
            }});
            ['reward_money', 'badge'].forEach(id => {{
                const el = document.getElementById(id);
                if (el) el.addEventListener('input', syncRewardsToText);
            }});
        }}

        function switchPkmnTab(idx) {{
            document.querySelectorAll('.pkmn-tab-btn').forEach(b => b.classList.remove('active'));
            document.getElementById('tab-btn-' + idx).classList.add('active');
            renderSlot(idx);
        }}

        function renderSlot(idx) {{
            const p = teamData[idx];
            const ivs = p.ivs || {{ hp: 31, attack: 31, defense: 31, sp_attack: 31, sp_defense: 31, speed: 31 }};
            const evs = p.evs || {{ hp: 0, attack: 0, defense: 0, sp_attack: 0, sp_defense: 0, speed: 0 }};
            const moves = p.moves || [];

            let movesHtml = '';
            for (let m = 0; m < 4; m++) {{
                const mv = moves[m] || {{ name: '', type: 'Normal', category: 'Physical', power: 80, accuracy: 100, pp: 15 }};
                movesHtml += `
                    <div style="background: #111827; border: 1px solid #1f2937; border-radius: 6px; padding: 10px; margin-bottom: 8px;">
                        <div style="font-size: 11px; font-weight: 700; color: #9ca3af; margin-bottom: 4px;">MOVE ${{m + 1}}</div>
                        <div style="display: grid; grid-template-columns: 2fr 1fr 1fr 1fr 1fr 1fr; gap: 8px;">
                            <input type="text" placeholder="Move Name" value="${{mv.name || ''}}" id="m_name_${{m}}" onchange="updateMove(${{idx}}, ${{m}})">
                            <input type="text" placeholder="Type" value="${{mv.type || 'Normal'}}" id="m_type_${{m}}" onchange="updateMove(${{idx}}, ${{m}})">
                            <select id="m_cat_${{m}}" onchange="updateMove(${{idx}}, ${{m}})">
                                <option value="Physical" ${{mv.category === 'Physical' ? 'selected' : ''}}>Physical</option>
                                <option value="Special" ${{mv.category === 'Special' ? 'selected' : ''}}>Special</option>
                                <option value="Status" ${{mv.category === 'Status' ? 'selected' : ''}}>Status</option>
                            </select>
                            <input type="number" placeholder="Power" value="${{mv.power !== undefined && mv.power !== null ? mv.power : 80}}" id="m_pow_${{m}}" onchange="updateMove(${{idx}}, ${{m}})">
                            <input type="number" placeholder="Acc" value="${{mv.accuracy !== undefined && mv.accuracy !== null ? mv.accuracy : 100}}" id="m_acc_${{m}}" onchange="updateMove(${{idx}}, ${{m}})">
                            <input type="number" placeholder="PP" value="${{mv.pp !== undefined && mv.pp !== null ? mv.pp : 15}}" id="m_pp_${{m}}" onchange="updateMove(${{idx}}, ${{m}})">
                        </div>
                    </div>
                `;
            }}

            const html = `
                <div class="grid-2">
                    <div class="form-group">
                        <label>Species</label>
                        <select id="slot_species" onchange="changeSpecies(${{idx}})">
                            ${{POKEMON_OPTIONS_HTML}}
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Nickname / Custom Name</label>
                        <input type="text" id="slot_nickname" value="${{p.name || ''}}" onchange="teamData[${{idx}}].name = this.value">
                    </div>
                    <div class="form-group">
                        <label>Level</label>
                        <input type="number" min="1" max="100" id="slot_level" value="${{p.level || 100}}" onchange="teamData[${{idx}}].level = parseInt(this.value) || 100">
                    </div>
                    <div class="form-group">
                        <label>Nature</label>
                        <input type="text" id="slot_nature" value="${{p.nature || 'Adamant'}}" onchange="teamData[${{idx}}].nature = this.value">
                    </div>
                </div>

                <div class="sub-box">
                    <h4>Individual Values (IVs: 0 - 31) <button class="preset-btn" onclick="maxIVs(${{idx}})">Max 31 All</button></h4>
                    <div class="grid-6">
                        <div><label>HP</label><input type="number" min="0" max="31" value="${{ivs.hp || 31}}" onchange="updateIV(${{idx}}, 'hp', this.value)"></div>
                        <div><label>Atk</label><input type="number" min="0" max="31" value="${{ivs.attack || 31}}" onchange="updateIV(${{idx}}, 'attack', this.value)"></div>
                        <div><label>Def</label><input type="number" min="0" max="31" value="${{ivs.defense || 31}}" onchange="updateIV(${{idx}}, 'defense', this.value)"></div>
                        <div><label>SpA</label><input type="number" min="0" max="31" value="${{ivs.sp_attack || 31}}" onchange="updateIV(${{idx}}, 'sp_attack', this.value)"></div>
                        <div><label>SpD</label><input type="number" min="0" max="31" value="${{ivs.sp_defense || 31}}" onchange="updateIV(${{idx}}, 'sp_defense', this.value)"></div>
                        <div><label>Spe</label><input type="number" min="0" max="31" value="${{ivs.speed || 31}}" onchange="updateIV(${{idx}}, 'speed', this.value)"></div>
                    </div>
                </div>

                <div class="sub-box">
                    <h4>Effort Values (EVs: 0 - 252, Max Total: 510) <span id="ev-counter" class="counter">Total: ${{calcEVTotal(evs)}} / 510</span></h4>
                    <div style="display: flex; gap: 8px; margin-bottom: 10px;">
                        <button class="preset-btn" onclick="setEVPreset(${{idx}}, 'atk_spe')">252 Atk / 252 Spe / 6 HP</button>
                        <button class="preset-btn" onclick="setEVPreset(${{idx}}, 'spa_spe')">252 SpA / 252 Spe / 6 HP</button>
                        <button class="preset-btn" onclick="setEVPreset(${{idx}}, 'bulky')">252 HP / 252 Def / 6 SpD</button>
                        <button class="preset-btn" onclick="setEVPreset(${{idx}}, 'reset')">Reset 0</button>
                    </div>
                    <div class="grid-6">
                        <div><label>HP</label><input type="number" min="0" max="252" value="${{evs.hp || 0}}" onchange="updateEV(${{idx}}, 'hp', this.value)"></div>
                        <div><label>Atk</label><input type="number" min="0" max="252" value="${{evs.attack || 0}}" onchange="updateEV(${{idx}}, 'attack', this.value)"></div>
                        <div><label>Def</label><input type="number" min="0" max="252" value="${{evs.defense || 0}}" onchange="updateEV(${{idx}}, 'defense', this.value)"></div>
                        <div><label>SpA</label><input type="number" min="0" max="252" value="${{evs.sp_attack || 0}}" onchange="updateEV(${{idx}}, 'sp_attack', this.value)"></div>
                        <div><label>SpD</label><input type="number" min="0" max="252" value="${{evs.sp_defense || 0}}" onchange="updateEV(${{idx}}, 'sp_defense', this.value)"></div>
                        <div><label>Spe</label><input type="number" min="0" max="252" value="${{evs.speed || 0}}" onchange="updateEV(${{idx}}, 'speed', this.value)"></div>
                    </div>
                </div>

                <div class="sub-box">
                    <h4>Move Set (Up to 4 Moves) <button class="preset-btn" onclick="autofillMoves(${{idx}})">Auto-fill Level 100 Moves</button></h4>
                    ${{movesHtml}}
                </div>
            `;

            document.getElementById('pkmn-editor-container').innerHTML = html;
            document.getElementById('slot_species').value = p.pokemon_id;
        }}

        function calcEVTotal(evs) {{
            return (evs.hp || 0) + (evs.attack || 0) + (evs.defense || 0) + (evs.sp_attack || 0) + (evs.sp_defense || 0) + (evs.speed || 0);
        }}

        function changeSpecies(idx) {{
            const sel = document.getElementById('slot_species');
            const pid = parseInt(sel.value) || 1;
            teamData[idx].pokemon_id = pid;
            const text = sel.options[sel.selectedIndex].text;
            const match = text.match(/#\d+\s+(.*)/);
            if (match && (!teamData[idx].name || teamData[idx].name.startsWith('Pokemon #'))) {{
                teamData[idx].name = match[1];
                document.getElementById('slot_nickname').value = match[1];
            }}
            autofillMoves(idx);
        }}

        function updateIV(idx, stat, val) {{
            if (!teamData[idx].ivs) teamData[idx].ivs = {{ hp: 31, attack: 31, defense: 31, sp_attack: 31, sp_defense: 31, speed: 31 }};
            teamData[idx].ivs[stat] = Math.min(31, Math.max(0, parseInt(val) || 0));
        }}

        function maxIVs(idx) {{
            teamData[idx].ivs = {{ hp: 31, attack: 31, defense: 31, sp_attack: 31, sp_defense: 31, speed: 31 }};
            renderSlot(idx);
        }}

        function updateEV(idx, stat, val) {{
            if (!teamData[idx].evs) teamData[idx].evs = {{ hp: 0, attack: 0, defense: 0, sp_attack: 0, sp_defense: 0, speed: 0 }};
            teamData[idx].evs[stat] = Math.min(252, Math.max(0, parseInt(val) || 0));
            const total = calcEVTotal(teamData[idx].evs);
            const counter = document.getElementById('ev-counter');
            if (counter) {{
                counter.innerText = `Total: ${{total}} / 510`;
                counter.style.color = total > 510 ? '#f87171' : '#94a3b8';
            }}
        }}

        function setEVPreset(idx, type) {{
            if (type === 'atk_spe') teamData[idx].evs = {{ hp: 6, attack: 252, defense: 0, sp_attack: 0, sp_defense: 0, speed: 252 }};
            else if (type === 'spa_spe') teamData[idx].evs = {{ hp: 6, attack: 0, defense: 0, sp_attack: 252, sp_defense: 0, speed: 252 }};
            else if (type === 'bulky') teamData[idx].evs = {{ hp: 252, attack: 0, defense: 252, sp_attack: 0, sp_defense: 6, speed: 0 }};
            else teamData[idx].evs = {{ hp: 0, attack: 0, defense: 0, sp_attack: 0, sp_defense: 0, speed: 0 }};
            renderSlot(idx);
        }}

        function updateMove(pkmnIdx, mvIdx) {{
            if (!teamData[pkmnIdx].moves) teamData[pkmnIdx].moves = [];
            while (teamData[pkmnIdx].moves.length <= mvIdx) teamData[pkmnIdx].moves.push({{ name: '', type: 'Normal', category: 'Physical', power: 80, accuracy: 100, pp: 15 }});
            teamData[pkmnIdx].moves[mvIdx] = {{
                name: document.getElementById('m_name_' + mvIdx).value.trim(),
                type: document.getElementById('m_type_' + mvIdx).value.trim(),
                category: document.getElementById('m_cat_' + mvIdx).value,
                power: parseInt(document.getElementById('m_pow_' + mvIdx).value) || 0,
                accuracy: parseInt(document.getElementById('m_acc_' + mvIdx).value) || 100,
                pp: parseInt(document.getElementById('m_pp_' + mvIdx).value) || 15
            }};
        }}

        async function autofillMoves(idx) {{
            const pid = teamData[idx].pokemon_id || 1;
            try {{
                const res = await fetch('/api/pokemon_moves?id=' + pid);
                const moves = await res.json();
                if (Array.isArray(moves) && moves.length > 0) {{
                    teamData[idx].moves = moves.slice(-4);
                    renderSlot(idx);
                }}
            }} catch (err) {{
                console.error("Autofill moves error:", err);
            }}
        }}

        function syncRulesToText() {{
            const leaderName = document.getElementById('leader_name').value.trim() || 'Gym Leader';
            const bannedTypes = Array.from(document.querySelectorAll('input[name="banned_type"]:checked')).map(cb => cb.value);
            const banLeg = document.getElementById('rule_ban_legendaries').checked;
            const banMyth = document.getElementById('rule_ban_mythicals').checked;
            const banUb = document.getElementById('rule_ban_ultra_beasts').checked;
            const banParadox = document.getElementById('rule_ban_paradox').checked;
            const noDupes = document.getElementById('rule_no_duplicates').checked;
            const reqConscious = document.getElementById('rule_require_conscious').checked;
            const maxLvl = parseInt(document.getElementById('rule_max_level').value) || null;

            const bullets = [];
            if (bannedTypes.length > 0) {{
                bullets.push(`• No ${{bannedTypes.join(', ')}}-type Pokémon in your team!`);
            }}
            const specials = [];
            if (banMyth) specials.push('Mythical');
            if (banLeg) specials.push('Legendary');
            if (banUb) specials.push('Ultra Beast');
            if (banParadox) specials.push('Paradox');
            if (specials.length > 0) {{
                bullets.push(`• No ${{specials.join(', ')}} Pokémon allowed!`);
            }}
            if (noDupes) {{
                bullets.push('• No duplicate Pokémon allowed!');
            }}
            if (reqConscious) {{
                bullets.push('• Active battle team must have at least 1 conscious Pokémon!');
            }}
            if (maxLvl && maxLvl > 0) {{
                bullets.push(`• Maximum Pokémon level allowed: Lv. ${{maxLvl}}!`);
            }}
            bullets.push(`• 6v6 Pokémon Battle against ${{leaderName}}`);
            bullets.push('• Level 100 Anime Signature Pokémon Team');
            bullets.push('• Perfect IVs (186) & Max EVs (510)');
            bullets.push('• Defeat all 6 of the Gym Leader\'s Pokémon to earn the Gym Badge!');

            document.getElementById('rules_text').value = 'Gym Rules:\n' + bullets.join('\n');
        }}

        function syncRewardsToText() {{
            const badge = document.getElementById('badge').value.trim();
            const money = parseInt(document.getElementById('reward_money').value) || 0;
            const msSelect = document.getElementById('reward_mega_stone');
            const msVal = msSelect.value;
            const msText = msSelect.options[msSelect.selectedIndex]?.text || '';
            const tmSelect = document.getElementById('reward_tm');
            const tmVal = tmSelect.value;
            const tmText = tmSelect.options[tmSelect.selectedIndex]?.text || '';
            const eggSelect = document.getElementById('reward_egg');
            const eggVal = eggSelect.value;
            const eggText = eggSelect.options[eggSelect.selectedIndex]?.text || '';

            const bullets = [];
            if (badge) {{
                bullets.push(`• ${{badge}}`);
            }}
            if (money > 0) {{
                bullets.push(`• ${{money}} PokéDollars 💵`);
            }}
            if (msVal && msVal !== '') {{
                if (msVal === 'random') {{
                    bullets.push('• Random Mega Stone');
                }} else {{
                    bullets.push(`• ${{msText}} (Mega Stone)`);
                }}
            }}
            if (tmVal && tmVal !== '') {{
                if (tmVal === 'random') {{
                    bullets.push('• Random TM');
                }} else {{
                    bullets.push(`• ${{tmText}}`);
                }}
            }}
            if (eggVal && eggVal !== '') {{
                if (eggVal === 'random') {{
                    bullets.push('• Random Pokémon Egg');
                }} else {{
                    bullets.push(`• ${{eggText}}`);
                }}
            }}

            if (bullets.length === 0) {{
                document.getElementById('rewards_text').value = 'Gym Rewards:\n• No rewards configured';
            }} else {{
                document.getElementById('rewards_text').value = 'Gym Rewards:\n' + bullets.join('\n');
            }}
        }}

        async function saveGym() {{
            const bannedTypes = Array.from(document.querySelectorAll('input[name="banned_type"]:checked')).map(cb => cb.value);
            const rulesObj = {{
                banned_types: bannedTypes,
                ban_legendaries: document.getElementById('rule_ban_legendaries').checked,
                ban_mythicals: document.getElementById('rule_ban_mythicals').checked,
                ban_ultra_beasts: document.getElementById('rule_ban_ultra_beasts').checked,
                ban_paradox: document.getElementById('rule_ban_paradox').checked,
                no_duplicates: document.getElementById('rule_no_duplicates').checked,
                require_conscious: document.getElementById('rule_require_conscious').checked,
                max_level: parseInt(document.getElementById('rule_max_level').value) || null
            }};

            const rewardsObj = {{
                pokedollars: parseInt(document.getElementById('reward_money').value) || 0,
                mega_stone: document.getElementById('reward_mega_stone').value || null,
                tm: document.getElementById('reward_tm').value || null,
                egg: document.getElementById('reward_egg').value || null,
                badge: document.getElementById('badge').value.trim() || null
            }};

            const payload = {{
                region: REGION,
                gym_num: GYM_NUM,
                leader_name: document.getElementById('leader_name').value.trim(),
                gym_name: document.getElementById('gym_name').value.trim(),
                city: document.getElementById('city').value.trim(),
                poke_type: document.getElementById('poke_type').value.trim(),
                fee: parseInt(document.getElementById('fee').value) || 200,
                reward_money: parseInt(document.getElementById('reward_money').value) || 1000,
                badge: document.getElementById('badge').value.trim(),
                reward_badge: document.getElementById('badge').value.trim(),
                signature_pokemon_id: parseInt(document.getElementById('signature_pokemon_id').value) || 0,
                quote: document.getElementById('quote').value.trim(),
                is_open: document.getElementById('is_open').value === '1',
                closed_reason: document.getElementById('closed_reason').value.trim(),
                rules: rulesObj,
                rewards: rewardsObj,
                rules_text: document.getElementById('rules_text').value.trim(),
                rewards_text: document.getElementById('rewards_text').value.trim(),
                team: teamData.filter(p => p.pokemon_id > 0)
            }};

            try {{
                const res = await fetch('/api/gym/save', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(payload)
                }});
                const data = await res.json();
                if (data.success) {{
                    alert("Gym configuration saved successfully!");
                    location.href = "/admin/gyms";
                }} else {{
                    alert("Error saving gym: " + (data.error || 'Unknown'));
                }}
            }} catch (err) {{
                alert("Network error: " + err);
            }}
        }}

        async function resetDefaults() {{
            if (!confirm("Are you sure you want to reset this Gym Leader back to canonical anime defaults? All custom changes will be overwritten!")) return;
            try {{
                const res = await fetch('/api/gym/reset', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ region: REGION, gym_num: GYM_NUM }})
                }});
                const data = await res.json();
                if (data.success) {{
                    alert("Gym reset to anime defaults!");
                    location.reload();
                }} else {{
                    alert("Error resetting gym: " + (data.error || 'Unknown'));
                }}
            }} catch (err) {{
                alert("Network error: " + err);
            }}
        }}

        initEditor();
    </script>
</body>
</html>"#,
        html_escape(&gym_name),
        navbar_css,
        navbar,
        html_escape(&leader_name),
        html_escape(&gym_name),
        html_escape(&leader_name),
        html_escape(&gym_name),
        html_escape(&city),
        html_escape(&poke_type),
        fee,
        html_escape(&badge),
        sig_id,
        html_escape(&quote),
        if is_open { "selected" } else { "" },
        if !is_open { "selected" } else { "" },
        html_escape(&closed_reason),
        type_checkboxes,
        ban_leg_checked,
        ban_myth_checked,
        ban_ub_checked,
        ban_paradox_checked,
        no_dupes_checked,
        req_conscious_checked,
        max_lvl_val,
        reward_money,
        mega_stone_options,
        tm_options,
        egg_options,
        html_escape(&effective_rules_text),
        html_escape(&effective_rewards_text),
        region,
        gym_num,
        pokemon_options,
        team_json
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

async fn handle_api_redeem_list(state: &AdminState) -> serde_json::Value {
    let rows = sqlx::query(
        "SELECT id, code, description, rewards_json, max_uses, current_uses, max_per_user, expires_at, is_active, created_at
         FROM redeem_codes ORDER BY id DESC"
    )
    .fetch_all(&*state.pool)
    .await
    .unwrap_or_default();

    let mut list = Vec::new();
    for r in rows {
        let id: i64 = r.get("id");
        let code: String = r.get("code");
        let description: String = r.get("description");
        let rewards_json: String = r.get("rewards_json");
        let max_uses: Option<i64> = r.get("max_uses");
        let current_uses: i64 = r.get("current_uses");
        let max_per_user: i64 = r.get("max_per_user");
        let expires_at: Option<String> = r.get("expires_at");
        let is_active: i64 = r.get("is_active");
        let created_at: String = r.get("created_at");

        let rewards_cfg: RedeemRewardsConfig = serde_json::from_str(&rewards_json).unwrap_or_default();
        let summary = rewards_cfg.to_summary_string(&state.pokedex);

        list.push(json!({
            "id": id,
            "code": code,
            "description": description,
            "rewards_json": rewards_json,
            "rewards_summary": summary,
            "max_uses": max_uses,
            "current_uses": current_uses,
            "max_per_user": max_per_user,
            "expires_at": expires_at,
            "is_active": is_active == 1,
            "created_at": created_at
        }));
    }

    json!({"success": true, "codes": list})
}

async fn handle_api_redeem_create(state: &AdminState, body: &[u8]) -> serde_json::Value {
    let payload: serde_json::Value = serde_json::from_slice(body).unwrap_or_else(|_| json!({}));
    let code = payload.get("code").and_then(|c| c.as_str()).unwrap_or("").trim().to_uppercase();
    if code.is_empty() {
        return json!({"success": false, "error": "Redeem code cannot be empty"});
    }

    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM redeem_codes WHERE UPPER(code) = UPPER(?)")
        .bind(&code)
        .fetch_one(&*state.pool)
        .await
        .unwrap_or(0);

    if exists > 0 {
        return json!({"success": false, "error": "A redeem code with this name already exists"});
    }

    let description = payload.get("description").and_then(|d| d.as_str()).unwrap_or("").trim();
    let max_uses = payload.get("max_uses").and_then(|m| m.as_i64()).filter(|&m| m > 0);
    let max_per_user = payload.get("max_per_user").and_then(|m| m.as_i64()).unwrap_or(1).max(1);
    let expires_at = payload.get("expires_at").and_then(|e| e.as_str()).map(|s| s.trim()).filter(|s| !s.is_empty());

    let rewards_val = payload.get("rewards").cloned().unwrap_or_else(|| json!({}));
    let rewards_json = serde_json::to_string(&rewards_val).unwrap_or_else(|_| "{}".to_string());

    let res = sqlx::query(
        "INSERT INTO redeem_codes (code, description, rewards_json, max_uses, current_uses, max_per_user, expires_at, is_active)
         VALUES (?, ?, ?, ?, 0, ?, ?, 1)"
    )
    .bind(&code)
    .bind(description)
    .bind(&rewards_json)
    .bind(max_uses)
    .bind(max_per_user)
    .bind(expires_at)
    .execute(&*state.pool)
    .await;

    match res {
        Ok(r) => json!({"success": true, "id": r.last_insert_rowid()}),
        Err(e) => json!({"success": false, "error": e.to_string()}),
    }
}

async fn handle_api_redeem_toggle(state: &AdminState, body: &[u8]) -> serde_json::Value {
    let payload: serde_json::Value = serde_json::from_slice(body).unwrap_or_else(|_| json!({}));
    let id = payload.get("id").and_then(|i| i.as_i64());
    let Some(id) = id else {
        return json!({"success": false, "error": "Missing redeem code id"});
    };

    let cur: Option<i64> = sqlx::query_scalar("SELECT is_active FROM redeem_codes WHERE id = ?")
        .bind(id)
        .fetch_optional(&*state.pool)
        .await
        .unwrap_or(None);

    let Some(curr) = cur else {
        return json!({"success": false, "error": "Redeem code not found"});
    };

    let new_val = if curr == 1 { 0 } else { 1 };
    let _ = sqlx::query("UPDATE redeem_codes SET is_active = ? WHERE id = ?")
        .bind(new_val)
        .bind(id)
        .execute(&*state.pool)
        .await;

    json!({"success": true, "is_active": new_val == 1})
}

async fn handle_api_redeem_delete(state: &AdminState, body: &[u8]) -> serde_json::Value {
    let payload: serde_json::Value = serde_json::from_slice(body).unwrap_or_else(|_| json!({}));
    let id = payload.get("id").and_then(|i| i.as_i64());
    let Some(id) = id else {
        return json!({"success": false, "error": "Missing redeem code id"});
    };

    let _ = sqlx::query("DELETE FROM user_redeemed_codes WHERE code_id = ?")
        .bind(id)
        .execute(&*state.pool)
        .await;

    let res = sqlx::query("DELETE FROM redeem_codes WHERE id = ?")
        .bind(id)
        .execute(&*state.pool)
        .await;

    match res {
        Ok(_) => json!({"success": true}),
        Err(e) => json!({"success": false, "error": e.to_string()}),
    }
}

async fn render_redeem_html(state: &AdminState) -> String {
    let navbar = render_navbar("redeem");
    let navbar_css = get_navbar_css();

    let mut tm_options = String::from("<option value=\"none\">None</option><option value=\"random\">Random TM (Gen 1-9)</option><optgroup label=\"Specific TMs\">");
    for tm in TM_CATALOG {
        tm_options.push_str(&format!(
            "<option value=\"{}\">TM #{} - {} ({})</option>",
            tm.number, tm.number, html_escape(&tm.name), html_escape(&tm.move_type)
        ));
    }
    tm_options.push_str("</optgroup>");

    let mut tutor_options = String::from("<option value=\"none\">None</option><option value=\"random\">Random Move Tutor</option><optgroup label=\"Specific Move Tutors\">");
    for tut in TUTOR_CATALOG {
        tutor_options.push_str(&format!(
            "<option value=\"{}\">Tutor #{} - {} ({})</option>",
            tut.number, tut.number, html_escape(&tut.name), html_escape(&tut.move_type)
        ));
    }
    tutor_options.push_str("</optgroup>");

    let mut mega_stone_options = String::from("<option value=\"none\">None</option><option value=\"random\">Random Mega Stone</option><optgroup label=\"Specific Mega Stones\">");
    for &stone in ALL_MEGA_STONES {
        mega_stone_options.push_str(&format!(
            "<option value=\"{}\">{}</option>",
            html_escape(stone), html_escape(stone)
        ));
    }
    mega_stone_options.push_str("</optgroup>");

    let mut sorted_pokemon: Vec<_> = state.pokedex.pokemon.values().filter(|p| p.id <= 1025).collect();
    sorted_pokemon.sort_by_key(|p| p.id);

    let mut egg_options = String::from("<option value=\"none\">None</option><option value=\"random\">Random Pokémon Egg</option><optgroup label=\"Specific Species Eggs\">");
    for p in &sorted_pokemon {
        let name = format_pokemon_name(&p.name);
        egg_options.push_str(&format!(
            "<option value=\"{}\">#{} {}</option>",
            p.id, p.id, html_escape(&name)
        ));
    }
    egg_options.push_str("</optgroup>");

    let mut pkmn_reward_options = String::from("<option value=\"\">-- Choose Pokémon Species --</option>");
    for p in &sorted_pokemon {
        let name = format_pokemon_name(&p.name);
        pkmn_reward_options.push_str(&format!(
            "<option value=\"{}\">#{} {}</option>",
            p.id, p.id, html_escape(&name)
        ));
    }

    let rows = sqlx::query(
        "SELECT id, code, description, rewards_json, max_uses, current_uses, max_per_user, expires_at, is_active, created_at
         FROM redeem_codes ORDER BY id DESC"
    )
    .fetch_all(&*state.pool)
    .await
    .unwrap_or_default();

    let mut code_rows_html = String::new();
    if rows.is_empty() {
        code_rows_html.push_str(r#"<tr><td colspan="8" class="empty-state">No redeem codes created yet. Use the form above to generate one.</td></tr>"#);
    } else {
        for r in &rows {
            let id: i64 = r.get("id");
            let code: String = r.get("code");
            let description: String = r.get("description");
            let rewards_json: String = r.get("rewards_json");
            let max_uses: Option<i64> = r.get("max_uses");
            let current_uses: i64 = r.get("current_uses");
            let max_per_user: i64 = r.get("max_per_user");
            let expires_at: Option<String> = r.get("expires_at");
            let is_active: i64 = r.get("is_active");

            let rewards_cfg: RedeemRewardsConfig = serde_json::from_str(&rewards_json).unwrap_or_default();
            let summary = rewards_cfg.to_summary_string(&state.pokedex);

            let uses_str = match max_uses {
                Some(max) => format!("{}/{}", current_uses, max),
                None => format!("{}/Unlimited", current_uses),
            };

            let expiry_str = expires_at.as_deref().filter(|s| !s.is_empty()).unwrap_or("Never");

            let (status_badge, toggle_text, toggle_class) = if is_active == 0 {
                (r#"<span class="badge-status badge-inactive">Inactive</span>"#, "Activate", "btn-activate")
            } else {
                (r#"<span class="badge-status badge-active">Active</span>"#, "Deactivate", "btn-deactivate")
            };

            code_rows_html.push_str(&format!(
                r#"<tr id="row-{id}">
                    <td><code class="code-badge">{code}</code></td>
                    <td>{desc}</td>
                    <td><div class="reward-pill-container">{rewards}</div></td>
                    <td>{uses}</td>
                    <td>{limit} / trainer</td>
                    <td>{expiry}</td>
                    <td id="status-{id}">{status_badge}</td>
                    <td>
                        <div class="btn-group">
                            <button onclick="toggleCode({id})" id="toggle-btn-{id}" class="btn-sm {toggle_class}">{toggle_text}</button>
                            <button onclick="deleteCode({id})" class="btn-sm btn-delete">Delete</button>
                        </div>
                    </td>
                </tr>"#,
                id = id,
                code = html_escape(&code),
                desc = if description.is_empty() { "-" } else { &description },
                rewards = html_escape(&summary),
                uses = uses_str,
                limit = max_per_user,
                expiry = html_escape(expiry_str),
                status_badge = status_badge,
                toggle_text = toggle_text,
                toggle_class = toggle_class
            ));
        }
    }

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Redeem Codes - Admin Dashboard</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
        body {{ background: #090d16; color: #e2e8f0; min-height: 100vh; display: flex; flex-direction: column; }}
        {}
        main {{ max-width: 1300px; margin: 0 auto; padding: 28px 24px; width: 100%; flex: 1; }}

        .page-header {{ margin-bottom: 24px; }}
        .page-title {{ font-size: 26px; font-weight: 700; color: #f8fafc; letter-spacing: -0.02em; margin-bottom: 6px; }}
        .page-desc {{ font-size: 14px; color: #94a3b8; }}

        .card {{ background: #111827; border: 1px solid #1e293b; border-radius: 12px; padding: 24px; margin-bottom: 28px; }}
        .card-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; border-bottom: 1px solid #1e293b; padding-bottom: 12px; }}
        .card-title {{ font-size: 17px; font-weight: 700; color: #38bdf8; display: flex; align-items: center; gap: 8px; }}

        .form-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 16px; margin-bottom: 20px; }}
        .form-group {{ display: flex; flex-direction: column; gap: 6px; }}
        .form-group label {{ font-size: 12px; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; }}
        .form-input, .form-select {{ background: #1a2234; border: 1px solid #334155; border-radius: 6px; padding: 9px 12px; font-size: 14px; color: #f8fafc; outline: none; width: 100%; }}
        .form-input:focus, .form-select:focus {{ border-color: #38bdf8; ring: 2px solid rgba(56, 189, 248, 0.2); }}

        .input-with-button {{ display: flex; gap: 8px; }}
        .btn-gen {{ background: #1e293b; border: 1px solid #334155; color: #38bdf8; padding: 0 12px; border-radius: 6px; font-size: 13px; font-weight: 600; cursor: pointer; white-space: nowrap; }}
        .btn-gen:hover {{ background: #334155; }}

        .rewards-section-title {{ font-size: 14px; font-weight: 700; color: #e2e8f0; margin: 18px 0 12px; text-transform: uppercase; letter-spacing: 0.05em; }}
        .rewards-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; margin-bottom: 24px; }}
        .reward-card {{ background: #161f30; border: 1px solid #243046; border-radius: 8px; padding: 16px; display: flex; flex-direction: column; gap: 10px; }}
        .reward-card-title {{ font-size: 13px; font-weight: 600; color: #38bdf8; }}

        .checkbox-row {{ display: flex; align-items: center; gap: 8px; margin-top: 6px; }}
        .checkbox-row label {{ font-size: 13px; color: #cbd5e1; cursor: pointer; text-transform: none; }}

        .card-actions {{ display: flex; justify-content: flex-end; align-items: center; gap: 12px; margin-top: 10px; }}
        .btn-create {{ background: #0284c7; color: white; border: none; border-radius: 8px; padding: 11px 22px; font-size: 14px; font-weight: 600; cursor: pointer; transition: background 0.15s ease; }}
        .btn-create:hover {{ background: #0369a1; }}

        .status-alert {{ display: none; padding: 12px 16px; border-radius: 8px; font-size: 13px; margin-top: 14px; font-weight: 500; }}
        .status-alert.error {{ display: block; background: rgba(220, 38, 38, 0.15); border: 1px solid rgba(220, 38, 38, 0.3); color: #fca5a5; }}
        .status-alert.success {{ display: block; background: rgba(5, 150, 105, 0.15); border: 1px solid rgba(5, 150, 105, 0.3); color: #6ee7b7; }}

        .table-responsive {{ overflow-x: auto; width: 100%; }}
        .codes-table {{ width: 100%; border-collapse: collapse; text-align: left; font-size: 13px; }}
        .codes-table th {{ background: #0f172a; color: #94a3b8; font-weight: 600; padding: 12px 14px; border-bottom: 1px solid #1e293b; text-transform: uppercase; font-size: 11px; letter-spacing: 0.05em; }}
        .codes-table td {{ padding: 12px 14px; border-bottom: 1px solid #1e293b; color: #cbd5e1; vertical-align: middle; }}
        .codes-table tr:hover {{ background: #131d2e; }}

        .code-badge {{ background: #1e293b; color: #38bdf8; padding: 3px 8px; border-radius: 4px; font-family: monospace; font-size: 13px; font-weight: 700; border: 1px solid #334155; }}
        .reward-pill-container {{ max-width: 320px; font-size: 12px; line-height: 1.4; color: #94a3b8; }}
        .badge-status {{ display: inline-block; padding: 3px 8px; border-radius: 9999px; font-size: 11px; font-weight: 600; }}
        .badge-active {{ background: #065f46; color: #6ee7b7; }}
        .badge-inactive {{ background: #7f1d1d; color: #fca5a5; }}
        .badge-expired {{ background: #374151; color: #9ca3af; }}

        .btn-group {{ display: flex; gap: 6px; }}
        .btn-sm {{ padding: 5px 10px; border-radius: 5px; font-size: 11px; font-weight: 600; cursor: pointer; border: none; }}
        .btn-activate {{ background: #065f46; color: #6ee7b7; }}
        .btn-activate:hover {{ background: #047857; }}
        .btn-deactivate {{ background: #374151; color: #e2e8f0; }}
        .btn-deactivate:hover {{ background: #4b5563; }}
        .btn-delete {{ background: #7f1d1d; color: #fca5a5; }}
        .btn-delete:hover {{ background: #991b1b; }}

        .empty-state {{ text-align: center; color: #64748b; padding: 36px 0 !important; font-style: italic; }}
    </style>
</head>
<body>
    {}
    <main>
        <div class="page-header">
            <h1 class="page-title">Redeem Code Management</h1>
            <p class="page-desc">Create promotional reward codes with custom drops (Technical Machines, Move Tutors, Pokémon Eggs, Mega Stones, specific or random Pokémon, and PokéDollars).</p>
        </div>

        <div class="card">
            <div class="card-header">
                <span class="card-title">Create New Redeem Code</span>
            </div>

            <div class="form-grid">
                <div class="form-group">
                    <label for="code_input">Redeem Code *</label>
                    <div class="input-with-button">
                        <input type="text" id="code_input" class="form-input" placeholder="e.g. OZONE2026" maxlength="32">
                        <button type="button" class="btn-gen" onclick="generateCode()">Generate</button>
                    </div>
                </div>
                <div class="form-group">
                    <label for="desc_input">Description / Memo</label>
                    <input type="text" id="desc_input" class="form-input" placeholder="e.g. Discord Community Milestone">
                </div>
                <div class="form-group">
                    <label for="max_uses_input">Max Total Uses</label>
                    <input type="number" id="max_uses_input" class="form-input" placeholder="Leave empty or 0 for unlimited" min="0">
                </div>
                <div class="form-group">
                    <label for="max_per_user_input">Max Uses Per Trainer</label>
                    <input type="number" id="max_per_user_input" class="form-input" value="1" min="1">
                </div>
                <div class="form-group">
                    <label for="expires_at_input">Expiration Date (Optional)</label>
                    <input type="date" id="expires_at_input" class="form-input">
                </div>
            </div>

            <div class="rewards-section-title">Configure Reward Package</div>
            <div class="rewards-grid">
                <!-- 1. PokéDollars -->
                <div class="reward-card">
                    <span class="reward-card-title">PokéDollars (💵)</span>
                    <input type="number" id="reward_pokedollars" class="form-input" placeholder="0" min="0" step="500">
                </div>

                <!-- 2. Technical Machine -->
                <div class="reward-card">
                    <span class="reward-card-title">Technical Machine (TM)</span>
                    <select id="reward_tm" class="form-select">
                        {}
                    </select>
                </div>

                <!-- 3. Move Tutor -->
                <div class="reward-card">
                    <span class="reward-card-title">Move Tutor</span>
                    <select id="reward_tutor" class="form-select">
                        {}
                    </select>
                </div>

                <!-- 4. Pokémon Egg -->
                <div class="reward-card">
                    <span class="reward-card-title">Pokémon Egg</span>
                    <select id="reward_egg" class="form-select">
                        {}
                    </select>
                </div>

                <!-- 5. Mega Stone -->
                <div class="reward-card">
                    <span class="reward-card-title">Mega Stone</span>
                    <select id="reward_mega_stone" class="form-select">
                        {}
                    </select>
                </div>

                <!-- 6. Pokémon Reward -->
                <div class="reward-card">
                    <span class="reward-card-title">Pokémon Reward</span>
                    <select id="reward_pkmn_mode" class="form-select" onchange="togglePokemonInputs()">
                        <option value="none">None</option>
                        <option value="specific">Specific Pokémon</option>
                        <option value="random">Random Pokémon</option>
                    </select>

                    <div id="pkmn_specific_wrap" style="display:none; margin-top: 8px;">
                        <select id="reward_pkmn_specific" class="form-select">
                            {}
                        </select>
                    </div>

                    <div id="pkmn_random_wrap" style="display:none; margin-top: 8px;">
                        <select id="reward_pkmn_category" class="form-select">
                            <option value="all">All Pokémon Species</option>
                            <option value="non_legendary">Only Non-Legendary</option>
                            <option value="legendary">Only Legendary</option>
                            <option value="mythical">Only Mythical</option>
                            <option value="ultra_beast">Only Ultra Beasts</option>
                            <option value="paradox">Only Paradox</option>
                        </select>
                    </div>

                    <div id="pkmn_settings_wrap" style="display:none; margin-top: 8px;">
                        <div style="display: flex; gap: 8px; align-items: center;">
                            <div style="flex: 1;">
                                <label style="font-size: 11px; color: #94a3b8; display: block; margin-bottom: 2px;">Level (1-100)</label>
                                <input type="number" id="reward_pkmn_level" class="form-input" value="50" min="1" max="100">
                            </div>
                            <div style="flex: 1; padding-top: 14px;">
                                <div class="checkbox-row">
                                    <input type="checkbox" id="reward_pkmn_shiny">
                                    <label for="reward_pkmn_shiny">Shiny</label>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card-actions">
                <button type="button" class="btn-create" onclick="createCode()">Create Redeem Code</button>
            </div>
            <div id="status_msg" class="status-alert"></div>
        </div>

        <div class="card">
            <div class="card-header">
                <span class="card-title">Configured Redeem Codes</span>
            </div>
            <div class="table-responsive">
                <table class="codes-table">
                    <thead>
                        <tr>
                            <th>Code</th>
                            <th>Description</th>
                            <th>Rewards</th>
                            <th>Redemptions</th>
                            <th>Per Trainer</th>
                            <th>Expiration</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody id="codes_tbody">
                        {}
                    </tbody>
                </table>
            </div>
        </div>
    </main>

    <script>
        function generateCode() {{
            const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
            let res = 'OZONE-';
            for (let i = 0; i < 6; i++) {{
                res += chars.charAt(Math.floor(Math.random() * chars.length));
            }}
            document.getElementById('code_input').value = res;
        }}

        function togglePokemonInputs() {{
            const mode = document.getElementById('reward_pkmn_mode').value;
            const specWrap = document.getElementById('pkmn_specific_wrap');
            const randWrap = document.getElementById('pkmn_random_wrap');
            const setWrap = document.getElementById('pkmn_settings_wrap');

            if (mode === 'specific') {{
                specWrap.style.display = 'block';
                randWrap.style.display = 'none';
                setWrap.style.display = 'block';
            }} else if (mode === 'random') {{
                specWrap.style.display = 'none';
                randWrap.style.display = 'block';
                setWrap.style.display = 'block';
            }} else {{
                specWrap.style.display = 'none';
                randWrap.style.display = 'none';
                setWrap.style.display = 'none';
            }}
        }}

        function showMessage(msg, isError) {{
            const box = document.getElementById('status_msg');
            box.textContent = msg;
            box.className = 'status-alert ' + (isError ? 'error' : 'success');
            setTimeout(() => {{
                box.className = 'status-alert';
            }}, 5000);
        }}

        async function createCode() {{
            const code = document.getElementById('code_input').value.trim();
            if (!code) {{
                showMessage('Please enter or generate a redeem code.', true);
                return;
            }}

            const desc = document.getElementById('desc_input').value.trim();
            const maxUsesVal = document.getElementById('max_uses_input').value.trim();
            const maxUses = maxUsesVal ? parseInt(maxUsesVal, 10) : null;
            const maxPerUserVal = document.getElementById('max_per_user_input').value.trim();
            const maxPerUser = maxPerUserVal ? parseInt(maxPerUserVal, 10) : 1;
            const expiresAt = document.getElementById('expires_at_input').value || null;

            const pdVal = document.getElementById('reward_pokedollars').value.trim();
            const pokedollars = pdVal ? parseInt(pdVal, 10) : null;
            const tmVal = document.getElementById('reward_tm').value;
            const tm = (tmVal && tmVal !== 'none') ? tmVal : null;
            const tutorVal = document.getElementById('reward_tutor').value;
            const tutor = (tutorVal && tutorVal !== 'none') ? tutorVal : null;
            const eggVal = document.getElementById('reward_egg').value;
            const egg = (eggVal && eggVal !== 'none') ? eggVal : null;
            const msVal = document.getElementById('reward_mega_stone').value;
            const megaStone = (msVal && msVal !== 'none') ? msVal : null;

            const pkmnMode = document.getElementById('reward_pkmn_mode').value;
            let pkmnReward = null;
            if (pkmnMode === 'specific') {{
                const specId = parseInt(document.getElementById('reward_pkmn_specific').value, 10);
                if (!specId) {{
                    showMessage('Please select a specific Pokémon.', true);
                    return;
                }}
                const level = parseInt(document.getElementById('reward_pkmn_level').value, 10) || 50;
                const isShiny = document.getElementById('reward_pkmn_shiny').checked;
                pkmnReward = {{
                    mode: 'specific',
                    pokemon_id: specId,
                    level: level,
                    is_shiny: isShiny
                }};
            }} else if (pkmnMode === 'random') {{
                const cat = document.getElementById('reward_pkmn_category').value;
                const level = parseInt(document.getElementById('reward_pkmn_level').value, 10) || 50;
                const isShiny = document.getElementById('reward_pkmn_shiny').checked;
                pkmnReward = {{
                    mode: 'random',
                    category: cat,
                    level: level,
                    is_shiny: isShiny
                }};
            }}

            const payload = {{
                code: code,
                description: desc,
                max_uses: maxUses,
                max_per_user: maxPerUser,
                expires_at: expiresAt,
                rewards: {{
                    pokedollars: pokedollars,
                    tm: tm,
                    tutor: tutor,
                    egg: egg,
                    mega_stone: megaStone,
                    pokemon: pkmnReward
                }}
            }};

            try {{
                const res = await fetch('/api/redeem/create', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(payload)
                }});
                const data = await res.json();
                if (data.success) {{
                    showMessage('Redeem code ' + code.toUpperCase() + ' created successfully.', false);
                    setTimeout(() => window.location.reload(), 800);
                }} else {{
                    showMessage(data.error || 'Failed to create redeem code.', true);
                }}
            }} catch (e) {{
                showMessage('Network error while creating redeem code.', true);
            }}
        }}

        async function toggleCode(id) {{
            try {{
                const res = await fetch('/api/redeem/toggle', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ id: id }})
                }});
                const data = await res.json();
                if (data.success) {{
                    const statusTd = document.getElementById('status-' + id);
                    const btn = document.getElementById('toggle-btn-' + id);
                    if (data.is_active) {{
                        statusTd.innerHTML = '<span class="badge-status badge-active">Active</span>';
                        btn.textContent = 'Deactivate';
                        btn.className = 'btn-sm btn-deactivate';
                    }} else {{
                        statusTd.innerHTML = '<span class="badge-status badge-inactive">Inactive</span>';
                        btn.textContent = 'Activate';
                        btn.className = 'btn-sm btn-activate';
                    }}
                }} else {{
                    alert(data.error || 'Failed to toggle status.');
                }}
            }} catch (e) {{
                alert('Network error.');
            }}
        }}

        async function deleteCode(id) {{
            if (!confirm('Are you sure you want to delete this redeem code? This action cannot be undone.')) {{
                return;
            }}

            try {{
                const res = await fetch('/api/redeem/delete', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ id: id }})
                }});
                const data = await res.json();
                if (data.success) {{
                    const row = document.getElementById('row-' + id);
                    if (row) row.remove();
                }} else {{
                    alert(data.error || 'Failed to delete code.');
                }}
            }} catch (e) {{
                alert('Network error.');
            }}
        }}
    </script>
</body>
</html>"#,
        navbar_css,
        navbar,
        tm_options,
        tutor_options,
        egg_options,
        mega_stone_options,
        pkmn_reward_options,
        code_rows_html
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    async fn send_http_request(port: u16, req: &str) -> String {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.expect("Failed to connect to test server");
        stream.write_all(req.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
        let mut res = Vec::new();
        stream.read_to_end(&mut res).await.unwrap();
        String::from_utf8_lossy(&res).to_string()
    }

    #[tokio::test]
    async fn test_pure_rust_admin_web_panel_e2e() {
        let pool = Arc::new(SqlitePool::connect("sqlite::memory:").await.unwrap());

        sqlx::query(
            "CREATE TABLE gym_global_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                gyms_open INTEGER NOT NULL DEFAULT 1,
                closed_reason TEXT NOT NULL DEFAULT 'Gyms are temporarily closed by the Pokémon League.'
            );"
        )
        .execute(&*pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO gym_global_settings (id, gyms_open, closed_reason) VALUES (1, 1, 'Closed');")
            .execute(&*pool)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE gym_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                region TEXT NOT NULL,
                gym_num INTEGER NOT NULL,
                is_open INTEGER NOT NULL DEFAULT 1,
                closed_reason TEXT NOT NULL DEFAULT 'This Gym is currently closed.',
                leader_name TEXT NOT NULL,
                city TEXT NOT NULL,
                gym_name TEXT NOT NULL,
                poke_type TEXT NOT NULL,
                fee INTEGER NOT NULL DEFAULT 200,
                quote TEXT NOT NULL DEFAULT '',
                badge TEXT NOT NULL DEFAULT '',
                rules_text TEXT NOT NULL DEFAULT '',
                rewards_text TEXT NOT NULL DEFAULT '',
                reward_money INTEGER NOT NULL DEFAULT 1000,
                reward_badge TEXT NOT NULL DEFAULT '',
                signature_pokemon_id INTEGER NOT NULL DEFAULT 0,
                team_json TEXT NOT NULL DEFAULT '[]',
                rules_json TEXT NOT NULL DEFAULT '{}',
                rewards_json TEXT NOT NULL DEFAULT '{}',
                UNIQUE(region, gym_num)
            );"
        )
        .execute(&*pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO gym_configs (
                region, gym_num, is_open, closed_reason, leader_name, city, gym_name,
                poke_type, fee, quote, badge, rules_text, rewards_text, reward_money,
                reward_badge, signature_pokemon_id, team_json, rules_json, rewards_json
            ) VALUES (
                'kanto', 1, 1, 'Closed', 'Brock', 'Pewter City', 'Pewter City Gym',
                'Rock', 200, 'Rock willpower!', 'Boulder Badge', 'Rules', 'Rewards',
                1000, 'Boulder Badge', 95, '[]', '{}', '{}'
            );"
        )
        .execute(&*pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT,
                pokemons_caught INTEGER NOT NULL DEFAULT 0
            );"
        )
        .execute(&*pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS pokemons (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL
            );"
        )
        .execute(&*pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO users (id, username, pokemons_caught) VALUES (1, 'Ash', 25);")
            .execute(&*pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO pokemons (id, user_id) VALUES (1, 1);")
            .execute(&*pool)
            .await
            .unwrap();

        let pokedex = Arc::new(PokedexData::load_from_file("./pokedex.json").expect("Failed to load pokedex.json"));
        let port = 19854; // Test port
        let pool_clone = pool.clone();
        let pokedex_clone = pokedex.clone();

        tokio::spawn(async move {
            start_admin_server(pool_clone, pokedex_clone, "127.0.0.1", port, "test_admin_key_123".to_string()).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // 1. Test unauthenticated request to /admin -> redirects to /login
        let res = send_http_request(port, "GET /admin HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").await;
        assert!(res.contains("302 Found"));
        assert!(res.contains("Location: /login"));

        // 2. Test GET /login -> 200 OK
        let res = send_http_request(port, "GET /login HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("Gym Management Control Center"));

        // 3. Test POST /login with invalid key
        let body = "admin_key=wrong_key";
        let req = format!("POST /login HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nContent-Type: application/x-www-form-urlencoded\r\n\r\n{}", body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("Invalid administrator access key!"));

        // 4. Test POST /login with valid key -> sets cookie and redirects to /admin
        let body = "admin_key=test_admin_key_123";
        let req = format!("POST /login HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nContent-Type: application/x-www-form-urlencoded\r\n\r\n{}", body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("302 Found"));
        assert!(res.contains("Location: /admin"));
        assert!(res.contains("Set-Cookie: admin_session="));

        // Extract session cookie
        let session_cookie = res.lines().find(|l| l.starts_with("Set-Cookie:"))
            .and_then(|l| l.split_once(':'))
            .map(|(_, v)| v.trim().split(';').next().unwrap().to_string())
            .expect("Session cookie not found");

        // 5. Test GET /admin with session cookie -> 200 OK home page with KPI cards only
        let req = format!("GET /admin HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\n\r\n", session_cookie);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("Total Users"));
        assert!(res.contains("Total Pokémon Caught"));
        assert!(res.contains("Latency"));
        assert!(res.contains("Uptime"));
        assert!(!res.contains("Bot Architecture &amp; Storage") && !res.contains("Bot Architecture & Storage"));

        // 5b. Test GET /admin/gyms with session cookie -> 200 OK gyms dashboard
        let req = format!("GET /admin/gyms HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\n\r\n", session_cookie);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("Gym League Management - Admin Dashboard"));
        assert!(res.contains("Pewter City"));

        // 5c. Test GET /api/stats -> returns 200 OK with live telemetry JSON
        let req = format!("GET /api/stats HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\n\r\n", session_cookie);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"success\":true"));
        assert!(res.contains("\"total_users\":1"));
        assert!(res.contains("\"total_pokemon_caught\":25"));
        assert!(res.contains("\"latency_str\":"));
        assert!(res.contains("\"uptime_str\":"));

        // 6. Test POST /api/gym/toggle_global -> close all gyms
        let body = json!({"open": false, "closed_reason": "Maintenance"}).to_string();
        let req = format!("POST /api/gym/toggle_global HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}", session_cookie, body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"gyms_open\":false"));

        // Verify in DB
        let open: i64 = sqlx::query_scalar("SELECT gyms_open FROM gym_global_settings WHERE id = 1").fetch_one(&*pool).await.unwrap();
        assert_eq!(open, 0);

        // 7. Test POST /api/gym/toggle -> toggle Pewter City gym closed
        let body = json!({"region": "kanto", "gym_num": 1, "is_open": false, "closed_reason": "On break"}).to_string();
        let req = format!("POST /api/gym/toggle HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}", session_cookie, body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"is_open\":false"));

        // 8. Test POST /api/gym/save -> update Brock with custom team & rewards
        let body = json!({
            "region": "kanto",
            "gym_num": 1,
            "leader_name": "Brock",
            "gym_name": "Pewter City Gym",
            "city": "Pewter City",
            "poke_type": "Rock",
            "fee": 500,
            "reward_money": 8000,
            "badge": "Boulder Badge",
            "signature_pokemon_id": 95,
            "quote": "Rock solid!",
            "is_open": true,
            "closed_reason": "",
            "rules_text": "Custom Gym Rules",
            "rewards_text": "Custom Rewards",
            "rules": {
                "banned_types": ["Water"],
                "ban_legendaries": true,
                "ban_mythicals": false,
                "ban_ultra_beasts": false,
                "ban_paradox": false,
                "no_duplicates": true,
                "require_conscious": true,
                "max_level": 50
            },
            "rewards": {
                "pokedollars": 8000,
                "mega_stone": "Lucarionite",
                "tm": "26",
                "egg": "random",
                "badge": "Boulder Badge"
            },
            "team": [
                {
                    "pokemon_id": 95,
                    "name": "Titan Onix",
                    "level": 100,
                    "nature": "Impish",
                    "ivs": {"hp": 31, "attack": 31, "defense": 31, "sp_attack": 31, "sp_defense": 31, "speed": 31},
                    "evs": {"hp": 252, "attack": 0, "defense": 252, "sp_attack": 0, "sp_defense": 6, "speed": 0},
                    "moves": [
                        {"name": "Stone Edge", "type": "Rock", "category": "Physical", "power": 100, "accuracy": 80, "pp": 5}
                    ]
                }
            ]
        }).to_string();
        let req = format!("POST /api/gym/save HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}", session_cookie, body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"success\":true"));

        let saved_fee: i64 = sqlx::query_scalar("SELECT fee FROM gym_configs WHERE LOWER(region) = 'kanto' AND gym_num = 1").fetch_one(&*pool).await.unwrap();
        assert_eq!(saved_fee, 500);

        let saved_rules: String = sqlx::query_scalar("SELECT rules_json FROM gym_configs WHERE LOWER(region) = 'kanto' AND gym_num = 1").fetch_one(&*pool).await.unwrap();
        assert!(saved_rules.contains("Water"));
        assert!(saved_rules.contains("\"ban_legendaries\":true"));
        assert!(saved_rules.contains("\"max_level\":50"));

        let saved_rewards: String = sqlx::query_scalar("SELECT rewards_json FROM gym_configs WHERE LOWER(region) = 'kanto' AND gym_num = 1").fetch_one(&*pool).await.unwrap();
        assert!(saved_rewards.contains("Lucarionite"));
        assert!(saved_rewards.contains("\"tm\":\"26\""));
        assert!(saved_rewards.contains("\"egg\":\"random\""));

        // 9. Test POST /api/gym/reset -> resets Brock back to canonical anime defaults
        let body = json!({"region": "kanto", "gym_num": 1}).to_string();
        let req = format!("POST /api/gym/reset HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}", session_cookie, body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"success\":true"));

        let reset_fee: i64 = sqlx::query_scalar("SELECT fee FROM gym_configs WHERE LOWER(region) = 'kanto' AND gym_num = 1").fetch_one(&*pool).await.unwrap();
        assert_eq!(reset_fee, 200);

        let reset_rewards: String = sqlx::query_scalar("SELECT rewards_json FROM gym_configs WHERE LOWER(region) = 'kanto' AND gym_num = 1").fetch_one(&*pool).await.unwrap();
        assert_eq!(reset_rewards, "{}");

        // 10. Test GET /api/pokemon_moves?id=95 -> returns Onix level-up moves
        let req = format!("GET /api/pokemon_moves?id=95 HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\n\r\n", session_cookie);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("Stone Edge") || res.contains("Slam") || res.contains("Rock Throw"));

        // 11. Test GET /admin/redeem -> 200 OK redeem code creator page
        let req = format!("GET /admin/redeem HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\n\r\n", session_cookie);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("Redeem Code Management"));
        assert!(res.contains("Technical Machine (TM)"));
        assert!(res.contains("Move Tutor"));
        assert!(res.contains("Pokémon Egg"));
        assert!(res.contains("Mega Stone"));
        assert!(res.contains("Pokémon Reward"));

        // 12. Test POST /api/redeem/create -> create a promo code with full rewards
        let body = json!({
            "code": "TESTPROMO2026",
            "description": "Test Promo Giveaway",
            "max_uses": 50,
            "max_per_user": 1,
            "expires_at": "2030-12-31",
            "rewards": {
                "pokedollars": 15000,
                "tm": "45",
                "tutor": "1",
                "egg": "random",
                "mega_stone": "Lucarionite",
                "pokemon": {
                    "mode": "random",
                    "category": "legendary",
                    "level": 70,
                    "is_shiny": true
                }
            }
        }).to_string();
        let req = format!("POST /api/redeem/create HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}", session_cookie, body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"success\":true"));

        // Verify in DB
        let code_in_db: String = sqlx::query_scalar("SELECT code FROM redeem_codes WHERE code = 'TESTPROMO2026'")
            .fetch_one(&*pool)
            .await
            .unwrap();
        assert_eq!(code_in_db, "TESTPROMO2026");

        // 13. Test GET /api/redeem/list
        let req = format!("GET /api/redeem/list HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\n\r\n", session_cookie);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"success\":true"));
        assert!(res.contains("TESTPROMO2026"));
        assert!(res.contains("Lucarionite"));

        // 14. Test POST /api/redeem/toggle -> toggle active status
        let code_id: i64 = sqlx::query_scalar("SELECT id FROM redeem_codes WHERE code = 'TESTPROMO2026'")
            .fetch_one(&*pool)
            .await
            .unwrap();
        let body = json!({"id": code_id}).to_string();
        let req = format!("POST /api/redeem/toggle HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}", session_cookie, body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"is_active\":false"));

        let active: i64 = sqlx::query_scalar("SELECT is_active FROM redeem_codes WHERE id = ?")
            .bind(code_id)
            .fetch_one(&*pool)
            .await
            .unwrap();
        assert_eq!(active, 0);

        // 15. Test POST /api/redeem/delete
        let req = format!("POST /api/redeem/delete HTTP/1.1\r\nHost: 127.0.0.1\r\nCookie: {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}", session_cookie, body.len(), body);
        let res = send_http_request(port, &req).await;
        assert!(res.contains("200 OK"));
        assert!(res.contains("\"success\":true"));

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM redeem_codes WHERE id = ?")
            .bind(code_id)
            .fetch_one(&*pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}