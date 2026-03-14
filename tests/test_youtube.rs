//! YouTube upload and update tests (YT-U1–YT-U8).
//!
//! Uses a mock HTTP server to simulate YouTube API responses.

mod common;

use corky::social::youtube::{
    get_channel_id_at, map_visibility, update_video_at, upload_video_at,
    VideoMetadata,
};
use std::io::Write;
use tempfile::NamedTempFile;
use tiny_http::{Response, Server};

fn test_metadata() -> VideoMetadata {
    VideoMetadata {
        title: "Test Video".to_string(),
        description: "A test description".to_string(),
        tags: vec!["test".to_string(), "rust".to_string()],
        visibility: "private".to_string(),
        category_id: String::new(),
    }
}

// YT-U1: map_visibility returns correct values
#[test]
fn yt_u1_map_visibility() {
    assert_eq!(map_visibility("public").unwrap(), "public");
    assert_eq!(map_visibility("unlisted").unwrap(), "unlisted");
    assert_eq!(map_visibility("private").unwrap(), "private");
    assert_eq!(map_visibility("Private").unwrap(), "private");
    assert!(map_visibility("invalid").is_err());
}

// YT-U2: get_channel_id parses channel response
#[test]
fn yt_u2_get_channel_id() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let addr = server.server_addr().to_ip().unwrap();
    let api_base = format!("http://{}", addr);

    let handle = std::thread::spawn(move || {
        let req = server.recv().unwrap();
        assert!(req.url().contains("/youtube/v3/channels"));
        let body = r#"{"items":[{"id":"UC_test_channel_123"}]}"#;
        req.respond(Response::from_string(body).with_header(
            "Content-Type: application/json".parse::<tiny_http::Header>().unwrap(),
        ))
        .unwrap();
    });

    let result = get_channel_id_at(&api_base, "fake_token");
    handle.join().unwrap();
    assert_eq!(result.unwrap(), "UC_test_channel_123");
}

// YT-U3: get_channel_id fails on empty items
#[test]
fn yt_u3_get_channel_id_no_channel() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let addr = server.server_addr().to_ip().unwrap();
    let api_base = format!("http://{}", addr);

    let handle = std::thread::spawn(move || {
        let req = server.recv().unwrap();
        let body = r#"{"items":[]}"#;
        req.respond(Response::from_string(body).with_header(
            "Content-Type: application/json".parse::<tiny_http::Header>().unwrap(),
        ))
        .unwrap();
    });

    let result = get_channel_id_at(&api_base, "fake_token");
    handle.join().unwrap();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No YouTube channel"));
}

// YT-U4: upload_video handles resumable upload with final JSON response
#[test]
fn yt_u4_upload_video_small_file() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let addr = server.server_addr().to_ip().unwrap();
    let upload_base = format!("http://{}/upload/youtube/v3/videos", addr);
    let upload_url = format!("http://{}/upload/resume", addr);

    // Create a small temp video file
    let mut video_file = NamedTempFile::new().unwrap();
    video_file.write_all(b"fake video content here").unwrap();
    video_file.flush().unwrap();

    let upload_url_clone = upload_url.clone();
    let handle = std::thread::spawn(move || {
        // Request 1: Init resumable upload — return Location header
        let req = server.recv().unwrap();
        assert!(req.url().contains("uploadType=resumable"));
        req.respond(
            Response::from_string("")
                .with_header(
                    format!("Location: {}", upload_url_clone)
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                )
                .with_status_code(200),
        )
        .unwrap();

        // Request 2: Chunk upload — return final response with video ID
        let req = server.recv().unwrap();
        assert!(req.url().contains("/upload/resume"));
        let body = r#"{"id":"VIDEO_ID_123","snippet":{"title":"Test Video"}}"#;
        req.respond(
            Response::from_string(body)
                .with_header(
                    "Content-Type: application/json"
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                )
                .with_status_code(200),
        )
        .unwrap();
    });

    let metadata = test_metadata();
    let result = upload_video_at(&upload_base, "fake_token", video_file.path(), &metadata);
    handle.join().unwrap();
    assert_eq!(result.unwrap(), "VIDEO_ID_123");
}

// YT-U5: upload_video handles empty body on 200 (the bug fix)
#[test]
fn yt_u5_upload_video_empty_body_then_final() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let addr = server.server_addr().to_ip().unwrap();
    let upload_base = format!("http://{}/upload/youtube/v3/videos", addr);
    let upload_url = format!("http://{}/upload/resume", addr);

    // Create a small temp video file (will be sent as one chunk)
    let mut video_file = NamedTempFile::new().unwrap();
    video_file.write_all(b"fake video data").unwrap();
    video_file.flush().unwrap();

    let upload_url_clone = upload_url.clone();
    let handle = std::thread::spawn(move || {
        // Request 1: Init — return Location
        let req = server.recv().unwrap();
        req.respond(
            Response::from_string("")
                .with_header(
                    format!("Location: {}", upload_url_clone)
                        .parse::<tiny_http::Header>()
                        .unwrap(),
                )
                .with_status_code(200),
        )
        .unwrap();

        // Request 2: Chunk — return 200 with EMPTY body (this was the bug)
        let req = server.recv().unwrap();
        req.respond(Response::from_string("").with_status_code(200))
            .unwrap();

        // Request 3: Next read returns 0 bytes (EOF), loop breaks
        // But actually the code will try to read again from the file, get 0 bytes, and bail.
        // So the upload will end with the "completed without receiving video ID" error.
        // That's expected — in real YouTube, the final chunk returns the JSON body.
    });

    let metadata = test_metadata();
    let result = upload_video_at(&upload_base, "fake_token", video_file.path(), &metadata);
    handle.join().unwrap();
    // With the bug fix, empty body causes the loop to continue.
    // Since file is fully read, next iteration reads 0 bytes and breaks.
    // Result is the "completed without receiving video ID" error — not a JSON parse crash.
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("completed without receiving a video ID"),
        "Expected graceful error, got: {}",
        err
    );
}

// YT-U6: update_video sends correct PUT request
#[test]
fn yt_u6_update_video() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let addr = server.server_addr().to_ip().unwrap();
    let api_base = format!("http://{}", addr);

    let handle = std::thread::spawn(move || {
        let req = server.recv().unwrap();
        assert!(req.url().contains("/youtube/v3/videos"));
        assert!(req.url().contains("part=snippet,status"));
        assert_eq!(req.method().as_str(), "PUT");
        req.respond(
            Response::from_string(r#"{"id":"VIDEO_ID_123"}"#)
                .with_status_code(200),
        )
        .unwrap();
    });

    let metadata = test_metadata();
    let result = update_video_at(&api_base, "fake_token", "VIDEO_ID_123", &metadata);
    handle.join().unwrap();
    assert!(result.is_ok());
}

// YT-U7: update_video returns error on 403
#[test]
fn yt_u7_update_video_forbidden() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let addr = server.server_addr().to_ip().unwrap();
    let api_base = format!("http://{}", addr);

    let handle = std::thread::spawn(move || {
        let req = server.recv().unwrap();
        req.respond(
            Response::from_string(r#"{"error":{"message":"Forbidden"}}"#)
                .with_status_code(403),
        )
        .unwrap();
    });

    let metadata = test_metadata();
    let result = update_video_at(&api_base, "fake_token", "VIDEO_ID_123", &metadata);
    handle.join().unwrap();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("403"));
}

// YT-U8: upload_video rejects missing file
#[test]
fn yt_u8_upload_missing_file() {
    let metadata = test_metadata();
    let result = upload_video_at(
        "http://unused",
        "fake_token",
        std::path::Path::new("/tmp/nonexistent_video_12345.mp4"),
        &metadata,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}
