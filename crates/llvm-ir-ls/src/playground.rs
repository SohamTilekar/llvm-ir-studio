use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// Embed static web playground assets directly into binary
const INDEX_HTML: &str = include_str!("../../../web/index.html");
const STYLE_CSS: &str = include_str!("../../../web/css/style.css");
const APP_JS: &str = include_str!("../../../web/js/app.js");
const ANALYZER_JS: &str = include_str!("../../../web/js/analyzer.js");
const VISUALIZER_JS: &str = include_str!("../../../web/js/visualizer.js");
const PRESETS_JS: &str = include_str!("../../../web/js/presets.js");
const MONACO_LLVM_JS: &str = include_str!("../../../web/js/monaco-llvm.js");
const ICON_PNG: &[u8] = include_bytes!("../../../web/assets/icon.png");

pub async fn run_playground(file: Option<PathBuf>, port: u16) -> ExitCode {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to http://{}: {}", addr, e);
            return ExitCode::FAILURE;
        }
    };

    let initial_file_content: Option<String> = if let Some(ref path) = file {
        match fs::read_to_string(path) {
            Ok(content) => Some(content),
            Err(e) => {
                eprintln!("Warning: could not read file '{:?}': {}", path, e);
                None
            }
        }
    } else {
        None
    };

    println!("============================================================");
    println!("   LLVM IR Studio - Interactive In-Browser Playground       ");
    println!("============================================================");
    println!("   Server running at: http://127.0.0.1:{}", port);
    if let Some(ref path) = file {
        println!("   Pre-loaded file:   {:?}", path);
    }
    println!("   Press Ctrl+C to terminate the server.\n");

    loop {
        let (mut socket, _) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };

        let file_content = initial_file_content.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            let n = match socket.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req_str = String::from_utf8_lossy(&buf[..n]);
            let first_line = req_str.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() < 2 {
                return;
            }

            let path = parts[1].split('?').next().unwrap_or("/");

            let (status, content_type, body_bytes) = match path {
                "/" | "/index.html" => {
                    // If we have an initial file, inject a script tag to auto-load it
                    if let Some(ref code) = file_content {
                        let encoded = base64_encode(code.as_bytes());
                        let injected = INDEX_HTML.replace(
                            "</body>",
                            &format!(
                                r#"<script>
window.addEventListener('DOMContentLoaded', () => {{
  const check = setInterval(() => {{
    if (window.app && window.app.editor) {{
      clearInterval(check);
      try {{
        const code = decodeURIComponent(escape(atob('{}')));
        window.app.editor.setValue(code);
      }} catch(e) {{}}
    }}
  }}, 100);
}});
</script></body>"#,
                                encoded
                            ),
                        );
                        (200, "text/html; charset=utf-8", injected.into_bytes())
                    } else {
                        (200, "text/html; charset=utf-8", INDEX_HTML.as_bytes().to_vec())
                    }
                }
                "/css/style.css" => (200, "text/css; charset=utf-8", STYLE_CSS.as_bytes().to_vec()),
                "/js/app.js" => (
                    200,
                    "application/javascript; charset=utf-8",
                    APP_JS.as_bytes().to_vec(),
                ),
                "/js/analyzer.js" => (
                    200,
                    "application/javascript; charset=utf-8",
                    ANALYZER_JS.as_bytes().to_vec(),
                ),
                "/js/visualizer.js" => (
                    200,
                    "application/javascript; charset=utf-8",
                    VISUALIZER_JS.as_bytes().to_vec(),
                ),
                "/js/presets.js" => (
                    200,
                    "application/javascript; charset=utf-8",
                    PRESETS_JS.as_bytes().to_vec(),
                ),
                "/js/monaco-llvm.js" => (
                    200,
                    "application/javascript; charset=utf-8",
                    MONACO_LLVM_JS.as_bytes().to_vec(),
                ),
                "/assets/icon.png" => (200, "image/png", ICON_PNG.to_vec()),
                _ => (
                    404,
                    "text/plain",
                    b"404 Not Found - LLVM IR Studio".to_vec(),
                ),
            };

            let response = format!(
                "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                status,
                content_type,
                body_bytes.len()
            );

            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.write_all(&body_bytes).await;
            let _ = socket.flush().await;
        });
    }
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);

        if i + 1 < data.len() {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < data.len() {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }

    out
}
