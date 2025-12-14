use browser_core::storage::{StorageEngine, Cookie, HistoryEntry, Bookmark};
use std::path::PathBuf;
use tempfile::tempdir;

/// Helper function to get current timestamp for tests
fn current_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

/// One hour in seconds
const ONE_HOUR_SECONDS: i64 = 3600;

fn create_test_storage() -> StorageEngine {
    let dir = tempdir().expect("Failed to create temp dir");
    StorageEngine::new(dir.path()).expect("Failed to create storage engine")
}

#[test]
fn test_cookie_creation() {
    let expires_timestamp = current_timestamp() + ONE_HOUR_SECONDS; // 1 hour from now
    let cookie = Cookie {
        domain: "example.com".to_string(),
        name: "session_id".to_string(),
        value: "abc123".to_string(),
        path: "/".to_string(),
        expires: Some(expires_timestamp),
        http_only: true,
        secure: true,
        same_site: "Strict".to_string(),
    };
    
    assert_eq!(cookie.domain, "example.com");
    assert_eq!(cookie.name, "session_id");
    assert_eq!(cookie.value, "abc123");
    assert!(cookie.http_only);
    assert!(cookie.secure);
}

#[test]
fn test_cookie_serialization() {
    let cookie = Cookie {
        domain: "test.com".to_string(),
        name: "test_cookie".to_string(),
        value: "test_value".to_string(),
        path: "/app".to_string(),
        expires: None,
        http_only: false,
        secure: false,
        same_site: "Lax".to_string(),
    };
    
    let json = serde_json::to_string(&cookie).expect("Failed to serialize cookie");
    assert!(json.contains("test.com"));
    assert!(json.contains("test_cookie"));
    
    let deserialized: Cookie = serde_json::from_str(&json).expect("Failed to deserialize cookie");
    assert_eq!(deserialized.domain, "test.com");
    assert_eq!(deserialized.name, "test_cookie");
}

#[test]
fn test_history_entry_creation() {
    let timestamp = current_timestamp();
    let entry = HistoryEntry {
        id: 1,
        url: "https://example.com/page".to_string(),
        title: Some("Example Page".to_string()),
        visit_count: 5,
        last_visit: timestamp,
    };
    
    assert_eq!(entry.id, 1);
    assert_eq!(entry.url, "https://example.com/page");
    assert_eq!(entry.title, Some("Example Page".to_string()));
    assert_eq!(entry.visit_count, 5);
}

#[test]
fn test_history_entry_serialization() {
    let timestamp = current_timestamp();
    let entry = HistoryEntry {
        id: 2,
        url: "https://test.com".to_string(),
        title: None,
        visit_count: 1,
        last_visit: timestamp,
    };
    
    let json = serde_json::to_string(&entry).expect("Failed to serialize history");
    let deserialized: HistoryEntry = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(deserialized.id, 2);
    assert_eq!(deserialized.url, "https://test.com");
    assert!(deserialized.title.is_none());
}

#[test]
fn test_bookmark_creation() {
    let timestamp = current_timestamp();
    let bookmark = Bookmark {
        id: 1,
        url: "https://github.com".to_string(),
        title: "GitHub".to_string(),
        folder: Some("Development".to_string()),
        created_at: timestamp,
    };
    
    assert_eq!(bookmark.id, 1);
    assert_eq!(bookmark.url, "https://github.com");
    assert_eq!(bookmark.title, "GitHub");
    assert_eq!(bookmark.folder, Some("Development".to_string()));
}

#[test]
fn test_bookmark_without_folder() {
    let timestamp = current_timestamp();
    let bookmark = Bookmark {
        id: 2,
        url: "https://rust-lang.org".to_string(),
        title: "Rust Programming Language".to_string(),
        folder: None,
        created_at: timestamp,
    };
    
    assert_eq!(bookmark.id, 2);
    assert!(bookmark.folder.is_none());
}

#[test]
fn test_bookmark_serialization() {
    let timestamp = current_timestamp();
    let bookmark = Bookmark {
        id: 3,
        url: "https://svelte.dev".to_string(),
        title: "Svelte".to_string(),
        folder: Some("Frontend".to_string()),
        created_at: timestamp,
    };
    
    let json = serde_json::to_string(&bookmark).expect("Failed to serialize bookmark");
    assert!(json.contains("svelte.dev"));
    assert!(json.contains("Frontend"));
    
    let deserialized: Bookmark = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.title, "Svelte");
}

#[test]
fn test_cookie_same_site_values() {
    let cookies = vec![
        Cookie {
            domain: "a.com".to_string(),
            name: "strict".to_string(),
            value: "1".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: "Strict".to_string(),
        },
        Cookie {
            domain: "b.com".to_string(),
            name: "lax".to_string(),
            value: "2".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: "Lax".to_string(),
        },
        Cookie {
            domain: "c.com".to_string(),
            name: "none".to_string(),
            value: "3".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: true,
            same_site: "None".to_string(),
        },
    ];
    
    assert_eq!(cookies[0].same_site, "Strict");
    assert_eq!(cookies[1].same_site, "Lax");
    assert_eq!(cookies[2].same_site, "None");
    // SameSite=None requires Secure flag
    assert!(cookies[2].secure);
}

#[test]
fn test_history_entry_visit_count() {
    let initial_timestamp = current_timestamp();
    let mut entry = HistoryEntry {
        id: 1,
        url: "https://example.com".to_string(),
        title: Some("Example".to_string()),
        visit_count: 0,
        last_visit: initial_timestamp,
    };
    
    // Simulate visiting the page multiple times
    for _ in 0..5 {
        entry.visit_count += 1;
        entry.last_visit += ONE_HOUR_SECONDS; // Advance 1 hour
    }
    
    assert_eq!(entry.visit_count, 5);
    assert_eq!(entry.last_visit, initial_timestamp + 5 * ONE_HOUR_SECONDS);
}
