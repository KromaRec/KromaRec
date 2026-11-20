use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use percent_encoding::percent_decode_str;
use std::{path::Path, time::UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

const CACHE: &str = "public, max-age=2592000";

fn range(value: &str, size: u64) -> Option<(u64, u64)> {
    let (first, last) = value.strip_prefix("bytes=")?.split_once('-')?;
    if size == 0 {
        return None;
    }
    if first.is_empty() {
        let suffix = last.parse::<u64>().ok()?.min(size);
        return (suffix > 0).then(|| (size - suffix, size - 1));
    }
    let start = first.parse::<u64>().ok()?;
    let end = if last.is_empty() {
        size - 1
    } else {
        last.parse::<u64>().ok()?.min(size - 1)
    };
    (start <= end).then_some((start, end))
}

pub async fn serve(root: &Path, request: Request<Body>, head: bool) -> Response {
    let mut response = serve_file(root, request, head).await;
    let headers = response.headers_mut();
    headers.insert("access-control-allow-origin", "*".parse().unwrap());
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert(
        "content-security-policy",
        "sandbox; default-src 'none'".parse().unwrap(),
    );
    response
}

async fn serve_file(root: &Path, request: Request<Body>, head: bool) -> Response {
    let path = request.uri().path();
    if path == "/" {
        return crate::json_body(StatusCode::OK, r#"{"service":"cdn","status":"ok"}"#);
    }
    if path == "/config/LoadingScreenTipData" {
        let mut response = crate::json_body(
            StatusCode::OK,
            include_str!("../assets/loading-screen-tip-data.json"),
        );
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, CACHE.parse().unwrap());
        return response;
    }
    let decoded = match percent_decode_str(path).decode_utf8() {
        Ok(value) => value,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let Some((prefix, key)) = decoded.trim_start_matches('/').split_once('/') else {
        return crate::not_found();
    };
    let config = prefix == "config";
    if !matches!(
        prefix,
        "config" | "sigs" | "room" | "invention" | "data" | "avatar" | "roomunityasset"
    ) {
        return crate::not_found();
    }
    if key.contains("..")
        || key.contains('\\')
        || key.contains('\0')
        || key.split('/').any(|part| part.is_empty() || part == ".")
    {
        return if config {
            crate::not_found()
        } else {
            StatusCode::BAD_REQUEST.into_response()
        };
    }
    if (config
        && !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-')))
        || (prefix == "sigs" && key.contains('/'))
    {
        return crate::not_found();
    }

    let relative = format!("{prefix}/{key}");
    let lookup = root.join(&relative);
    let path = match tokio::fs::canonicalize(&lookup).await {
        Ok(file) => file,
        Err(_) if config && !key.contains('.') => {
            let Ok(file) = tokio::fs::canonicalize(root.join(format!("{relative}.json"))).await
            else {
                return crate::not_found();
            };
            file
        }
        Err(_) => return crate::not_found(),
    };
    if !path.starts_with(root)
        || !tokio::fs::metadata(&path)
            .await
            .is_ok_and(|meta| meta.is_file())
    {
        return crate::not_found();
    }
    let Ok(mut file) = tokio::fs::File::open(&path).await else {
        return crate::not_found();
    };
    let Ok(meta) = file.metadata().await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let size = meta.len();
    let timestamp = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |time| time.as_nanos());
    let etag = format!("\"{size:x}-{timestamp:x}\"");
    let mime = if config && path.extension().is_some_and(|ext| ext == "json") {
        "application/json"
    } else {
        "application/octet-stream"
    };
    let response = Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::ETAG, &etag)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, CACHE);
    let headers = request.headers();
    if headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(',')
                .map(str::trim)
                .any(|item| item == "*" || item.trim_start_matches("W/") == etag)
        })
    {
        return response
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .unwrap();
    }
    let slice = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok());
    let (response, offset, length) = match slice {
        Some(value)
            if value.starts_with("bytes=")
                && headers
                    .get(header::IF_RANGE)
                    .is_none_or(|v| v.as_bytes() == etag.as_bytes()) =>
        {
            let Some((start, end)) = range(value, size) else {
                return response
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(header::CONTENT_RANGE, format!("bytes */{size}"))
                    .body(Body::empty())
                    .unwrap();
            };
            (
                response
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{size}")),
                start,
                end - start + 1,
            )
        }
        _ => (response.status(StatusCode::OK), 0, size),
    };
    let body = if head || length == 0 {
        Body::empty()
    } else {
        if offset > 0 && file.seek(std::io::SeekFrom::Start(offset)).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Body::from_stream(ReaderStream::with_capacity(file.take(length), 16 * 1024))
    };
    response
        .header(header::CONTENT_LENGTH, length)
        .body(body)
        .unwrap()
}
