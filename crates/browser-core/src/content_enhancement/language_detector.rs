//! Language Detector Module
//!
//! Provides advanced language detection capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
#[allow(dead_code)]
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
/// Represents a LanguageProfile.
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
/// Represents a LanguageDetectionResult.
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
/// Enumeration of ScriptType variants.
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
    /// Creates a new new.
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
    /// Detect the script type of text
    ///
    /// # Arguments
    /// * `text` - The text to analyze
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
    /// Detect the language of text
    ///
    /// # Arguments
    /// * `text` - The text to analyze
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
        let mut total_weight: f64 = 0.0;

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

