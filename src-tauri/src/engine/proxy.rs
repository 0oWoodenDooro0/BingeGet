use std::sync::atomic::{AtomicU16, Ordering};
use anyhow::Result;
use reqwest::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

static PROXY_PORT: AtomicU16 = AtomicU16::new(0);

pub fn get_proxy_port() -> u16 {
    PROXY_PORT.load(Ordering::SeqCst)
}

pub fn build_proxy_url(port: u16, target_url: &str, referer: &str) -> String {
    let mut u = Url::parse(&format!("http://127.0.0.1:{}/proxy", port))
        .unwrap_or_else(|_| Url::parse("http://127.0.0.1/proxy").unwrap());
    u.query_pairs_mut()
        .append_pair("url", target_url)
        .append_pair("referer", referer);
    u.to_string()
}

pub fn rewrite_m3u8(content: &str, base_url: &str, referer: &str, port: u16) -> String {
    let mut out = String::new();
    let base = Url::parse(base_url).ok();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        let abs_url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed.to_string()
        } else if let Some(ref b) = base {
            match b.join(trimmed) {
                Ok(joined) => joined.to_string(),
                Err(_) => trimmed.to_string(),
            }
        } else {
            trimmed.to_string()
        };

        let proxy_line = build_proxy_url(port, &abs_url, referer);
        out.push_str(&proxy_line);
        out.push('\n');
    }

    out
}

async fn handle_connection(mut stream: TcpStream, client: reqwest::Client, port: u16) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req_str = String::from_utf8_lossy(&buf[..n]);
    let first_line = req_str.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path_and_query = parts[1];

    if method == "OPTIONS" {
        let resp = "HTTP/1.1 200 OK\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, OPTIONS\r\n\
Access-Control-Allow-Headers: *\r\n\
Content-Length: 0\r\n\
\r\n";
        let _ = stream.write_all(resp.as_bytes()).await;
        return;
    }

    if method != "GET" || !path_and_query.starts_with("/proxy") {
        let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes()).await;
        return;
    }

    let dummy_url = format!("http://127.0.0.1{}", path_and_query);
    let parsed_url = match Url::parse(&dummy_url) {
        Ok(u) => u,
        Err(_) => {
            let resp = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
            return;
        }
    };

    let mut target_url = None;
    let mut referer = None;

    for (k, v) in parsed_url.query_pairs() {
        if k == "url" {
            target_url = Some(v.into_owned());
        } else if k == "referer" {
            referer = Some(v.into_owned());
        }
    }

    let target = match target_url {
        Some(u) if !u.trim().is_empty() => u,
        _ => {
            let resp = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
            return;
        }
    };

    let ref_str = referer.unwrap_or_default();

    let mut req = client
        .get(&target)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        );

    if !ref_str.is_empty() {
        req = req.header("Referer", &ref_str);
    }

    let upstream_res = req.send().await;

    match upstream_res {
        Ok(res) => {
            let status = res.status();
            let content_type = res
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();

            let bytes = match res.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    let err_body = format!("HTTP/1.1 502 Bad Gateway\r\n\r\n{}", e);
                    let _ = stream.write_all(err_body.as_bytes()).await;
                    return;
                }
            };

            // If it is an M3U8 playlist, rewrite the URLs
            let is_m3u8 = content_type.contains("mpegurl")
                || target.contains(".m3u8")
                || bytes.starts_with(b"#EXTM3U");

            if is_m3u8 {
                let text = String::from_utf8_lossy(&bytes);
                let rewritten = rewrite_m3u8(&text, &target, &ref_str, port);
                let rewritten_bytes = rewritten.into_bytes();

                let header = format!(
                    "HTTP/1.1 {} OK\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, OPTIONS\r\n\
Content-Type: application/vnd.apple.mpegurl\r\n\
Content-Length: {}\r\n\
\r\n",
                    status.as_u16(),
                    rewritten_bytes.len()
                );

                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(&rewritten_bytes).await;
            } else {
                let header = format!(
                    "HTTP/1.1 {} OK\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, OPTIONS\r\n\
Content-Type: {}\r\n\
Content-Length: {}\r\n\
\r\n",
                    status.as_u16(),
                    content_type,
                    bytes.len()
                );

                let _ = stream.write_all(header.as_bytes()).await;
                let _ = stream.write_all(&bytes).await;
            }
        }
        Err(e) => {
            let err_body = format!("HTTP/1.1 502 Bad Gateway\r\n\r\nFailed to fetch stream: {}", e);
            let _ = stream.write_all(err_body.as_bytes()).await;
        }
    }
}

pub async fn start_streaming_proxy(client: reqwest::Client) -> Result<u16> {
    let current = get_proxy_port();
    if current > 0 {
        return Ok(current);
    }

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    PROXY_PORT.store(port, Ordering::SeqCst);

    tokio::spawn(async move {
        loop {
            if let Ok((socket, _)) = listener.accept().await {
                let c = client.clone();
                tokio::spawn(async move {
                    handle_connection(socket, c, port).await;
                });
            }
        }
    });

    Ok(port)
}
