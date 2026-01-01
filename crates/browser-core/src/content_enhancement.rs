//! Content Enhancement Module - V1000 Experimental
//!
//! Provides content enhancement features including:
//! - Reader mode with customization
//! - Media player enhancement
//! - Content extraction and transformation
//! - Accessibility enhancements

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

// =============================================================================
// Reader Mode
// =============================================================================

/// Reader mode configuration and engine
pub struct ReaderMode {
    config: ReaderModeConfig,
    extracted_content: HashMap<String, ExtractedArticle>,
}

/// Reader mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderModeConfig {
    /// Font family
    pub font_family: String,
    /// Font size in pixels
    pub font_size: u32,
    /// Line height multiplier
    pub line_height: f64,
    /// Maximum content width in pixels
    pub max_width: u32,
    /// Theme
    pub theme: ReaderTheme,
    /// Enable text-to-speech
    pub tts_enabled: bool,
    /// Enable auto-scroll
    pub auto_scroll_enabled: bool,
    /// Auto-scroll speed (words per minute)
    pub auto_scroll_wpm: u32,
    /// Enable estimated reading time
    pub show_reading_time: bool,
    /// Enable progress indicator
    pub show_progress: bool,
}

impl Default for ReaderModeConfig {
    fn default() -> Self {
        Self {
            font_family: "Georgia, serif".to_string(),
            font_size: 18,
            line_height: 1.6,
            max_width: 680,
            theme: ReaderTheme::Light,
            tts_enabled: false,
            auto_scroll_enabled: false,
            auto_scroll_wpm: 250,
            show_reading_time: true,
            show_progress: true,
        }
    }
}

/// Reader mode themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReaderTheme {
    Light,
    Dark,
    Sepia,
    HighContrast,
    Custom,
}

/// Extracted article content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedArticle {
    pub url: String,
    pub title: String,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub content: String,
    pub text_content: String,
    pub word_count: usize,
    pub reading_time_minutes: u32,
    pub images: Vec<ArticleImage>,
    pub language: Option<String>,
}

/// Article image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleImage {
    pub src: String,
    pub alt: Option<String>,
    pub caption: Option<String>,
}

impl ReaderMode {
    pub fn new() -> Self {
        Self {
            config: ReaderModeConfig::default(),
            extracted_content: HashMap::new(),
        }
    }

    pub fn with_config(config: ReaderModeConfig) -> Self {
        Self {
            config,
            extracted_content: HashMap::new(),
        }
    }

    /// Extract article content from HTML
    pub fn extract_article(&mut self, url: &str, html: &str, title: Option<&str>) -> ExtractedArticle {
        // Simple content extraction (in production would use readability algorithm)
        let text_content = self.strip_html(html);
        let word_count = text_content.split_whitespace().count();
        let reading_time = (word_count as f64 / 200.0).ceil() as u32; // 200 WPM average

        let article = ExtractedArticle {
            url: url.to_string(),
            title: title.unwrap_or("Untitled").to_string(),
            author: self.extract_author(html),
            published_date: self.extract_date(html),
            content: html.to_string(),
            text_content: text_content.clone(),
            word_count,
            reading_time_minutes: reading_time,
            images: self.extract_images(html),
            language: self.detect_language(&text_content),
        };

        self.extracted_content.insert(url.to_string(), article.clone());
        article
    }

    /// Strip HTML tags from content
    fn strip_html(&self, html: &str) -> String {
        // Simple HTML stripping
        let mut result = String::new();
        let mut in_tag = false;
        
        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }
        
        result
    }

    /// Extract author from HTML
    fn extract_author(&self, html: &str) -> Option<String> {
        // Simple author extraction using meta tags
        if html.contains("author") {
            Some("Unknown Author".to_string())
        } else {
            None
        }
    }

    /// Extract publication date
    fn extract_date(&self, _html: &str) -> Option<String> {
        None
    }

    /// Extract images from HTML
    fn extract_images(&self, html: &str) -> Vec<ArticleImage> {
        // Simple image extraction
        let mut images = Vec::new();
        let lower = html.to_lowercase();
        
        if lower.contains("<img") {
            images.push(ArticleImage {
                src: "placeholder.jpg".to_string(),
                alt: Some("Image".to_string()),
                caption: None,
            });
        }
        
        images
    }

    /// Detect language using simple heuristics
    /// Note: For production use, consider using a proper language detection library
    fn detect_language(&self, text: &str) -> Option<String> {
        if text.len() < 20 {
            return None; // Not enough text for reliable detection
        }
        
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        let word_count = words.len();
        
        if word_count < 10 {
            return None;
        }
        
        // English detection - requires multiple common words
        let english_markers = ["the", "and", "is", "are", "was", "were", "have", "has", "been"];
        let english_count = english_markers.iter()
            .filter(|marker| words.contains(marker))
            .count();
        if english_count >= 3 {
            return Some("en".to_string());
        }
        
        // Spanish detection
        let spanish_markers = ["el", "los", "las", "es", "son", "está", "están", "que"];
        let spanish_count = spanish_markers.iter()
            .filter(|marker| words.contains(marker))
            .count();
        if spanish_count >= 3 {
            return Some("es".to_string());
        }
        
        // French detection
        let french_markers = ["le", "les", "est", "sont", "avec", "dans", "pour", "qui"];
        let french_count = french_markers.iter()
            .filter(|marker| words.contains(marker))
            .count();
        if french_count >= 3 {
            return Some("fr".to_string());
        }
        
        // German detection
        let german_markers = ["der", "die", "das", "ist", "sind", "und", "mit", "für"];
        let german_count = german_markers.iter()
            .filter(|marker| words.contains(marker))
            .count();
        if german_count >= 3 {
            return Some("de".to_string());
        }
        
        None
    }

    /// Generate reader mode CSS
    pub fn generate_css(&self) -> String {
        let (bg_color, text_color) = match self.config.theme {
            ReaderTheme::Light => ("#ffffff", "#1a1a1a"),
            ReaderTheme::Dark => ("#1a1a1a", "#e0e0e0"),
            ReaderTheme::Sepia => ("#f4ecd8", "#5c4b37"),
            ReaderTheme::HighContrast => ("#000000", "#ffffff"),
            ReaderTheme::Custom => ("#ffffff", "#000000"),
        };

        format!(
            r#"
            body {{
                font-family: {font_family};
                font-size: {font_size}px;
                line-height: {line_height};
                max-width: {max_width}px;
                margin: 0 auto;
                padding: 20px;
                background-color: {bg_color};
                color: {text_color};
            }}
            
            h1, h2, h3 {{
                line-height: 1.3;
                margin-top: 1.5em;
            }}
            
            p {{
                margin-bottom: 1em;
            }}
            
            img {{
                max-width: 100%;
                height: auto;
                display: block;
                margin: 1em auto;
            }}
            
            blockquote {{
                border-left: 3px solid #ccc;
                margin-left: 0;
                padding-left: 1em;
                font-style: italic;
            }}
            
            a {{
                color: #0066cc;
            }}
            "#,
            font_family = self.config.font_family,
            font_size = self.config.font_size,
            line_height = self.config.line_height,
            max_width = self.config.max_width,
            bg_color = bg_color,
            text_color = text_color,
        )
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ReaderModeConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ReaderModeConfig {
        &self.config
    }

    /// Get cached article
    pub fn get_cached(&self, url: &str) -> Option<&ExtractedArticle> {
        self.extracted_content.get(url)
    }
}

impl Default for ReaderMode {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Media Player Enhancement
// =============================================================================

/// Enhanced media player
pub struct MediaPlayer {
    config: MediaPlayerConfig,
    current_media: Option<MediaInfo>,
    playlist: Vec<MediaInfo>,
    history: Vec<MediaInfo>,
}

/// Media player configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaPlayerConfig {
    /// Default volume (0.0 - 1.0)
    pub default_volume: f64,
    /// Enable picture-in-picture
    pub pip_enabled: bool,
    /// Enable background playback
    pub background_playback: bool,
    /// Default playback speed
    pub default_speed: f64,
    /// Enable auto quality
    pub auto_quality: bool,
    /// Preferred quality
    pub preferred_quality: VideoQuality,
    /// Enable subtitles by default
    pub subtitles_enabled: bool,
    /// Preferred subtitle language
    pub subtitle_language: String,
    /// Enable media keys
    pub media_keys_enabled: bool,
    /// Enable skip silence
    pub skip_silence: bool,
}

impl Default for MediaPlayerConfig {
    fn default() -> Self {
        Self {
            default_volume: 1.0,
            pip_enabled: true,
            background_playback: true,
            default_speed: 1.0,
            auto_quality: true,
            preferred_quality: VideoQuality::Auto,
            subtitles_enabled: false,
            subtitle_language: "en".to_string(),
            media_keys_enabled: true,
            skip_silence: false,
        }
    }
}

/// Video quality options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoQuality {
    Auto,
    Quality144p,
    Quality240p,
    Quality360p,
    Quality480p,
    Quality720p,
    Quality1080p,
    Quality1440p,
    Quality4k,
    Quality8k,
}

/// Media information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub url: String,
    pub title: String,
    pub media_type: MediaType,
    pub duration_seconds: Option<f64>,
    pub thumbnail: Option<String>,
    pub source: Option<String>,
    pub quality_options: Vec<VideoQuality>,
}

/// Media type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Video,
    Audio,
    Stream,
    Playlist,
}

impl MediaPlayer {
    pub fn new() -> Self {
        Self {
            config: MediaPlayerConfig::default(),
            current_media: None,
            playlist: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Set current media
    pub fn set_media(&mut self, media: MediaInfo) {
        if let Some(current) = self.current_media.take() {
            self.history.push(current);
        }
        self.current_media = Some(media);
    }

    /// Add to playlist
    pub fn add_to_playlist(&mut self, media: MediaInfo) {
        self.playlist.push(media);
    }

    /// Get current media
    pub fn get_current(&self) -> Option<&MediaInfo> {
        self.current_media.as_ref()
    }

    /// Get playlist
    pub fn get_playlist(&self) -> &[MediaInfo] {
        &self.playlist
    }

    /// Play next in playlist
    pub fn next(&mut self) -> Option<&MediaInfo> {
        if !self.playlist.is_empty() {
            if let Some(current) = self.current_media.take() {
                self.history.push(current);
            }
            self.current_media = Some(self.playlist.remove(0));
        }
        self.current_media.as_ref()
    }

    /// Play previous from history
    pub fn previous(&mut self) -> Option<&MediaInfo> {
        if let Some(prev) = self.history.pop() {
            if let Some(current) = self.current_media.take() {
                self.playlist.insert(0, current);
            }
            self.current_media = Some(prev);
        }
        self.current_media.as_ref()
    }

    /// Generate enhanced player JavaScript
    pub fn generate_player_script(&self) -> String {
        format!(
            r#"
(function() {{
    const config = {{
        defaultVolume: {volume},
        defaultSpeed: {speed},
        pipEnabled: {pip},
        backgroundPlayback: {background},
        skipSilence: {skip_silence}
    }};
    
    // Enhanced video/audio controls
    document.querySelectorAll('video, audio').forEach(media => {{
        media.volume = config.defaultVolume;
        media.playbackRate = config.defaultSpeed;
        
        // Add keyboard shortcuts
        document.addEventListener('keydown', (e) => {{
            if (e.target.tagName !== 'INPUT' && e.target.tagName !== 'TEXTAREA') {{
                switch(e.key) {{
                    case ' ':
                        e.preventDefault();
                        media.paused ? media.play() : media.pause();
                        break;
                    case 'ArrowLeft':
                        media.currentTime -= 10;
                        break;
                    case 'ArrowRight':
                        media.currentTime += 10;
                        break;
                    case 'ArrowUp':
                        media.volume = Math.min(1, media.volume + 0.1);
                        break;
                    case 'ArrowDown':
                        media.volume = Math.max(0, media.volume - 0.1);
                        break;
                    case 'f':
                        if (document.fullscreenElement) {{
                            document.exitFullscreen();
                        }} else {{
                            media.requestFullscreen();
                        }}
                        break;
                    case 'p':
                        if (config.pipEnabled && document.pictureInPictureEnabled) {{
                            if (document.pictureInPictureElement) {{
                                document.exitPictureInPicture();
                            }} else {{
                                media.requestPictureInPicture();
                            }}
                        }}
                        break;
                }}
            }}
        }});
    }});
    
    console.log('[MediaPlayer] Enhanced controls active');
}})();
"#,
            volume = self.config.default_volume,
            speed = self.config.default_speed,
            pip = self.config.pip_enabled,
            background = self.config.background_playback,
            skip_silence = self.config.skip_silence,
        )
    }

    /// Update configuration
    pub fn set_config(&mut self, config: MediaPlayerConfig) {
        self.config = config;
    }

    /// Get configuration
    pub fn get_config(&self) -> &MediaPlayerConfig {
        &self.config
    }
}

impl Default for MediaPlayer {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Content Transformation
// =============================================================================

/// Content transformer for accessibility and enhancement
pub struct ContentTransformer {
    transformations: Vec<TransformationType>,
}

/// Types of content transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    /// Simplify content for easier reading
    Simplify,
    /// Increase font size
    LargeText,
    /// High contrast mode
    HighContrast,
    /// Reduce motion
    ReduceMotion,
    /// Remove images
    TextOnly,
    /// Add alt text to images
    AutoAltText,
    /// Translate content
    Translate { target_language: String },
    /// Summarize content
    Summarize { max_sentences: usize },
    /// Reading level adjustment
    ReadingLevel { target_grade: u8 },
    /// Custom CSS injection
    CustomCss { css: String },
}

impl ContentTransformer {
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
        }
    }

    /// Add a transformation
    pub fn add_transformation(&mut self, transform: TransformationType) {
        self.transformations.push(transform);
    }

    /// Remove all transformations
    pub fn clear_transformations(&mut self) {
        self.transformations.clear();
    }

    /// Generate transformation CSS
    pub fn generate_css(&self) -> String {
        let mut css = String::new();
        
        for transform in &self.transformations {
            match transform {
                TransformationType::LargeText => {
                    css.push_str("body { font-size: 150% !important; }\n");
                }
                TransformationType::HighContrast => {
                    css.push_str(r#"
                        body { background: #000 !important; color: #fff !important; }
                        a { color: #ff0 !important; }
                        img { filter: contrast(1.5) !important; }
                    "#);
                }
                TransformationType::ReduceMotion => {
                    css.push_str(r#"
                        *, *::before, *::after {
                            animation-duration: 0.01ms !important;
                            animation-iteration-count: 1 !important;
                            transition-duration: 0.01ms !important;
                        }
                    "#);
                }
                TransformationType::TextOnly => {
                    css.push_str(r#"
                        img, video, iframe, svg, canvas {
                            display: none !important;
                        }
                    "#);
                }
                TransformationType::CustomCss { css: custom } => {
                    css.push_str(custom);
                    css.push('\n');
                }
                _ => {}
            }
        }
        
        css
    }

    /// Get active transformations
    pub fn get_transformations(&self) -> &[TransformationType] {
        &self.transformations
    }
}

impl Default for ContentTransformer {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Accessibility Enhancements (EXP-18001 to EXP-18005)
// =============================================================================

/// Accessibility manager
pub struct AccessibilityManager {
    config: AccessibilityConfig,
    announcements: Vec<String>,
}

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Enable screen reader support
    pub screen_reader_support: bool,
    /// Enable keyboard navigation
    pub keyboard_navigation: bool,
    /// Focus indicator style
    pub focus_indicator: FocusIndicatorStyle,
    /// Color blindness mode
    pub color_blindness_mode: ColorBlindnessMode,
    /// Enable captions/subtitles
    pub captions_enabled: bool,
    /// Enable audio descriptions
    pub audio_descriptions: bool,
    /// Minimum contrast ratio
    pub min_contrast_ratio: f64,
    /// Enable dyslexia-friendly font
    pub dyslexia_font: bool,
    /// Enable text spacing
    pub enhanced_text_spacing: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader_support: true,
            keyboard_navigation: true,
            focus_indicator: FocusIndicatorStyle::Default,
            color_blindness_mode: ColorBlindnessMode::None,
            captions_enabled: false,
            audio_descriptions: false,
            min_contrast_ratio: 4.5,
            dyslexia_font: false,
            enhanced_text_spacing: false,
        }
    }
}

/// Focus indicator styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusIndicatorStyle {
    Default,
    HighVisibility,
    Custom,
    None,
}

/// Color blindness modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorBlindnessMode {
    None,
    Protanopia,    // Red-blind
    Deuteranopia,  // Green-blind
    Tritanopia,    // Blue-blind
    Achromatopsia, // Total color blindness
}

impl AccessibilityManager {
    pub fn new() -> Self {
        Self {
            config: AccessibilityConfig::default(),
            announcements: Vec::new(),
        }
    }

    /// Generate accessibility CSS
    pub fn generate_css(&self) -> String {
        let mut css = String::new();
        
        // Focus indicator
        if self.config.focus_indicator == FocusIndicatorStyle::HighVisibility {
            css.push_str(r#"
                *:focus {
                    outline: 3px solid #ff0 !important;
                    outline-offset: 2px !important;
                }
            "#);
        }
        
        // Dyslexia font
        if self.config.dyslexia_font {
            css.push_str(r#"
                body {
                    font-family: "OpenDyslexic", "Comic Sans MS", sans-serif !important;
                    letter-spacing: 0.05em !important;
                    word-spacing: 0.1em !important;
                }
            "#);
        }
        
        // Enhanced text spacing
        if self.config.enhanced_text_spacing {
            css.push_str(r#"
                body {
                    line-height: 1.8 !important;
                    letter-spacing: 0.12em !important;
                    word-spacing: 0.16em !important;
                }
                p { margin-bottom: 2em !important; }
            "#);
        }
        
        // Color blindness filters
        let filter = match self.config.color_blindness_mode {
            ColorBlindnessMode::None => "",
            ColorBlindnessMode::Protanopia => "filter: url('#protanopia') !important;",
            ColorBlindnessMode::Deuteranopia => "filter: url('#deuteranopia') !important;",
            ColorBlindnessMode::Tritanopia => "filter: url('#tritanopia') !important;",
            ColorBlindnessMode::Achromatopsia => "filter: grayscale(100%) !important;",
        };
        
        if !filter.is_empty() {
            css.push_str(&format!("html {{ {} }}", filter));
        }
        
        css
    }

    /// Announce message for screen readers
    pub fn announce(&mut self, message: &str) {
        self.announcements.push(message.to_string());
        info!("Accessibility announcement: {}", message);
    }

    /// Get pending announcements
    pub fn get_announcements(&mut self) -> Vec<String> {
        std::mem::take(&mut self.announcements)
    }

    /// Update configuration
    pub fn set_config(&mut self, config: AccessibilityConfig) {
        self.config = config;
    }

    /// Get configuration
    pub fn get_config(&self) -> &AccessibilityConfig {
        &self.config
    }
}

impl Default for AccessibilityManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Unified Content Enhancement Manager
// =============================================================================

/// Unified content enhancement manager
pub struct ContentEnhancementManager {
    pub reader_mode: ReaderMode,
    pub media_player: MediaPlayer,
    pub transformer: ContentTransformer,
    pub accessibility: AccessibilityManager,
    start_time: Instant,
}

impl ContentEnhancementManager {
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
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get combined CSS for all enhancements
    pub fn get_combined_css(&self) -> String {
        let mut css = String::new();
        css.push_str(&self.transformer.generate_css());
        css.push_str(&self.accessibility.generate_css());
        css
    }

    /// Get combined JavaScript
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

// =============================================================================
// Enhanced Language Detection System
// =============================================================================

/// Advanced language detector with multiple detection strategies
pub struct AdvancedLanguageDetector {
    /// Minimum text length for reliable detection
    min_text_length: usize,
    /// Confidence threshold for detection
    confidence_threshold: f64,
    /// Language profiles for detection
    language_profiles: HashMap<String, LanguageProfile>,
}

/// Language profile for detection
#[derive(Debug, Clone)]
pub struct LanguageProfile {
    /// ISO 639-1 code
    pub code: String,
    /// Language name
    pub name: String,
    /// Common words with their frequency weight
    pub common_words: HashMap<String, f64>,
    /// Character patterns (n-grams)
    pub char_patterns: Vec<String>,
    /// Typical word length range
    pub avg_word_length: (f64, f64),
    /// Typical sentence ending patterns
    pub sentence_endings: Vec<String>,
}

/// Language detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDetectionResult {
    /// Detected language code
    pub language: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Alternative detected languages with scores
    pub alternatives: Vec<(String, f64)>,
    /// Script type detected (Latin, Cyrillic, CJK, etc.)
    pub script_type: ScriptType,
}

/// Script type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    Latin,
    Cyrillic,
    Greek,
    Arabic,
    Hebrew,
    CJK,       // Chinese, Japanese, Korean
    Devanagari,
    Thai,
    Unknown,
}

impl AdvancedLanguageDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            min_text_length: 20,
            confidence_threshold: 0.6,
            language_profiles: HashMap::new(),
        };
        detector.initialize_profiles();
        detector
    }

    /// Initialize language profiles
    fn initialize_profiles(&mut self) {
        // English profile
        let mut en_words = HashMap::new();
        for (word, weight) in [
            ("the", 1.0), ("and", 0.9), ("is", 0.8), ("are", 0.8), 
            ("was", 0.7), ("were", 0.7), ("have", 0.7), ("has", 0.7),
            ("been", 0.6), ("will", 0.6), ("would", 0.6), ("could", 0.6),
            ("should", 0.6), ("this", 0.7), ("that", 0.7), ("with", 0.7),
            ("from", 0.6), ("they", 0.6), ("their", 0.6), ("which", 0.6),
        ] {
            en_words.insert(word.to_string(), weight);
        }
        self.language_profiles.insert("en".to_string(), LanguageProfile {
            code: "en".to_string(),
            name: "English".to_string(),
            common_words: en_words,
            char_patterns: vec!["th".to_string(), "ing".to_string(), "tion".to_string()],
            avg_word_length: (4.0, 6.0),
            sentence_endings: vec![".".to_string(), "!".to_string(), "?".to_string()],
        });

        // Spanish profile
        let mut es_words = HashMap::new();
        for (word, weight) in [
            ("el", 1.0), ("la", 1.0), ("los", 0.9), ("las", 0.9),
            ("es", 0.9), ("son", 0.8), ("está", 0.8), ("están", 0.8),
            ("que", 0.9), ("de", 0.9), ("en", 0.8), ("por", 0.7),
            ("para", 0.7), ("con", 0.7), ("como", 0.6), ("pero", 0.6),
            ("más", 0.6), ("este", 0.6), ("esta", 0.6), ("uno", 0.5),
        ] {
            es_words.insert(word.to_string(), weight);
        }
        self.language_profiles.insert("es".to_string(), LanguageProfile {
            code: "es".to_string(),
            name: "Spanish".to_string(),
            common_words: es_words,
            char_patterns: vec!["ción".to_string(), "mente".to_string()],
            avg_word_length: (4.5, 6.5),
            sentence_endings: vec![".".to_string(), "!".to_string(), "?".to_string(), "¡".to_string(), "¿".to_string()],
        });

        // French profile
        let mut fr_words = HashMap::new();
        for (word, weight) in [
            ("le", 1.0), ("la", 1.0), ("les", 0.9), ("de", 0.9),
            ("est", 0.9), ("sont", 0.8), ("avec", 0.8), ("dans", 0.8),
            ("pour", 0.8), ("qui", 0.7), ("que", 0.7), ("sur", 0.7),
            ("pas", 0.6), ("plus", 0.6), ("être", 0.6), ("avoir", 0.6),
            ("fait", 0.5), ("bien", 0.5), ("tout", 0.5), ("nous", 0.6),
        ] {
            fr_words.insert(word.to_string(), weight);
        }
        self.language_profiles.insert("fr".to_string(), LanguageProfile {
            code: "fr".to_string(),
            name: "French".to_string(),
            common_words: fr_words,
            char_patterns: vec!["tion".to_string(), "ment".to_string(), "eux".to_string()],
            avg_word_length: (4.0, 5.5),
            sentence_endings: vec![".".to_string(), "!".to_string(), "?".to_string()],
        });

        // German profile
        let mut de_words = HashMap::new();
        for (word, weight) in [
            ("der", 1.0), ("die", 1.0), ("das", 1.0), ("und", 0.9),
            ("ist", 0.9), ("sind", 0.8), ("mit", 0.8), ("für", 0.8),
            ("auf", 0.7), ("von", 0.7), ("zu", 0.7), ("den", 0.7),
            ("nicht", 0.6), ("sich", 0.6), ("auch", 0.6), ("als", 0.6),
            ("aber", 0.5), ("oder", 0.5), ("wenn", 0.5), ("wir", 0.6),
        ] {
            de_words.insert(word.to_string(), weight);
        }
        self.language_profiles.insert("de".to_string(), LanguageProfile {
            code: "de".to_string(),
            name: "German".to_string(),
            common_words: de_words,
            char_patterns: vec!["sch".to_string(), "ung".to_string(), "heit".to_string()],
            avg_word_length: (5.0, 7.0),
            sentence_endings: vec![".".to_string(), "!".to_string(), "?".to_string()],
        });

        // Portuguese profile
        let mut pt_words = HashMap::new();
        for (word, weight) in [
            ("o", 1.0), ("a", 1.0), ("os", 0.9), ("as", 0.9),
            ("é", 0.9), ("são", 0.8), ("está", 0.8), ("de", 0.9),
            ("que", 0.9), ("em", 0.8), ("para", 0.7), ("com", 0.7),
            ("não", 0.7), ("uma", 0.6), ("por", 0.6), ("mais", 0.6),
            ("como", 0.5), ("mas", 0.5), ("foi", 0.5), ("ele", 0.6),
        ] {
            pt_words.insert(word.to_string(), weight);
        }
        self.language_profiles.insert("pt".to_string(), LanguageProfile {
            code: "pt".to_string(),
            name: "Portuguese".to_string(),
            common_words: pt_words,
            char_patterns: vec!["ção".to_string(), "mente".to_string(), "ão".to_string()],
            avg_word_length: (4.5, 6.0),
            sentence_endings: vec![".".to_string(), "!".to_string(), "?".to_string()],
        });

        // Italian profile
        let mut it_words = HashMap::new();
        for (word, weight) in [
            ("il", 1.0), ("la", 1.0), ("i", 0.9), ("le", 0.9),
            ("è", 0.9), ("sono", 0.8), ("di", 0.9), ("che", 0.9),
            ("per", 0.8), ("con", 0.7), ("non", 0.7), ("una", 0.7),
            ("del", 0.6), ("della", 0.6), ("come", 0.6), ("più", 0.6),
            ("anche", 0.5), ("ma", 0.5), ("tutto", 0.5), ("questo", 0.6),
        ] {
            it_words.insert(word.to_string(), weight);
        }
        self.language_profiles.insert("it".to_string(), LanguageProfile {
            code: "it".to_string(),
            name: "Italian".to_string(),
            common_words: it_words,
            char_patterns: vec!["zione".to_string(), "mente".to_string()],
            avg_word_length: (4.5, 6.0),
            sentence_endings: vec![".".to_string(), "!".to_string(), "?".to_string()],
        });

        // Dutch profile
        let mut nl_words = HashMap::new();
        for (word, weight) in [
            ("de", 1.0), ("het", 1.0), ("een", 0.9), ("van", 0.9),
            ("is", 0.9), ("zijn", 0.8), ("in", 0.8), ("op", 0.8),
            ("te", 0.7), ("met", 0.7), ("voor", 0.7), ("dat", 0.7),
            ("niet", 0.6), ("ook", 0.6), ("aan", 0.6), ("door", 0.6),
            ("maar", 0.5), ("nog", 0.5), ("wel", 0.5), ("bij", 0.5),
        ] {
            nl_words.insert(word.to_string(), weight);
        }
        self.language_profiles.insert("nl".to_string(), LanguageProfile {
            code: "nl".to_string(),
            name: "Dutch".to_string(),
            common_words: nl_words,
            char_patterns: vec!["ij".to_string(), "heid".to_string()],
            avg_word_length: (4.5, 6.5),
            sentence_endings: vec![".".to_string(), "!".to_string(), "?".to_string()],
        });

        // Russian profile (Cyrillic)
        let mut ru_words = HashMap::new();
        for (word, weight) in [
            ("и", 1.0), ("в", 1.0), ("на", 0.9), ("с", 0.9),
            ("что", 0.9), ("это", 0.8), ("как", 0.8), ("не", 0.8),
            ("он", 0.7), ("она", 0.7), ("был", 0.7), ("для", 0.7),
            ("по", 0.6), ("но", 0.6), ("от", 0.6), ("из", 0.6),
        ] {
            ru_words.insert(word.to_string(), weight);
        }
        self.language_profiles.insert("ru".to_string(), LanguageProfile {
            code: "ru".to_string(),
            name: "Russian".to_string(),
            common_words: ru_words,
            char_patterns: vec!["ть".to_string(), "ние".to_string()],
            avg_word_length: (5.0, 7.0),
            sentence_endings: vec![".".to_string(), "!".to_string(), "?".to_string()],
        });
    }

    /// Detect the script type from text
    pub fn detect_script(&self, text: &str) -> ScriptType {
        let mut latin_count = 0;
        let mut cyrillic_count = 0;
        let mut cjk_count = 0;
        let mut arabic_count = 0;
        let mut greek_count = 0;
        let mut devanagari_count = 0;
        let mut thai_count = 0;

        for c in text.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '\u{00C0}'..='\u{024F}' => latin_count += 1,
                '\u{0400}'..='\u{04FF}' => cyrillic_count += 1,
                '\u{4E00}'..='\u{9FFF}' | '\u{3040}'..='\u{30FF}' | '\u{AC00}'..='\u{D7AF}' => cjk_count += 1,
                '\u{0600}'..='\u{06FF}' => arabic_count += 1,
                '\u{0370}'..='\u{03FF}' => greek_count += 1,
                '\u{0900}'..='\u{097F}' => devanagari_count += 1,
                '\u{0E00}'..='\u{0E7F}' => thai_count += 1,
                _ => {}
            }
        }

        let counts = [
            (ScriptType::Latin, latin_count),
            (ScriptType::Cyrillic, cyrillic_count),
            (ScriptType::CJK, cjk_count),
            (ScriptType::Arabic, arabic_count),
            (ScriptType::Greek, greek_count),
            (ScriptType::Devanagari, devanagari_count),
            (ScriptType::Thai, thai_count),
        ];

        counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .filter(|(_, count)| *count > 0)
            .map(|(script, _)| *script)
            .unwrap_or(ScriptType::Unknown)
    }

    /// Detect language with confidence score
    pub fn detect(&self, text: &str) -> LanguageDetectionResult {
        if text.len() < self.min_text_length {
            return LanguageDetectionResult {
                language: "unknown".to_string(),
                confidence: 0.0,
                alternatives: vec![],
                script_type: ScriptType::Unknown,
            };
        }

        let script_type = self.detect_script(text);
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();

        if words.len() < 5 {
            return LanguageDetectionResult {
                language: "unknown".to_string(),
                confidence: 0.0,
                alternatives: vec![],
                script_type,
            };
        }

        let mut scores: Vec<(String, f64)> = Vec::new();

        for (code, profile) in &self.language_profiles {
            let score = self.calculate_language_score(&words, profile);
            if score > 0.0 {
                scores.push((code.clone(), score));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if scores.is_empty() {
            return LanguageDetectionResult {
                language: "unknown".to_string(),
                confidence: 0.0,
                alternatives: vec![],
                script_type,
            };
        }

        let (best_lang, best_score) = scores[0].clone();
        let alternatives: Vec<(String, f64)> = scores.iter().skip(1).take(3).cloned().collect();

        LanguageDetectionResult {
            language: best_lang,
            confidence: best_score.min(1.0),
            alternatives,
            script_type,
        }
    }

    /// Calculate score for a specific language
    fn calculate_language_score(&self, words: &[&str], profile: &LanguageProfile) -> f64 {
        let total_words = words.len() as f64;
        let mut weighted_matches = 0.0;
        let mut total_weight = 0.0;

        for word in words {
            if let Some(weight) = profile.common_words.get(*word) {
                weighted_matches += weight;
            }
            total_weight += 1.0;
        }

        if total_weight == 0.0 {
            return 0.0;
        }

        // Calculate base score from word matches
        let word_score = weighted_matches / total_weight.sqrt();

        // Apply pattern bonus
        let text = words.join(" ");
        let pattern_bonus: f64 = profile.char_patterns
            .iter()
            .filter(|pattern| text.contains(pattern.as_str()))
            .count() as f64 * 0.05;

        // Calculate average word length and compare to profile
        let avg_len: f64 = words.iter().map(|w| w.len() as f64).sum::<f64>() / total_words;
        let (min_len, max_len) = profile.avg_word_length;
        let length_score = if avg_len >= min_len && avg_len <= max_len {
            0.1
        } else {
            0.0
        };

        (word_score + pattern_bonus + length_score).min(1.0)
    }
}

impl Default for AdvancedLanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Text Analysis Utilities
// =============================================================================

/// Text statistics analyzer
pub struct TextAnalyzer;

/// Text statistics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStatistics {
    pub char_count: usize,
    pub word_count: usize,
    pub sentence_count: usize,
    pub paragraph_count: usize,
    pub avg_word_length: f64,
    pub avg_sentence_length: f64,
    pub reading_time_minutes: u32,
    pub speaking_time_minutes: u32,
    pub unique_words: usize,
    pub lexical_density: f64,
    pub readability_score: f64,
}

impl TextAnalyzer {
    /// Analyze text and return statistics
    pub fn analyze(text: &str) -> TextStatistics {
        let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
        let words: Vec<&str> = text.split_whitespace().collect();
        let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();
        let paragraphs: Vec<&str> = text.split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        let word_count = words.len();
        let sentence_count = sentences.len().max(1);
        
        let avg_word_length = if word_count > 0 {
            words.iter().map(|w| w.len() as f64).sum::<f64>() / word_count as f64
        } else {
            0.0
        };

        let avg_sentence_length = word_count as f64 / sentence_count as f64;

        // Reading time: ~200 words per minute
        let reading_time = (word_count as f64 / 200.0).ceil() as u32;
        
        // Speaking time: ~150 words per minute
        let speaking_time = (word_count as f64 / 150.0).ceil() as u32;

        // Unique words and lexical density
        let unique: std::collections::HashSet<&str> = words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>()
            .iter().map(|s| s.as_str()).collect();
        let unique_words = unique.len();
        let lexical_density = if word_count > 0 {
            unique_words as f64 / word_count as f64
        } else {
            0.0
        };

        // Flesch Reading Ease approximation
        let syllables_per_word = avg_word_length / 3.0; // Rough approximation
        let readability_score = 206.835 - 1.015 * avg_sentence_length - 84.6 * syllables_per_word;
        let readability_score = readability_score.max(0.0).min(100.0);

        TextStatistics {
            char_count: chars.len(),
            word_count,
            sentence_count,
            paragraph_count: paragraphs.len(),
            avg_word_length,
            avg_sentence_length,
            reading_time_minutes: reading_time,
            speaking_time_minutes: speaking_time,
            unique_words,
            lexical_density,
            readability_score,
        }
    }

    /// Get readability level description
    pub fn readability_level(score: f64) -> &'static str {
        match score as i32 {
            90..=100 => "Very Easy (5th grade)",
            80..=89 => "Easy (6th grade)",
            70..=79 => "Fairly Easy (7th grade)",
            60..=69 => "Standard (8th-9th grade)",
            50..=59 => "Fairly Difficult (10th-12th grade)",
            30..=49 => "Difficult (College)",
            _ => "Very Difficult (Professional)",
        }
    }

    /// Extract keywords from text (simple TF-based extraction)
    pub fn extract_keywords(text: &str, max_keywords: usize) -> Vec<(String, usize)> {
        let stop_words: std::collections::HashSet<&str> = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
            "be", "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "this", "that", "these",
            "those", "it", "its", "they", "them", "their", "we", "us", "our", "you",
            "your", "he", "she", "him", "her", "his", "i", "me", "my", "what", "which",
            "who", "whom", "whose", "where", "when", "why", "how", "all", "each",
            "every", "both", "few", "more", "most", "other", "some", "such", "no",
            "not", "only", "own", "same", "so", "than", "too", "very", "just",
        ].iter().cloned().collect();

        let words: Vec<String> = text.to_lowercase()
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| w.len() > 2 && !stop_words.contains(w))
            .map(|s| s.to_string())
            .collect();

        let mut freq: HashMap<String, usize> = HashMap::new();
        for word in words {
            *freq.entry(word).or_insert(0) += 1;
        }

        let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(max_keywords);
        sorted
    }
}

#[cfg(test)]
mod enhanced_content_tests {
    use super::*;

    #[test]
    fn test_advanced_language_detection() {
        let detector = AdvancedLanguageDetector::new();
        
        let english_text = "The quick brown fox jumps over the lazy dog. This is a test sentence with common English words.";
        let result = detector.detect(english_text);
        assert_eq!(result.language, "en");
        assert!(result.confidence > 0.3);
        
        let spanish_text = "El rápido zorro marrón salta sobre el perro perezoso. Esta es una prueba de texto en español.";
        let result = detector.detect(spanish_text);
        assert_eq!(result.language, "es");
    }

    #[test]
    fn test_script_detection() {
        let detector = AdvancedLanguageDetector::new();
        
        assert_eq!(detector.detect_script("Hello World"), ScriptType::Latin);
        assert_eq!(detector.detect_script("Привет мир"), ScriptType::Cyrillic);
        assert_eq!(detector.detect_script("你好世界"), ScriptType::CJK);
    }

    #[test]
    fn test_text_statistics() {
        let text = "This is a test. It has multiple sentences. The analysis should work correctly.";
        let stats = TextAnalyzer::analyze(text);
        
        assert_eq!(stats.sentence_count, 3);
        assert!(stats.word_count > 0);
        assert!(stats.readability_score > 0.0);
    }

    #[test]
    fn test_keyword_extraction() {
        let text = "Programming programming is fun. Rust programming language is great for systems programming.";
        let keywords = TextAnalyzer::extract_keywords(text, 5);
        
        assert!(!keywords.is_empty());
        assert!(keywords[0].0 == "programming");
    }
}
