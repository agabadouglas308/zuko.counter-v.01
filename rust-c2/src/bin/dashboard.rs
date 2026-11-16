use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

const HTML: &str = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>CUAS Simulator Operator Review</title>
<style>
body{font:15px system-ui,sans-serif;background:#10151c;color:#e8eef5;margin:0}main{max-width:1100px;margin:0 auto;padding:24px}h1{margin:0 0 6px}p{color:#9eabb8}.banner{background:#17212b;border:1px solid #2c3b49;border-radius:8px;padding:12px;margin:18px 0}.table{overflow:auto}table{width:100%;border-collapse:collapse;background:#151c24}th,td{padding:10px;border-bottom:1px solid #2b3540;text-align:left;white-space:nowrap}th{color:#9eabb8;font-size:12px;text-transform:uppercase}button{background:#2d7dce;color:white;border:0;border-radius:5px;padding:7px 10px;cursor:pointer}button[disabled]{background:#45515d;cursor:default}.review{color:#ffcf66}.security{color:#ff8c8c}.observe{color:#87d7a1}.muted{color:#9eabb8}
</style></head><body><main>
<h1>CUAS Simulator Operator Review</h1>
<p>Local synthetic-data dashboard. Recommendations are advisory. No effector or engagement controls are present.</p>
<div class="banner"><b>Human review state:</b> <span id="summary">Loading…</span></div>
<div class="table"><table><thead><tr><th>Time</th><th>Target</th><th>Sensor</th><th>Confidence</th><th>Recommendation</th><th>Review</th></tr></thead><tbody id="rows"></tbody></table></div>
</main><script>
async function refresh(){const [events,acks]=await Promise.all([fetch('/api/events').then(r=>r.json()),fetch('/api/acks').then(r=>r.json())]);
const rows=document.getElementById('rows'); rows.innerHTML=''; let pending=0;
for(const e of events){const id=e.event_id; const acked=acks.includes(id); if(!acked) pending++;
const tr=document.createElement('tr'); const rec=e.recommendation||''; const cls=rec.includes('review')?'review':rec.includes('security')?'security':'observe';
tr.innerHTML=`<td>${e.timestamp_ms??''}</td><td>${e.target_id??''}</td><td>${e.source_sensor??''}</td><td>${((e.confidence||0)*100).toFixed(1)}%</td><td class="${cls}">${rec}</td><td></td>`;
const cell=tr.lastElementChild; const b=document.createElement('button'); b.textContent=acked?'Reviewed':'Acknowledge review'; b.disabled=acked; b.onclick=async()=>{await fetch('/api/ack',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({event_id:id})});refresh()}; cell.appendChild(b); rows.appendChild(tr)}
document.getElementById('summary').textContent=`${pending} pending / ${events.length} total track updates`}
refresh(); setInterval(refresh,3000);
</script></body></html>"#;

#[derive(Clone)]
struct AppState {
    audit_path: String,
    acknowledgements: Arc<Mutex<HashSet<String>>>,
}

fn response(status: &str, content_type: &str, body: &str) -> String {
    format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
}

fn events(state: &AppState) -> Vec<Value> {
    fs::read_to_string(&state.audit_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect()
}

fn handle(mut stream: TcpStream, state: &AppState) {
    let mut head = [0u8; 4096];
    let size = stream.read(&mut head).unwrap_or(0);
    let request = String::from_utf8_lossy(&head[..size]);
    let first = request.lines().next().unwrap_or("");
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");
    let result = match (method, path) {
        ("GET", "/") => response("200 OK", "text/html; charset=utf-8", HTML),
        ("GET", "/api/events") => response(
            "200 OK",
            "application/json",
            &serde_json::to_string(&events(state)).unwrap(),
        ),
        ("GET", "/api/acks") => {
            let acks = state.acknowledgements.lock().unwrap();
            response(
                "200 OK",
                "application/json",
                &serde_json::to_string(&*acks).unwrap(),
            )
        }
        ("POST", "/api/ack") => {
            let payload = request.split("\r\n\r\n").nth(1).unwrap_or("");
            match serde_json::from_str::<Value>(payload)
                .ok()
                .and_then(|v| v.get("event_id").and_then(Value::as_str).map(str::to_owned))
            {
                Some(id) if !id.is_empty() => {
                    state.acknowledgements.lock().unwrap().insert(id);
                    response("200 OK", "application/json", "{\"ok\":true}")
                }
                _ => response(
                    "400 Bad Request",
                    "application/json",
                    "{\"ok\":false,\"error\":\"event_id required\"}",
                ),
            }
        }
        _ => response("404 Not Found", "text/plain", "not found"),
    };
    let _ = stream.write_all(result.as_bytes());
    let _ = stream.flush();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let audit = args
        .windows(2)
        .find(|w| w[0] == "--audit-log")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "audit.ndjson".into());
    let port = args
        .windows(2)
        .find(|w| w[0] == "--port")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "8080".into());
    let listener =
        TcpListener::bind(format!("127.0.0.1:{port}")).expect("cannot bind local dashboard");
    let state = AppState {
        audit_path: audit.clone(),
        acknowledgements: Arc::new(Mutex::new(HashSet::new())),
    };
    eprintln!("dashboard listening on http://127.0.0.1:{port} using {audit}");
    for stream in listener.incoming().flatten() {
        handle(stream, &state);
    }
}
