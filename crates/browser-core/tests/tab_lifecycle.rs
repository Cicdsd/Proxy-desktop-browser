use browser_core::TabStatus;

#[test]
fn test_tab_status_creating() {
    // Test TabStatus variant exists
    let status = TabStatus::Creating;
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(serialized.contains("Creating"));
}

#[test]
fn test_tab_status_active() {
    let status = TabStatus::Active;
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(serialized.contains("Active"));
}

#[test]
fn test_tab_status_closed() {
    let status = TabStatus::Closed;
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(serialized.contains("Closed"));
}

#[test]
fn test_tab_status_serialization_roundtrip() {
    let original = TabStatus::Active;
    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: TabStatus = serde_json::from_str(&serialized).unwrap();
    
    // Verify by re-serializing
    let reserialized = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(serialized, reserialized);
}
