//! Content Enhancement Manager Module
//!
//! Provides unified management of all content enhancement features.

use std::time::Instant;
use super::{ReaderMode, MediaPlayer, ContentTransformer, AccessibilityManager};
use tracing::info;

pub struct ContentEnhancementManager {
    pub reader_mode: ReaderMode,
    pub media_player: MediaPlayer,
    pub transformer: ContentTransformer,
    pub accessibility: AccessibilityManager,
    start_time: Instant,
}

impl ContentEnhancementManager {
    /// Creates a new new.
    pub fn new() -> Self {
        info!("Initializing Content Enhancement Manager");
        Self {
            reader_mode: ReaderMode::new(),
            media_player: MediaPlayer::new(),
            transformer: ContentTransformer::new(),
            accessibility: AccessibilityManager::new(),
            start_time: Instant::now(),
        }
    }

    /// Get uptime
    /// Get the uptime in seconds since the manager was created
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get combined CSS for all enhancements
    /// Get combined CSS from all enhancement modules
    pub fn get_combined_css(&self) -> String {
        let mut css = String::new();
        css.push_str(&self.transformer.generate_css());
        css.push_str(&self.accessibility.generate_css());
        css
    }

    /// Get combined JavaScript
    /// Get combined JavaScript from all enhancement modules
    pub fn get_combined_js(&self) -> String {
        self.media_player.generate_player_script()
    }
}

impl Default for ContentEnhancementManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{MediaInfo, MediaType, VideoQuality, TransformationType};

    #[test]
    fn test_reader_mode() {
        let mut reader = ReaderMode::new();
        let article = reader.extract_article(
            "https://example.com/article",
            "<h1>Test</h1><p>This is a test article with some content.</p>",
            Some("Test Article")
        );
        
        assert_eq!(article.title, "Test Article");
        assert!(article.word_count > 0);
        assert!(article.reading_time_minutes >= 1);
    }

    #[test]
    fn test_reader_css_generation() {
        let reader = ReaderMode::new();
        let css = reader.generate_css();
        
        assert!(css.contains("font-family"));
        assert!(css.contains("max-width"));
    }

    #[test]
    fn test_media_player() {
        let mut player = MediaPlayer::new();
        
        let media = MediaInfo {
            url: "https://example.com/video.mp4".to_string(),
            title: "Test Video".to_string(),
            media_type: MediaType::Video,
            duration_seconds: Some(120.0),
            thumbnail: None,
            source: None,
            quality_options: vec![VideoQuality::Quality720p, VideoQuality::Quality1080p],
        };
        
        player.set_media(media.clone());
        assert!(player.get_current().is_some());
        
        player.add_to_playlist(media);
        assert_eq!(player.get_playlist().len(), 1);
    }

    #[test]
    fn test_content_transformer() {
        let mut transformer = ContentTransformer::new();
        
        transformer.add_transformation(TransformationType::LargeText);
        transformer.add_transformation(TransformationType::HighContrast);
        
        let css = transformer.generate_css();
        assert!(css.contains("font-size"));
        assert!(css.contains("background"));
    }

    #[test]
    fn test_accessibility_manager() {
        let mut manager = AccessibilityManager::new();
        
        manager.announce("Page loaded");
        let announcements = manager.get_announcements();
        
        assert_eq!(announcements.len(), 1);
        assert!(announcements[0].contains("Page loaded"));
    }
}

