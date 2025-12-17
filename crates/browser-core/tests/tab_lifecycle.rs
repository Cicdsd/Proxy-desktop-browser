use browser_core::TabStatus;
use serde_json::{to_string, from_str};

#[test]
fn test_tab_status_creating() {
    // Test TabStatus variant exists
    let status = TabStatus::Creating;
    let serialized = to_string(&status).unwrap();
    assert!(serialized.contains("Creating"));
}

#[test]
fn test_tab_status_active() {
    let status = TabStatus::Active;
    let serialized = to_string(&status).unwrap();
    assert!(serialized.contains("Active"));
}

#[test]
fn test_tab_status_closed() {
    let status = TabStatus::Closed;
    let serialized = to_string(&status).unwrap();
    assert!(serialized.contains("Closed"));
}

#[test]
fn test_tab_status_serialization_roundtrip() {
    let original = TabStatus::Active;
    let serialized = to_string(&original).unwrap();
    let deserialized: TabStatus = from_str(&serialized).unwrap();
    
    // Verify by re-serializing
    let reserialized = to_string(&deserialized).unwrap();
    assert_eq!(serialized, reserialized);
}
