// SPDX-License-Identifier: GPL-3.0-or-later
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use crate::infrastructure::containers::error::ContainerError;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(10);

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

/// Send an HTTP request to `socket_path` and return the parsed response.
pub fn request(
    socket_path: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<HttpResponse, ContainerError> {
    let stream = UnixStream::connect(socket_path)
        .map_err(|e| ContainerError::ConnectionFailed(format!("{socket_path}: {e}")))?;
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    stream.set_write_timeout(Some(CONNECT_TIMEOUT))?;

    write_request(&stream, method, path, body)?;
    read_response(stream)
}

fn write_request(
    mut stream: &UnixStream,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<(), ContainerError> {
    let body_bytes = body.unwrap_or("").as_bytes();
    let headers = if body.is_some() {
        format!(
            "{method} {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\r\n",
            body_bytes.len()
        )
    } else {
        format!(
            "{method} {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\r\n"
        )
    };

    stream.write_all(headers.as_bytes())?;
    if body.is_some() {
        stream.write_all(body_bytes)?;
    }
    stream.flush()?;
    Ok(())
}

fn read_response(stream: UnixStream) -> Result<HttpResponse, ContainerError> {
    let mut reader = BufReader::new(stream);

    let mut status_line = String::new();
    reader.read_line(&mut status_line)?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut chunked = false;
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        let lower = trimmed.to_lowercase();
        if lower.starts_with("transfer-encoding:") && lower.contains("chunked") {
            chunked = true;
        }
        if lower.starts_with("content-length:") {
            content_length = trimmed
                .split_once(':')
                .map(|x| x.1)
                .and_then(|v| v.trim().parse().ok());
        }
    }

    let body = if chunked {
        read_chunked(&mut reader)?
    } else if let Some(len) = content_length {
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        String::from_utf8_lossy(&buf).into_owned()
    } else {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        buf
    };

    Ok(HttpResponse { status, body })
}

fn read_chunked(reader: &mut BufReader<UnixStream>) -> Result<String, ContainerError> {
    let mut body = Vec::new();
    loop {
        let mut size_line = String::new();
        reader.read_line(&mut size_line)?;
        let chunk_size = usize::from_str_radix(size_line.trim(), 16)
            .map_err(|_| ContainerError::ParseError("Invalid chunk size".into()))?;
        if chunk_size == 0 {
            // Terminal chunk — consume trailing CRLF
            let mut crlf = String::new();
            reader.read_line(&mut crlf)?;
            break;
        }
        let mut chunk = vec![0u8; chunk_size];
        reader.read_exact(&mut chunk)?;
        body.extend_from_slice(&chunk);
        // Consume trailing CRLF after chunk data
        let mut crlf = String::new();
        reader.read_line(&mut crlf)?;
    }
    Ok(String::from_utf8_lossy(&body).into_owned())
}

/// Strip Docker log multiplexing header (8-byte frame header on /logs endpoint).
/// Frame format: [stream_type(1), 0,0,0, size(4-BE)] followed by payload.
pub fn strip_log_frames(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i + 8 <= bytes.len() {
        let size =
            u32::from_be_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]) as usize;
        i += 8;
        if i + size <= bytes.len() {
            out.extend_from_slice(&bytes[i..i + size]);
            i += size;
        } else {
            break;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
