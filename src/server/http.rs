use crate::error::Result;
use crate::server::text_files::is_text_file;
use axum::{
    extract::{Path as AxumPath, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};
use tokio::fs;
use tokio_util::io::ReaderStream;
use tracing::{info, warn};

#[derive(Clone)]
struct IgnorePatterns {
    patterns: Vec<String>,
}

impl IgnorePatterns {
    fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    fn add_pattern(&mut self, pattern: &str) {
        let pattern = pattern.trim();
        if !pattern.is_empty() && !pattern.starts_with('#') {
            self.patterns.push(pattern.to_string());
        }
    }

    fn is_ignored(&self, path: &Path, base_path: &Path) -> bool {
        let relative_path = path.strip_prefix(base_path).unwrap_or(path);
        let path_str = relative_path.to_string_lossy();

        for pattern in &self.patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }
        false
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        let pattern = pattern.trim_start_matches('/');
        let path = path.trim_start_matches('/');

        if pattern == path {
            return true;
        }

        if pattern.contains('*') {
            let glob_pattern = if pattern.starts_with('*') {
                pattern.to_string()
            } else if pattern.contains('/') {
                format!("**/{}", pattern)
            } else {
                format!("**/{}", pattern)
            };

            if let Ok(matcher) = glob::Pattern::new(&glob_pattern) {
                return matcher.matches(path);
            }
        }

        if path.starts_with(&format!("{}/", pattern)) || path == pattern {
            return true;
        }

        if pattern.starts_with('/') && path == pattern.trim_start_matches('/') {
            return true;
        }

        false
    }
}

#[derive(Deserialize)]
pub struct TokenQuery {
    token: Option<String>,
    download: Option<String>,
}

async fn get_file_size(path: &Path) -> Option<u64> {
    fs::metadata(path).await.ok().map(|m| m.len())
}

async fn load_ignore_patterns(base_path: &Path) -> IgnorePatterns {
    let ignore_file = base_path.join(".shuttle");
    let mut patterns = IgnorePatterns::new();

    if let Ok(content) = fs::read_to_string(&ignore_file).await {
        for line in content.lines() {
            patterns.add_pattern(line);
        }
    }

    patterns
}

async fn render_file_viewer(file_path: &Path, content: &str) -> Result<String> {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["base16-ocean.dark"];

    let extension = file_path.extension().and_then(|e| e.to_str());
    let syntax = extension
        .and_then(|ext| syntax_set.find_syntax_by_extension(ext))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let highlighted =
        highlighted_html_for_string(content, &syntax_set, syntax, theme).map_err(|e| {
            crate::error::ShuttleError::InvalidConfig(format!("Syntax highlighting failed: {}", e))
        })?;

    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: monospace; padding: 20px; }}
        .header {{ margin-bottom: 20px; }}
        .download {{ margin-left: 20px; }}
        pre {{ background: #f5f5f5; padding: 15px; overflow-x: auto; }}
    </style>
</head>
<body>
    <div class="header">
        <strong>{}</strong>
        <a href="?download" class="download">Download</a>
    </div>
    <pre>{}</pre>
</body>
</html>"#,
        filename, filename, highlighted
    );

    Ok(html)
}

pub async fn serve(path: &Path, bind: &str, port: u16, require_token: bool) -> Result<()> {
    if !path.exists() {
        return Err(crate::error::ShuttleError::FileNotFound(
            path.display().to_string(),
        ));
    }

    let addr: SocketAddr = format!("{}:{}", bind, port).parse().map_err(|_| {
        crate::error::ShuttleError::InvalidConfig(format!("Invalid address: {}:{}", bind, port))
    })?;

    let serve_path = Arc::new(path.to_path_buf());
    let ignore_patterns = Arc::new(load_ignore_patterns(path).await);
    let token = if require_token {
        Some(uuid::Uuid::new_v4().to_string())
    } else {
        None
    };
    let token = Arc::new(token);

    let app = Router::new()
        .route(
            "/",
            get({
                let serve_path = serve_path.clone();
                let token = token.clone();
                let ignore_patterns = ignore_patterns.clone();
                move |query| {
                    serve_root(
                        serve_path.clone(),
                        query,
                        token.clone(),
                        ignore_patterns.clone(),
                    )
                }
            }),
        )
        .route(
            "/*path",
            get({
                let serve_path = serve_path.clone();
                let token = token.clone();
                let ignore_patterns = ignore_patterns.clone();
                move |path, query| {
                    serve_file(
                        serve_path.clone(),
                        path,
                        query,
                        token.clone(),
                        ignore_patterns.clone(),
                    )
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let network_info = crate::network::NetworkInfo::new(addr);
    network_info.show_access_info(token.as_deref());

    info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_root(
    serve_path: Arc<PathBuf>,
    Query(query): Query<TokenQuery>,
    token: Arc<Option<String>>,
    ignore_patterns: Arc<IgnorePatterns>,
) -> Response {
    info!("GET / - token: {}", query.token.is_some());

    if !check_token(&query.token, &token) {
        warn!("Unauthorized access attempt to /");
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    if serve_path.is_file() {
        serve_single_file(&serve_path, Query(query)).await
    } else {
        serve_directory(&serve_path, "", ignore_patterns).await
    }
}

async fn serve_file(
    serve_path: Arc<PathBuf>,
    AxumPath(path): AxumPath<String>,
    Query(query): Query<TokenQuery>,
    token: Arc<Option<String>>,
    ignore_patterns: Arc<IgnorePatterns>,
) -> Response {
    if path != "favicon.ico" {
        info!("GET /{} - token: {}", path, query.token.is_some());
    }

    if !check_token(&query.token, &token) {
        warn!("Unauthorized access attempt to /{}", path);
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let file_path = serve_path.join(&path);

    if ignore_patterns.is_ignored(&file_path, &serve_path) {
        warn!("Access denied to ignored path: /{}", path);
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    if !file_path.exists() {
        if path != "favicon.ico" {
            warn!("File not found: /{}", path);
        }
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    if file_path.is_dir() {
        serve_directory(&file_path, &path, ignore_patterns).await
    } else {
        info!("Serving file: {}", path);
        serve_single_file(&file_path, Query(query)).await
    }
}

fn check_token(provided: &Option<String>, required: &Arc<Option<String>>) -> bool {
    match (provided, required.as_ref()) {
        (Some(p), Some(r)) => p == r,
        (None, None) => true,
        _ => false,
    }
}

async fn serve_single_file(file_path: &Path, query: Query<TokenQuery>) -> Response {
    if query.download.is_some() {
        match fs::File::open(file_path).await {
            Ok(file) => {
                let mime = mime_guess::from_path(file_path).first_or_octet_stream();
                let stream = ReaderStream::new(file);
                let filename = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                (
                    [
                        (axum::http::header::CONTENT_TYPE, mime.as_ref().to_string()),
                        (
                            axum::http::header::CONTENT_DISPOSITION,
                            format!("attachment; filename=\"{}\"", filename),
                        ),
                    ],
                    axum::body::Body::from_stream(stream),
                )
                    .into_response()
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response(),
        }
    } else {
        match fs::read_to_string(file_path).await {
            Ok(content) => match render_file_viewer(file_path, &content).await {
                Ok(html) => Html(html).into_response(),
                Err(_) => {
                    let mime = mime_guess::from_path(file_path).first_or_octet_stream();
                    let stream = ReaderStream::new(std::io::Cursor::new(content));
                    (
                        [(axum::http::header::CONTENT_TYPE, mime.as_ref().to_string())],
                        axum::body::Body::from_stream(stream),
                    )
                        .into_response()
                }
            },
            Err(_) => {
                let file_size = get_file_size(file_path).await;
                let filename = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                let size_str = file_size
                    .map(|s| format!("{} bytes", s))
                    .unwrap_or_else(|| "unknown".to_string());

                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <title>{} - Binary File</title>
    <style>
        body {{ font-family: monospace; padding: 20px; }}
        .header {{ margin-bottom: 20px; }}
        .download {{ margin-left: 20px; }}
    </style>
</head>
<body>
    <div class="header">
        <strong>{}</strong>
        <a href="?download" class="download">Download</a>
    </div>
    <p>This is a binary file ({}) that cannot be displayed as text.</p>
</body>
</html>"#,
                    filename, filename, size_str
                );
                Html(html).into_response()
            }
        }
    }
}

async fn serve_directory(
    dir_path: &Path,
    url_path: &str,
    ignore_patterns: Arc<IgnorePatterns>,
) -> Response {
    match fs::read_dir(dir_path).await {
        Ok(mut entries) => {
            let mut html = String::from(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Directory Listing</title>
    <style>
        body {{ font-family: monospace; padding: 20px; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ padding: 5px 0; }}
        a {{ text-decoration: none; color: blue; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>Directory Listing</h1>"#,
            );

            if !url_path.is_empty() && url_path != "/" {
                html.push_str(&format!("<p>/{}</p>", url_path));
            }

            html.push_str("<ul>");

            if !url_path.is_empty() && url_path != "/" {
                html.push_str("<li><a href='..'>..</a></li>");
            }

            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(name) = entry.file_name().into_string() {
                    let entry_path = dir_path.join(&name);

                    if ignore_patterns.is_ignored(&entry_path, dir_path) {
                        continue;
                    }

                    let entry_path = name.clone();

                    match entry.file_type().await {
                        Ok(ft) => {
                            if ft.is_dir() {
                                html.push_str(&format!(
                                    "<li><a href='{}/'>{}/</a></li>",
                                    entry_path, name
                                ));
                            } else {
                                html.push_str(&format!(
                                    "<li><a href='{}'>{}</a></li>",
                                    entry_path, name
                                ));
                            }
                        }
                        Err(_) => {}
                    }
                }
            }

            html.push_str("</ul></body></html>");
            Html(html).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read directory",
        )
            .into_response(),
    }
}
