use browser_core::browser_controls::{BrowserController, BrowserState, BrowserSettings, WebRtcPolicy, HistoryItem};
use browser_core::chromium_engine::BrowserEngineType;

/// Constant representing an empty history index (no history entries)
const EMPTY_HISTORY_INDEX: i32 = -1;

#[tokio::test]
async fn test_browser_controller_creation() {
    let controller = BrowserController::new();
    let settings = controller.get_settings().await;
    
    // Check default settings
    assert!(settings.javascript_enabled);
    assert!(settings.cookies_enabled);
    assert!(settings.dns_over_https);
}

#[tokio::test]
async fn test_browser_state_default() {
    let state = BrowserState::default();
    
    assert!(state.tab_id.is_empty());
    assert_eq!(state.current_url, "about:blank");
    assert_eq!(state.title, "New Tab");
    assert!(!state.can_go_back);
    assert!(!state.can_go_forward);
    assert!(!state.is_loading);
    assert!(state.history.is_empty());
    assert_eq!(state.history_index, EMPTY_HISTORY_INDEX);
}

#[tokio::test]
async fn test_browser_controller_navigate() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_1";
    
    // Navigate to a URL
    let state = controller.navigate(tab_id, "https://example.com").await.unwrap();
    
    assert_eq!(state.current_url, "https://example.com");
    assert!(state.is_loading);
    assert_eq!(state.history.len(), 1);
    assert_eq!(state.history_index, 0);
}

#[tokio::test]
async fn test_browser_controller_multiple_navigations() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_2";
    
    // Navigate to multiple URLs
    controller.navigate(tab_id, "https://page1.com").await.unwrap();
    controller.navigate(tab_id, "https://page2.com").await.unwrap();
    let state = controller.navigate(tab_id, "https://page3.com").await.unwrap();
    
    assert_eq!(state.current_url, "https://page3.com");
    assert_eq!(state.history.len(), 3);
    assert_eq!(state.history_index, 2);
    assert!(state.can_go_back);
    assert!(!state.can_go_forward);
}

#[tokio::test]
async fn test_browser_controller_go_back() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_3";
    
    controller.navigate(tab_id, "https://page1.com").await.unwrap();
    controller.navigate(tab_id, "https://page2.com").await.unwrap();
    
    let result = controller.go_back(tab_id).await.unwrap();
    
    assert_eq!(result, Some("https://page1.com".to_string()));
}

#[tokio::test]
async fn test_browser_controller_go_forward() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_4";
    
    controller.navigate(tab_id, "https://page1.com").await.unwrap();
    controller.navigate(tab_id, "https://page2.com").await.unwrap();
    controller.go_back(tab_id).await.unwrap();
    
    let result = controller.go_forward(tab_id).await.unwrap();
    
    assert_eq!(result, Some("https://page2.com".to_string()));
}

#[tokio::test]
async fn test_browser_controller_go_back_at_start() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_5";
    
    controller.navigate(tab_id, "https://page1.com").await.unwrap();
    
    let result = controller.go_back(tab_id).await.unwrap();
    
    assert!(result.is_none()); // Can't go back from first page
}

#[tokio::test]
async fn test_browser_controller_reload() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_6";
    
    controller.navigate(tab_id, "https://example.com").await.unwrap();
    
    let result = controller.reload(tab_id).await.unwrap();
    
    assert_eq!(result, Some("https://example.com".to_string()));
}

#[tokio::test]
async fn test_browser_controller_stop_loading() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_7";
    
    controller.navigate(tab_id, "https://example.com").await.unwrap();
    controller.stop_loading(tab_id).await;
    
    let state = controller.get_state(tab_id).await.unwrap();
    assert!(!state.is_loading);
}

#[tokio::test]
async fn test_browser_controller_update_title() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_8";
    
    controller.navigate(tab_id, "https://example.com").await.unwrap();
    controller.update_title(tab_id, "Example Domain").await;
    
    let state = controller.get_state(tab_id).await.unwrap();
    assert_eq!(state.title, "Example Domain");
    
    // Title should also be updated in history
    assert_eq!(state.history[0].title, "Example Domain");
}

#[tokio::test]
async fn test_browser_controller_close_tab() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_9";
    
    controller.navigate(tab_id, "https://example.com").await.unwrap();
    controller.close_tab(tab_id).await;
    
    let state = controller.get_state(tab_id).await;
    assert!(state.is_none());
}

#[tokio::test]
async fn test_browser_settings_default() {
    let settings = BrowserSettings::default();
    
    assert!(settings.javascript_enabled);
    assert!(settings.cookies_enabled);
    assert!(settings.dns_over_https);
    assert!(settings.block_trackers);
    assert!(!settings.block_ads);
    assert!(settings.stealth_mode);
    assert!(!settings.headless_mode);
    assert_eq!(settings.language, "en-US");
    assert_eq!(settings.timezone, "America/New_York");
    assert!(matches!(settings.webrtc_policy, WebRtcPolicy::DisableNonProxiedUdp));
    assert!(matches!(settings.engine_type, BrowserEngineType::System));
}

#[tokio::test]
async fn test_browser_settings_modification() {
    let controller = BrowserController::new();
    
    let mut settings = controller.get_settings().await;
    settings.block_ads = true;
    settings.language = "de-DE".to_string();
    settings.webrtc_policy = WebRtcPolicy::Disabled;
    
    controller.set_settings(settings).await;
    
    let updated = controller.get_settings().await;
    assert!(updated.block_ads);
    assert_eq!(updated.language, "de-DE");
    assert!(matches!(updated.webrtc_policy, WebRtcPolicy::Disabled));
}

#[tokio::test]
async fn test_webrtc_policy_variants() {
    assert!(matches!(WebRtcPolicy::Default, WebRtcPolicy::Default));
    assert!(matches!(WebRtcPolicy::DisableNonProxiedUdp, WebRtcPolicy::DisableNonProxiedUdp));
    assert!(matches!(WebRtcPolicy::Disabled, WebRtcPolicy::Disabled));
    
    // Default should be DisableNonProxiedUdp
    let default_policy = WebRtcPolicy::default();
    assert!(matches!(default_policy, WebRtcPolicy::DisableNonProxiedUdp));
}

#[tokio::test]
async fn test_history_item_creation() {
    let item = HistoryItem {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    assert_eq!(item.url, "https://example.com");
    assert_eq!(item.title, "Example");
    assert!(item.timestamp > 0);
}

#[tokio::test]
async fn test_browser_controller_get_all_states() {
    let controller = BrowserController::new();
    
    controller.navigate("tab_1", "https://page1.com").await.unwrap();
    controller.navigate("tab_2", "https://page2.com").await.unwrap();
    controller.navigate("tab_3", "https://page3.com").await.unwrap();
    
    let all_states = controller.get_all_states().await;
    
    assert_eq!(all_states.len(), 3);
}

#[tokio::test]
async fn test_browser_controller_set_loading() {
    let controller = BrowserController::new();
    let tab_id = "test_tab_loading";
    
    controller.navigate(tab_id, "https://example.com").await.unwrap();
    
    // Set loading to false
    controller.set_loading(tab_id, false).await;
    let state = controller.get_state(tab_id).await.unwrap();
    assert!(!state.is_loading);
    
    // Set loading to true
    controller.set_loading(tab_id, true).await;
    let state = controller.get_state(tab_id).await.unwrap();
    assert!(state.is_loading);
}
